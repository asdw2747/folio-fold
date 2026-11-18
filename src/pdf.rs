//! PDF inspection, lossless page embedding, and imposed-sheet writing.
//!
//! Source pages are cloned into Form XObjects. Their content streams and every
//! reachable resource object (fonts, images, graphics states, colour spaces,
//! patterns, shadings) are copied with an object-ID remap, so text and vector
//! artwork stay native and embedded images are never decoded or rasterized.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};

use crate::impose::{build_plan, BindingMode, DuplexFlip, ImpositionPlan, PhysicalSheet, Side};
use crate::layout::{
    exact_pages_overflow, layout_sheet, trim_marks, LayoutSettings, Unit, PT_PER_MM,
};

const MAX_PAGE_CONTENT: usize = 256 * 1024 * 1024;
const MAX_TOTAL_CONTENT: usize = 1024 * 1024 * 1024;
const MAX_INHERITANCE_DEPTH: usize = 128;

#[derive(Clone, Debug)]
pub struct SourcePage {
    pub number: u32,
    /// Upright visual width in points, after `/Rotate` and `/UserUnit`.
    pub width_pt: f64,
    /// Upright visual height in points.
    pub height_pt: f64,
    pub rotation: i32,
    pub user_unit: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFingerprint {
    pub len: u64,
    pub modified_ns: Option<u128>,
}

impl SourceFingerprint {
    pub fn from_path(path: &Path) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;
        Some(Self {
            len: metadata.len(),
            modified_ns: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_nanos()),
        })
    }
}

#[derive(Clone, Debug)]
pub struct SourceInfo {
    pub path: PathBuf,
    pub title: String,
    pub pdf_version: String,
    pub pages: Vec<SourcePage>,
    pub fingerprint: Option<SourceFingerprint>,
}

impl SourceInfo {
    pub fn load(path: &Path) -> Result<Self> {
        let document = Document::load(path)
            .with_context(|| format!("Could not open PDF {}", path.display()))?;
        if document.is_encrypted() {
            bail!(
                "{} is encrypted. Remove the PDF password protection first.",
                path.display()
            );
        }
        let page_ids = page_ids(&document);
        if page_ids.is_empty() {
            bail!("{} contains no pages.", path.display());
        }
        let mut pages = Vec::with_capacity(page_ids.len());
        for (index, page_id) in page_ids.iter().copied().enumerate() {
            let geometry = page_geometry(&document, page_id)
                .with_context(|| format!("Could not inspect page {}", index + 1))?;
            let (width_pt, height_pt) = geometry.visual_size();
            pages.push(SourcePage {
                number: index as u32 + 1,
                width_pt,
                height_pt,
                rotation: geometry.rotation,
                user_unit: geometry.user_unit,
            });
        }
        let title = document_title(&document)
            .or_else(|| {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "Untitled PDF".to_string());
        Ok(Self {
            path: path.to_path_buf(),
            title,
            pdf_version: document.version.clone(),
            pages,
            fingerprint: SourceFingerprint::from_path(path),
        })
    }

    pub fn page_sizes(&self) -> Vec<(f64, f64)> {
        self.pages
            .iter()
            .map(|page| (page.width_pt, page.height_pt))
            .collect()
    }

    pub fn uniform_page_size(&self) -> Option<(f64, f64)> {
        let first = self.pages.first()?;
        let tolerance = 0.5;
        if self.pages.iter().all(|page| {
            (page.width_pt - first.width_pt).abs() <= tolerance
                && (page.height_pt - first.height_pt).abs() <= tolerance
        }) {
            Some((first.width_pt, first.height_pt))
        } else {
            None
        }
    }

    pub fn page_size_range(&self) -> Option<((f64, f64), (f64, f64))> {
        let first = self.pages.first()?;
        let mut min = (first.width_pt, first.height_pt);
        let mut max = min;
        for page in &self.pages {
            min.0 = min.0.min(page.width_pt);
            min.1 = min.1.min(page.height_pt);
            max.0 = max.0.max(page.width_pt);
            max.1 = max.1.max(page.height_pt);
        }
        Some((min, max))
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn size_summary(&self, unit: Unit) -> String {
        if let Some((width, height)) = self.uniform_page_size() {
            format!(
                "{} x {} {}",
                unit.format_value(width / PT_PER_MM),
                unit.format_value(height / PT_PER_MM),
                unit.label()
            )
        } else if let Some((min, max)) = self.page_size_range() {
            format!(
                "Mixed: {}–{} x {}–{} {}",
                unit.format_value(min.0 / PT_PER_MM),
                unit.format_value(max.0 / PT_PER_MM),
                unit.format_value(min.1 / PT_PER_MM),
                unit.format_value(max.1 / PT_PER_MM),
                unit.label()
            )
        } else {
            "No pages".to_string()
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExportSettings {
    pub mode: BindingMode,
    pub signature_pages: u32,
    pub duplex: DuplexFlip,
    pub layout: LayoutSettings,
    pub separate_front_back: bool,
    /// Fingerprint captured when the UI loaded the source, if available.
    pub source_fingerprint: Option<SourceFingerprint>,
}

#[derive(Clone, Debug)]
pub struct ExportReport {
    pub combined_path: PathBuf,
    pub fronts_path: Option<PathBuf>,
    pub backs_path: Option<PathBuf>,
    pub sheets: usize,
    pub output_pages: usize,
    pub bytes: u64,
    pub duration_ms: u128,
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageSet {
    Both,
    Fronts,
    Backs,
}

impl PageSet {
    fn includes(self, side: Side) -> bool {
        matches!(
            (self, side),
            (Self::Both, _) | (Self::Fronts, Side::Front) | (Self::Backs, Side::Back)
        )
    }
}

#[derive(Clone, Debug)]
struct PageGeometry {
    crop: [f64; 4],
    rotation: i32,
    user_unit: f64,
    resources: Option<Object>,
    group: Option<Object>,
}

impl PageGeometry {
    fn visual_size(&self) -> (f64, f64) {
        let width = (self.crop[2] - self.crop[0]) * self.user_unit;
        let height = (self.crop[3] - self.crop[1]) * self.user_unit;
        if self.rotation == 90 || self.rotation == 270 {
            (height, width)
        } else {
            (width, height)
        }
    }

    fn normalization_matrix(&self) -> [f64; 6] {
        let [x0, y0, x1, y1] = self.crop;
        let unit = self.user_unit;
        match self.rotation {
            90 => [0.0, -unit, unit, 0.0, -y0 * unit, x1 * unit],
            180 => [-unit, 0.0, 0.0, -unit, x1 * unit, y1 * unit],
            270 => [0.0, unit, -unit, 0.0, y1 * unit, -x0 * unit],
            _ => [unit, 0.0, 0.0, unit, -x0 * unit, -y0 * unit],
        }
    }
}

#[derive(Clone, Debug)]
struct LoadedDocument {
    document: Document,
    page_ids: Vec<ObjectId>,
    geometries: Vec<PageGeometry>,
    info: SourceInfo,
}

impl LoadedDocument {
    fn load(path: &Path) -> Result<Self> {
        let document = Document::load(path)
            .with_context(|| format!("Could not open PDF {}", path.display()))?;
        if document.is_encrypted() {
            bail!(
                "{} is encrypted. Remove the PDF password protection first.",
                path.display()
            );
        }
        let page_ids = page_ids(&document);
        if page_ids.is_empty() {
            bail!("{} contains no pages.", path.display());
        }
        let mut geometries = Vec::with_capacity(page_ids.len());
        let mut pages = Vec::with_capacity(page_ids.len());
        for (index, page_id) in page_ids.iter().copied().enumerate() {
            let geometry = page_geometry(&document, page_id)
                .with_context(|| format!("Could not inspect page {}", index + 1))?;
            let (width_pt, height_pt) = geometry.visual_size();
            pages.push(SourcePage {
                number: index as u32 + 1,
                width_pt,
                height_pt,
                rotation: geometry.rotation,
                user_unit: geometry.user_unit,
            });
            geometries.push(geometry);
        }
        let title = document_title(&document)
            .or_else(|| {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "Untitled PDF".to_string());
        let info = SourceInfo {
            path: path.to_path_buf(),
            title,
            pdf_version: document.version.clone(),
            pages,
            fingerprint: SourceFingerprint::from_path(path),
        };
        Ok(Self {
            document,
            page_ids,
            geometries,
            info,
        })
    }
}

#[derive(Debug)]
struct BuiltDocument {
    document: Document,
    page_count: usize,
}

/// Impose a source PDF and write it to `output_path`.
///
/// The combined PDF is always written. When `separate_front_back` is enabled,
/// `<name>-fronts.pdf` and `<name>-backs.pdf` are written next to it as well.
pub fn export_imposed_pdf(
    source_path: &Path,
    output_path: &Path,
    settings: &ExportSettings,
) -> Result<ExportReport> {
    export_imposed_pdf_with_cancel(
        source_path,
        output_path,
        settings,
        Arc::new(AtomicBool::new(false)),
    )
}

/// Impose a source PDF with cooperative cancellation.
///
/// A cancelled export returns an error containing `Export cancelled`. Any
/// completed files remain atomic; a file that was not completed is untouched.
pub fn export_imposed_pdf_with_cancel(
    source_path: &Path,
    output_path: &Path,
    settings: &ExportSettings,
    cancel: Arc<AtomicBool>,
) -> Result<ExportReport> {
    let started = Instant::now();
    if source_path == output_path {
        bail!("The output path must be different from the source PDF.");
    }
    if cancel.load(Ordering::Relaxed) {
        bail!("Export cancelled.");
    }
    let loaded = LoadedDocument::load(source_path)?;
    let source_pages = loaded.page_ids.len() as u32;
    let plan = build_plan(
        settings.mode,
        settings.signature_pages,
        source_pages,
        settings.duplex,
    );
    if plan.sheets.is_empty() {
        bail!("There are no pages to impose.");
    }

    let mut warnings = plan.warnings.clone();
    if let Some(expected) = settings.source_fingerprint.as_ref() {
        let current = SourceFingerprint::from_path(source_path);
        if current.as_ref() != Some(expected) {
            warnings.push(
                "The source PDF changed after it was loaded; the preview and this export may \
                 differ. Reload the PDF to refresh the preview."
                    .to_string(),
            );
        }
    }
    if exact_pages_overflow(
        settings.mode,
        &plan,
        &settings.layout,
        &loaded.info.page_sizes(),
    ) {
        warnings.push(
            "100% exact scale overflows at least one slot; use Fit to Slot for a guaranteed fit."
                .to_string(),
        );
    }
    if cancel.load(Ordering::Relaxed) {
        bail!("Export cancelled.");
    }
    let combined = build_document(&loaded, &plan, settings, PageSet::Both, &cancel)?;
    let combined_pages = combined.page_count;
    if cancel.load(Ordering::Relaxed) {
        bail!("Export cancelled.");
    }
    let bytes = save_document(combined.document, output_path)
        .with_context(|| format!("Could not write {}", output_path.display()))?;

    let (fronts_path, backs_path) = if settings.separate_front_back {
        let parent = output_path.parent().unwrap_or_else(|| Path::new("."));
        let stem = output_path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| "folio-fold".to_string());
        let fronts = parent.join(format!("{stem}-fronts.pdf"));
        let backs = parent.join(format!("{stem}-backs.pdf"));
        if cancel.load(Ordering::Relaxed) {
            bail!("Export cancelled.");
        }
        let fronts_doc = build_document(&loaded, &plan, settings, PageSet::Fronts, &cancel)?;
        save_document(fronts_doc.document, &fronts)
            .with_context(|| format!("Could not write {}", fronts.display()))?;
        if cancel.load(Ordering::Relaxed) {
            bail!("Export cancelled.");
        }
        let backs_doc = build_document(&loaded, &plan, settings, PageSet::Backs, &cancel)?;
        save_document(backs_doc.document, &backs)
            .with_context(|| format!("Could not write {}", backs.display()))?;
        (Some(fronts), Some(backs))
    } else {
        (None, None)
    };

    Ok(ExportReport {
        combined_path: output_path.to_path_buf(),
        fronts_path,
        backs_path,
        sheets: plan.sheets.len(),
        output_pages: combined_pages,
        bytes,
        duration_ms: started.elapsed().as_millis(),
        warnings,
    })
}

fn save_document(mut document: Document, path: &Path) -> Result<u64> {
    let mut bytes = Vec::new();
    document.save_to(&mut bytes)?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("Could not create a temporary file in {}", parent.display()))?;
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("Could not replace {}", path.display()))?;
    Ok(bytes.len() as u64)
}

fn build_document(
    loaded: &LoadedDocument,
    plan: &ImpositionPlan,
    settings: &ExportSettings,
    page_set: PageSet,
    cancel: &AtomicBool,
) -> Result<BuiltDocument> {
    let mut importer = PdfImporter::new(&loaded.document, &loaded.page_ids, &loaded.geometries);
    let page_sizes = loaded.info.page_sizes();

    let font_id = importer.dst.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
        "Encoding" => "WinAnsiEncoding",
    });
    let pages_id = importer.dst.new_object_id();
    let mut kids = Vec::new();

    for sheet in &plan.sheets {
        for side in [Side::Front, Side::Back] {
            if !page_set.includes(side) {
                continue;
            }
            if cancel.load(Ordering::Relaxed) {
                bail!("Export cancelled.");
            }
            let geometry = layout_sheet(settings.mode, sheet, side, &settings.layout, &page_sizes);
            let (content, used_forms) = compose_sheet_content(
                &mut importer,
                &geometry,
                sheet,
                side,
                &settings.layout,
                plan.signatures,
            )?;
            let mut content_stream = Stream::new(Dictionary::new(), content.into_bytes());
            content_stream.compress()?;
            let content_id = importer.dst.add_object(Object::Stream(content_stream));

            let mut xobjects = Dictionary::new();
            for page in used_forms.keys().copied().collect::<BTreeSet<_>>() {
                if let Some(form_id) = used_forms.get(&page) {
                    xobjects.set(format!("Fm{page}"), Object::Reference(*form_id));
                }
            }
            let resources = dictionary! {
                "ProcSet" => Object::Array(vec![
                    Object::Name(b"PDF".to_vec()),
                    Object::Name(b"Text".to_vec()),
                    Object::Name(b"ImageB".to_vec()),
                    Object::Name(b"ImageC".to_vec()),
                    Object::Name(b"ImageI".to_vec()),
                ]),
                "Font" => Object::Dictionary(dictionary! {
                    "F1" => Object::Reference(font_id),
                }),
                "XObject" => Object::Dictionary(xobjects),
            };
            let page_id = importer.dst.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => Object::Reference(pages_id),
                "MediaBox" => object_array(&[0.0, 0.0, geometry.width, geometry.height]),
                "Resources" => resources,
                "Contents" => Object::Reference(content_id),
            });
            kids.push(Object::Reference(page_id));
        }
    }

    let page_count = kids.len();
    importer.dst.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => Object::Array(kids),
            "Count" => Object::Integer(page_count as i64),
        }),
    );
    let catalog_id = importer.dst.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    importer
        .dst
        .trailer
        .set("Root", Object::Reference(catalog_id));
    let title = format!(
        "{} - FolioFold {} imposition",
        loaded.info.title,
        env!("CARGO_PKG_VERSION")
    );
    let info_id = importer.dst.add_object(dictionary! {
        "Title" => pdf_text_string(&title),
        "Producer" => Object::string_literal(format!("FolioFold {}", env!("CARGO_PKG_VERSION"))),
        "Creator" => Object::string_literal("FolioFold"),
    });
    importer.dst.trailer.set("Info", Object::Reference(info_id));
    importer.dst.version = "1.5".to_string();

    Ok(BuiltDocument {
        document: importer.dst,
        page_count,
    })
}

fn compose_sheet_content(
    importer: &mut PdfImporter<'_>,
    geometry: &crate::layout::SheetGeometry,
    sheet: &PhysicalSheet,
    side: Side,
    settings: &LayoutSettings,
    signature_count: u32,
) -> Result<(String, HashMap<u32, ObjectId>)> {
    let mut content = String::new();
    let mut used_forms = HashMap::new();
    for placement in &geometry.placements {
        let Some(page) = placement.page else {
            continue;
        };
        let form_id = importer.form_for_page(page)?;
        used_forms.insert(page, form_id);
        let matrix = placement_matrix(placement.scale, placement.page_rect, placement.rotation);
        let _ = writeln!(
            content,
            "q\n{} {} {} {} {} {} cm\n/Fm{} Do\nQ",
            number(matrix[0]),
            number(matrix[1]),
            number(matrix[2]),
            number(matrix[3]),
            number(matrix[4]),
            number(matrix[5]),
            page
        );
    }

    if settings.fold_lines && !geometry.fold_lines.is_empty() {
        let _ = writeln!(content, "q\n0.55 G\n[2.5 2.5] 0 d\n0.35 w");
        for line in &geometry.fold_lines {
            let _ = writeln!(
                content,
                "{} {} m {} {} l S",
                number(line.line.a.x),
                number(line.line.a.y),
                number(line.line.b.x),
                number(line.line.b.y)
            );
        }
        let _ = writeln!(content, "Q");
    }

    if settings.crop_marks {
        let gap = 2.0 * PT_PER_MM;
        let length = 4.0 * PT_PER_MM;
        let _ = writeln!(content, "q\n0.45 G\n0.25 w");
        for placement in &geometry.placements {
            for mark in trim_marks(placement.page_rect, gap, length) {
                let _ = writeln!(
                    content,
                    "{} {} m {} {} l S",
                    number(mark.a.x),
                    number(mark.a.y),
                    number(mark.b.x),
                    number(mark.b.y)
                );
            }
        }
        let _ = writeln!(content, "Q");
    }

    if settings.sequence_labels {
        let label = format!(
            "Sig {:02}/{:02} / Sheet {:02}/{:02} / {}",
            sheet.signature,
            signature_count.max(sheet.signature),
            sheet.sheet_in_signature,
            sheet.sheets_in_signature,
            side.label().to_uppercase()
        );
        let point = geometry.label_position;
        let _ = writeln!(
            content,
            "q\nBT\n/F1 5 Tf\n0.38 g\n{} {} Td\n({}) Tj\nET\nQ",
            number(point.x),
            number(point.y),
            escape_pdf_string(&label)
        );
    }

    Ok((content, used_forms))
}

struct PdfImporter<'a> {
    src: &'a Document,
    page_ids: &'a [ObjectId],
    geometries: &'a [PageGeometry],
    dst: Document,
    map: HashMap<ObjectId, ObjectId>,
    forms: HashMap<u32, ObjectId>,
    content_bytes: usize,
}

impl<'a> PdfImporter<'a> {
    fn new(src: &'a Document, page_ids: &'a [ObjectId], geometries: &'a [PageGeometry]) -> Self {
        Self {
            src,
            page_ids,
            geometries,
            dst: Document::with_version("1.5"),
            map: HashMap::new(),
            forms: HashMap::new(),
            content_bytes: 0,
        }
    }

    fn form_for_page(&mut self, page_number: u32) -> Result<ObjectId> {
        if let Some(id) = self.forms.get(&page_number) {
            return Ok(*id);
        }
        let index = page_number.saturating_sub(1) as usize;
        let page_id = *self
            .page_ids
            .get(index)
            .with_context(|| format!("Source page {page_number} is missing"))?;
        let geometry = self
            .geometries
            .get(index)
            .cloned()
            .with_context(|| format!("Source page {page_number} has no geometry"))?;
        let resources_id = self.import_resources(geometry.resources.as_ref())?;
        let (visual_width, visual_height) = geometry.visual_size();
        let source_content = self
            .src
            .get_page_content_with_limit(page_id, MAX_PAGE_CONTENT)
            .with_context(|| format!("Could not decode content for page {page_number}"))?;
        self.content_bytes = self
            .content_bytes
            .checked_add(source_content.len())
            .context("PDF page content size overflowed the memory budget")?;
        if self.content_bytes > MAX_TOTAL_CONTENT {
            bail!(
                "The source PDF expands past the {} MiB content budget; refusing to build an \
                 importer that could exhaust memory.",
                MAX_TOTAL_CONTENT / (1024 * 1024)
            );
        }
        let matrix = geometry.normalization_matrix();
        let prefix = format!(
            "q\n{} {} {} {} {} {} cm\n",
            number(matrix[0]),
            number(matrix[1]),
            number(matrix[2]),
            number(matrix[3]),
            number(matrix[4]),
            number(matrix[5])
        );
        let mut content = Vec::with_capacity(prefix.len() + source_content.len() + 8);
        content.extend_from_slice(prefix.as_bytes());
        content.extend_from_slice(&source_content);
        content.extend_from_slice(b"\nQ\n");

        let mut dict = dictionary! {
            "Type" => "XObject",
            "Subtype" => "Form",
            "FormType" => 1,
            "BBox" => object_array(&[0.0, 0.0, visual_width, visual_height]),
            "Resources" => Object::Reference(resources_id),
        };
        if let Some(group) = geometry.group.as_ref() {
            dict.set("Group", self.import_value(group)?);
        }
        let mut stream = Stream::new(dict, content);
        stream.compress()?;
        let id = self.dst.add_object(Object::Stream(stream));
        self.forms.insert(page_number, id);
        Ok(id)
    }

    fn import_resources(&mut self, resources: Option<&Object>) -> Result<ObjectId> {
        match resources {
            Some(Object::Reference(id)) => self.import_ref(*id),
            Some(Object::Dictionary(dict)) => {
                let object = self.import_value(&Object::Dictionary(dict.clone()))?;
                Ok(self.dst.add_object(object))
            }
            _ => Ok(self.dst.add_object(Dictionary::new())),
        }
    }

    fn import_value(&mut self, object: &Object) -> Result<Object> {
        Ok(match object {
            Object::Null => Object::Null,
            Object::Boolean(value) => Object::Boolean(*value),
            Object::Integer(value) => Object::Integer(*value),
            Object::Real(value) => Object::Real(*value),
            Object::Name(name) => Object::Name(name.clone()),
            Object::String(value, format) => Object::String(value.clone(), *format),
            Object::Array(values) => {
                let mut imported = Vec::with_capacity(values.len());
                for value in values {
                    imported.push(self.import_value(value)?);
                }
                Object::Array(imported)
            }
            Object::Dictionary(dict) => {
                let mut imported = Dictionary::new();
                for (key, value) in dict.iter() {
                    imported.set(key.clone(), self.import_value(value)?);
                }
                Object::Dictionary(imported)
            }
            Object::Stream(stream) => {
                let mut dict = Dictionary::new();
                for (key, value) in stream.dict.iter() {
                    dict.set(key.clone(), self.import_value(value)?);
                }
                Object::Stream(
                    Stream::new(dict, stream.content.clone())
                        .with_compression(stream.allows_compression),
                )
            }
            Object::Reference(id) => Object::Reference(self.import_ref(*id)?),
        })
    }

    fn import_ref(&mut self, id: ObjectId) -> Result<ObjectId> {
        if let Some(new_id) = self.map.get(&id) {
            return Ok(*new_id);
        }
        let new_id = self.dst.new_object_id();
        self.map.insert(id, new_id);
        self.dst.set_object(new_id, Object::Null);
        let raw = self
            .src
            .objects
            .get(&id)
            .cloned()
            .with_context(|| format!("Dangling PDF object reference {id:?}"))?;
        let imported = self.import_value(&raw)?;
        self.dst.set_object(new_id, imported);
        Ok(new_id)
    }
}

fn page_ids(document: &Document) -> Vec<ObjectId> {
    document.get_pages().into_values().collect()
}

fn page_geometry(document: &Document, page_id: ObjectId) -> Result<PageGeometry> {
    let media_object =
        inherited_attribute(document, page_id, b"MediaBox")?.context("Page has no MediaBox")?;
    let media = resolve_box(document, &media_object)?;
    let crop_object = inherited_attribute(document, page_id, b"CropBox")?;
    let mut crop = match crop_object {
        Some(object) => resolve_box(document, &object).unwrap_or(media),
        None => media,
    };
    crop[0] = crop[0].max(media[0]);
    crop[1] = crop[1].max(media[1]);
    crop[2] = crop[2].min(media[2]);
    crop[3] = crop[3].min(media[3]);
    if crop[2] - crop[0] < 1.0 || crop[3] - crop[1] < 1.0 {
        crop = media;
    }
    let rotation = inherited_attribute(document, page_id, b"Rotate")?
        .as_ref()
        .and_then(|object| object.as_i64().ok())
        .map(normalize_rotation)
        .unwrap_or(0);
    let user_unit = inherited_attribute(document, page_id, b"UserUnit")?
        .as_ref()
        .and_then(|object| object.as_float().ok())
        .map(f64::from)
        .filter(|value| *value > 0.0)
        .unwrap_or(1.0);
    let resources = inherited_attribute(document, page_id, b"Resources")?;
    let group = document
        .get_dictionary(page_id)
        .ok()
        .and_then(|dict| dict.get(b"Group").ok().cloned());
    Ok(PageGeometry {
        crop,
        rotation,
        user_unit,
        resources,
        group,
    })
}

fn normalize_rotation(value: i64) -> i32 {
    let rotation = value.rem_euclid(360);
    match rotation {
        45..=134 => 90,
        135..=224 => 180,
        225..=314 => 270,
        _ => 0,
    }
}

fn inherited_attribute(
    document: &Document,
    page_id: ObjectId,
    key: &[u8],
) -> Result<Option<Object>> {
    let mut current = page_id;
    let mut seen = HashSet::new();
    for _ in 0..MAX_INHERITANCE_DEPTH {
        if !seen.insert(current) {
            bail!(
                "Cycle in the page tree while reading /{}",
                String::from_utf8_lossy(key)
            );
        }
        let dict = document
            .get_dictionary(current)
            .with_context(|| format!("Page tree node {current:?} is not a dictionary"))?;
        if let Ok(value) = dict.get(key) {
            return Ok(Some(value.clone()));
        }
        let Some(parent) = dict
            .get(b"Parent")
            .ok()
            .and_then(|object| object.as_reference().ok())
        else {
            break;
        };
        current = parent;
    }
    Ok(None)
}

fn resolve_box(document: &Document, object: &Object) -> Result<[f64; 4]> {
    let (_, resolved) = document.dereference(object)?;
    let array = resolved
        .as_array()
        .context("PDF page box is not an array")?;
    if array.len() < 4 {
        bail!("PDF page box needs four numbers");
    }
    let mut values = [0.0; 4];
    for (index, value) in array.iter().take(4).enumerate() {
        values[index] = document
            .dereference(value)
            .map(|(_, value)| value.as_float().map(f64::from))
            .unwrap_or_else(|_| Ok(0.0))
            .context("PDF page box contains a non-numeric value")?;
    }
    if values[2] <= values[0] || values[3] <= values[1] {
        bail!("PDF page box has invalid dimensions");
    }
    Ok(values)
}

fn document_title(document: &Document) -> Option<String> {
    let info = document
        .trailer
        .get(b"Info")
        .ok()
        .and_then(|object| object.as_reference().ok())
        .and_then(|id| document.get_dictionary(id).ok())?;
    let title = info.get(b"Title").ok()?.as_str().ok()?;
    Some(pdf_string_to_string(title))
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty())
}

fn pdf_text_string(value: &str) -> Object {
    if value.is_ascii() {
        Object::string_literal(value)
    } else {
        let mut bytes = vec![0xFE, 0xFF];
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_be_bytes());
        }
        Object::String(bytes, lopdf::StringFormat::Literal)
    }
}

fn pdf_string_to_string(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let units: Vec<u16> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_be_bytes(*pair))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(bytes).to_string()
    }
}

fn placement_matrix(scale: f64, rect: crate::layout::RectPt, rotation: u16) -> [f64; 6] {
    match rotation % 360 {
        180 => [-scale, 0.0, 0.0, -scale, rect.x1, rect.y1],
        90 => [0.0, scale, -scale, 0.0, rect.x1, rect.y0],
        270 => [0.0, -scale, scale, 0.0, rect.x0, rect.y1],
        _ => [scale, 0.0, 0.0, scale, rect.x0, rect.y0],
    }
}

fn object_array(values: &[f64]) -> Object {
    Object::Array(
        values
            .iter()
            .map(|value| Object::Real(*value as f32))
            .collect(),
    )
}

fn number(value: f64) -> String {
    if value.abs() < 0.000_5 {
        "0".to_string()
    } else {
        format!("{value:.3}")
    }
}

fn escape_pdf_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '(' => output.push_str("\\("),
            ')' => output.push_str("\\)"),
            character if character.is_ascii() => output.push(character),
            _ => output.push('?'),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_normalization() {
        assert_eq!(normalize_rotation(0), 0);
        assert_eq!(normalize_rotation(90), 90);
        assert_eq!(normalize_rotation(180), 180);
        assert_eq!(normalize_rotation(270), 270);
        assert_eq!(normalize_rotation(-90), 270);
        assert_eq!(normalize_rotation(450), 90);
    }

    #[test]
    fn normalization_matrix_maps_crop_corners() {
        let geometry = PageGeometry {
            crop: [10.0, 20.0, 110.0, 220.0],
            rotation: 0,
            user_unit: 1.0,
            resources: None,
            group: None,
        };
        let matrix = geometry.normalization_matrix();
        let (width, height) = geometry.visual_size();
        assert!((width - 100.0).abs() < 1e-9);
        assert!((height - 200.0).abs() < 1e-9);
        let apply = |x: f64, y: f64| {
            (
                matrix[0] * x + matrix[2] * y + matrix[4],
                matrix[1] * x + matrix[3] * y + matrix[5],
            )
        };
        assert!((apply(10.0, 20.0).0).abs() < 1e-9);
        assert!((apply(10.0, 20.0).1).abs() < 1e-9);
        assert!((apply(110.0, 220.0).0 - width).abs() < 1e-9);
        assert!((apply(110.0, 220.0).1 - height).abs() < 1e-9);

        let rotated = PageGeometry {
            crop: [0.0, 0.0, 100.0, 200.0],
            rotation: 90,
            user_unit: 1.0,
            resources: None,
            group: None,
        };
        let matrix = rotated.normalization_matrix();
        let (width, height) = rotated.visual_size();
        assert!((width - 200.0).abs() < 1e-9);
        assert!((height - 100.0).abs() < 1e-9);
        let apply = |x: f64, y: f64| {
            (
                matrix[0] * x + matrix[2] * y + matrix[4],
                matrix[1] * x + matrix[3] * y + matrix[5],
            )
        };
        assert!((apply(0.0, 0.0).0).abs() < 1e-9);
        assert!((apply(0.0, 0.0).1 - 100.0).abs() < 1e-9);
        assert!((apply(100.0, 200.0).0 - 200.0).abs() < 1e-9);
        assert!((apply(100.0, 200.0).1).abs() < 1e-9);
    }

    #[test]
    fn placement_matrices_rotate_around_page_rect() {
        let rect = crate::layout::RectPt::new(10.0, 20.0, 110.0, 220.0);
        let matrix = placement_matrix(0.5, rect, 0);
        let apply = |x: f64, y: f64| {
            (
                matrix[0] * x + matrix[2] * y + matrix[4],
                matrix[1] * x + matrix[3] * y + matrix[5],
            )
        };
        assert!((apply(0.0, 0.0).0 - 10.0).abs() < 1e-9);
        assert!((apply(0.0, 0.0).1 - 20.0).abs() < 1e-9);

        let matrix = placement_matrix(0.5, rect, 180);
        let apply = |x: f64, y: f64| {
            (
                matrix[0] * x + matrix[2] * y + matrix[4],
                matrix[1] * x + matrix[3] * y + matrix[5],
            )
        };
        assert!((apply(0.0, 0.0).0 - 110.0).abs() < 1e-9);
        assert!((apply(0.0, 0.0).1 - 220.0).abs() < 1e-9);
        assert!((apply(200.0, 400.0).0 - 10.0).abs() < 1e-9);
        assert!((apply(200.0, 400.0).1 - 20.0).abs() < 1e-9);
    }

    #[test]
    fn pdf_string_escaping() {
        assert_eq!(escape_pdf_string("Sig 01 / (A)"), "Sig 01 / \\(A\\)");
    }

    #[test]
    fn page_set_filters_sides() {
        assert!(PageSet::Both.includes(Side::Front));
        assert!(PageSet::Both.includes(Side::Back));
        assert!(PageSet::Fronts.includes(Side::Front));
        assert!(!PageSet::Fronts.includes(Side::Back));
        assert!(PageSet::Backs.includes(Side::Back));
        assert!(!PageSet::Backs.includes(Side::Front));
    }
}
