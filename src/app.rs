use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::Duration;

use eframe::egui;

use crate::impose::{build_plan, BindingMode, DuplexFlip, ImpositionPlan, Side};
use crate::layout::{LayoutSettings, Orientation, PaperPreset, ScaleMode, Unit};
use crate::pdf::{export_imposed_pdf_with_cancel, ExportReport, SourceInfo};
use crate::theme;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub mode: BindingMode,
    pub signature_pages: u32,
    pub signature_custom: bool,
    pub duplex: DuplexFlip,
    pub paper: PaperPreset,
    pub orientation: Orientation,
    pub custom_width_mm: f64,
    pub custom_height_mm: f64,
    pub scale: ScaleMode,
    pub outer_margin_mm: f64,
    pub gutter_mm: f64,
    pub creep_mm: f64,
    pub fold_lines: bool,
    pub crop_marks: bool,
    pub sequence_labels: bool,
    pub unit: Unit,
    pub separate_front_back: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: BindingMode::Booklet,
            signature_pages: 16,
            signature_custom: false,
            duplex: DuplexFlip::ShortEdge,
            paper: PaperPreset::Letter,
            orientation: Orientation::Landscape,
            custom_width_mm: 210.0,
            custom_height_mm: 297.0,
            scale: ScaleMode::Fit,
            outer_margin_mm: 10.0,
            gutter_mm: 3.0,
            creep_mm: 0.0,
            fold_lines: true,
            crop_marks: true,
            sequence_labels: true,
            unit: Unit::Millimeters,
            separate_front_back: false,
        }
    }
}

impl Settings {
    pub fn layout_settings(&self) -> LayoutSettings {
        LayoutSettings {
            paper: self.paper,
            orientation: self.orientation,
            custom_width_mm: self.custom_width_mm,
            custom_height_mm: self.custom_height_mm,
            scale: self.scale,
            outer_margin_mm: self.outer_margin_mm,
            gutter_mm: self.gutter_mm,
            creep_mm: self.creep_mm,
            fold_lines: self.fold_lines,
            crop_marks: self.crop_marks,
            sequence_labels: self.sequence_labels,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SideView {
    #[default]
    Both,
    Front,
    Back,
}

impl SideView {
    pub const ALL: [Self; 3] = [Self::Both, Self::Front, Self::Back];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Both => "Both",
            Self::Front => "Front",
            Self::Back => "Back",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreviewSelection {
    pub side: Side,
    pub slot: usize,
}

pub struct FolioFoldApp {
    ctx: egui::Context,
    pub icon_texture: egui::TextureHandle,
    pub source: Option<SourceInfo>,
    pub source_sizes: Vec<(f64, f64)>,
    pub settings: Settings,
    pub plan: ImpositionPlan,
    pub selected_sheet: usize,
    pub side_view: SideView,
    pub selection: Option<PreviewSelection>,
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub status: String,
    pub logs: VecDeque<String>,
    pub loading: bool,
    pub exporting: bool,
    pub last_export: Option<ExportReport>,
    export_cancel: Arc<AtomicBool>,
    load_rx: Option<Receiver<Result<SourceInfo, String>>>,
    export_rx: Option<Receiver<Result<ExportReport, String>>>,
}

impl FolioFoldApp {
    pub fn new(cc: &eframe::CreationContext<'_>, startup_paths: &[PathBuf]) -> Self {
        theme::apply(&cc.egui_ctx);
        let settings = Settings::default();
        let icon = crate::icon::load_icon();
        let icon_texture = cc.egui_ctx.load_texture(
            "foliofold-app-icon",
            egui::ColorImage::from_rgba_unmultiplied(
                [icon.width as usize, icon.height as usize],
                &icon.rgba,
            ),
            egui::TextureOptions::LINEAR,
        );
        let mut app = Self {
            ctx: cc.egui_ctx.clone(),
            icon_texture,
            source: None,
            source_sizes: Vec::new(),
            plan: build_plan(settings.mode, settings.signature_pages, 0, settings.duplex),
            settings,
            selected_sheet: 0,
            side_view: SideView::Both,
            selection: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            status: "Drop a PDF anywhere, or open one to begin.".to_string(),
            logs: VecDeque::new(),
            loading: false,
            exporting: false,
            last_export: None,
            export_cancel: Arc::new(AtomicBool::new(false)),
            load_rx: None,
            export_rx: None,
        };
        app.push_log(format!("FolioFold {} ready", env!("CARGO_PKG_VERSION")));
        if let Some(path) = startup_paths.first() {
            app.start_load(path.clone());
        }
        app
    }

    pub fn can_export(&self) -> bool {
        self.source.is_some() && !self.exporting && !self.loading
    }

    pub fn current_sheet(&self) -> Option<&crate::impose::PhysicalSheet> {
        self.plan.sheets.get(self.selected_sheet)
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    pub fn push_log(&mut self, message: impl Into<String>) {
        self.logs.push_back(message.into());
        while self.logs.len() > 200 {
            self.logs.pop_front();
        }
    }

    pub fn set_mode(&mut self, mode: BindingMode) {
        if self.settings.mode == mode {
            return;
        }
        self.settings.mode = mode;
        self.settings.orientation = if mode.prefers_landscape() {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        };
        self.selected_sheet = 0;
        self.selection = None;
        self.rebuild_plan();
        self.set_status(format!(
            "{} · {} pages per folded section",
            mode.label(),
            self.plan.signature_pages
        ));
    }

    pub fn set_duplex(&mut self, duplex: DuplexFlip) {
        if self.settings.duplex == duplex {
            return;
        }
        self.settings.duplex = duplex;
        self.rebuild_plan();
    }

    pub fn rebuild_plan(&mut self) {
        let pages = self
            .source
            .as_ref()
            .map(|source| source.page_count() as u32)
            .unwrap_or(0);
        self.plan = build_plan(
            self.settings.mode,
            self.settings.signature_pages,
            pages,
            self.settings.duplex,
        );
        if self.plan.sheets.is_empty() {
            self.selected_sheet = 0;
            self.selection = None;
        } else {
            self.selected_sheet = self.selected_sheet.min(self.plan.sheets.len() - 1);
        }
        if self.zoom <= 0.0 {
            self.zoom = 1.0;
        }
    }

    pub fn open_pdf_dialog(&mut self) {
        if self.loading {
            self.set_status("A PDF is already loading.");
            return;
        }
        let mut dialog = rfd::FileDialog::new()
            .add_filter("PDF documents", &["pdf"])
            .set_title("Open a PDF to impose");
        if let Some(source) = &self.source {
            if let Some(parent) = source.path.parent() {
                dialog = dialog.set_directory(parent);
            }
        }
        if let Some(path) = dialog.pick_file() {
            self.start_load(path);
        }
    }

    pub fn start_load(&mut self, path: PathBuf) {
        if self.loading {
            self.set_status("A PDF is already loading.");
            return;
        }
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("pdf"))
        {
            let message = format!("{} is not a PDF file.", path.display());
            self.set_status(message.clone());
            self.push_log(message.clone());
            show_error_dialog("FolioFold — Unsupported File", &message);
            return;
        }
        let (tx, rx) = mpsc::channel();
        let context = self.ctx.clone();
        let thread_path = path.clone();
        std::thread::spawn(move || {
            let result = SourceInfo::load(&thread_path).map_err(|error| format!("{error:#}"));
            let _ = tx.send(result);
            context.request_repaint();
        });
        self.load_rx = Some(rx);
        self.loading = true;
        self.last_export = None;
        self.set_status(format!("Reading {}…", path.display()));
        self.push_log(format!("Reading {}", path.display()));
    }

    pub fn clear_source(&mut self) {
        self.source = None;
        self.source_sizes.clear();
        self.selection = None;
        self.selected_sheet = 0;
        self.zoom = 1.0;
        self.pan = egui::Vec2::ZERO;
        self.last_export = None;
        self.rebuild_plan();
        self.set_status("Source cleared. Drop a PDF to begin again.");
    }

    pub fn start_export(&mut self) {
        let Some(source) = &self.source else {
            self.set_status("Open a PDF before exporting.");
            return;
        };
        if self.exporting || self.loading {
            return;
        }
        let stem = source
            .path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| "folio-fold".to_string());
        let mut dialog = rfd::FileDialog::new()
            .add_filter("Imposed PDF", &["pdf"])
            .set_file_name(format!("{stem}-imposed.pdf"))
            .set_title("Export imposed PDF");
        if let Some(parent) = source.path.parent() {
            dialog = dialog.set_directory(parent);
        }
        let Some(path) = dialog.save_file() else {
            return;
        };
        let path = if path.extension().is_none() {
            path.with_extension("pdf")
        } else {
            path
        };
        if path == source.path {
            show_error_dialog(
                "FolioFold — Export",
                "Choose an output name different from the source PDF.",
            );
            return;
        }
        self.start_export_to(path);
    }

    pub fn cancel_export(&mut self) {
        if self.exporting {
            self.export_cancel.store(true, Ordering::Relaxed);
            self.set_status("Cancelling export after the current sheet…");
            self.push_log("Export cancellation requested.");
        }
    }

    pub fn start_export_to(&mut self, path: PathBuf) {
        if !self.can_export() {
            self.set_status("Open a PDF before exporting.");
            return;
        }
        let Some(source) = self.source.clone() else {
            return;
        };
        let export_settings = crate::pdf::ExportSettings {
            mode: self.settings.mode,
            signature_pages: self.settings.signature_pages,
            duplex: self.settings.duplex,
            layout: self.settings.layout_settings(),
            separate_front_back: self.settings.separate_front_back,
            source_fingerprint: self
                .source
                .as_ref()
                .and_then(|source| source.fingerprint.clone()),
        };
        self.export_cancel.store(false, Ordering::Relaxed);
        let cancel = Arc::clone(&self.export_cancel);
        let (tx, rx) = mpsc::channel();
        let context = self.ctx.clone();
        let output = path.clone();
        std::thread::spawn(move || {
            let result =
                export_imposed_pdf_with_cancel(&source.path, &output, &export_settings, cancel)
                    .map_err(|error| format!("{error:#}"));
            let _ = tx.send(result);
            context.request_repaint();
        });
        self.export_rx = Some(rx);
        self.exporting = true;
        self.set_status(format!(
            "Building {} physical sheets…",
            self.plan.sheets.len()
        ));
        self.push_log(format!(
            "Exporting {} sheet(s) to {}",
            self.plan.sheets.len(),
            path.display()
        ));
    }

    fn poll_workers(&mut self) {
        let load_result = self.load_rx.as_ref().map(Receiver::try_recv);
        match load_result {
            Some(Ok(Ok(source))) => {
                self.load_rx = None;
                self.loading = false;
                self.source_sizes = source.page_sizes();
                let page_count = source.page_count();
                let title = source.title.clone();
                let version = source.pdf_version.clone();
                self.source = Some(source);
                self.selected_sheet = 0;
                self.selection = None;
                self.zoom = 1.0;
                self.pan = egui::Vec2::ZERO;
                self.rebuild_plan();
                self.set_status(format!(
                    "Loaded {title} · {page_count} pages · {} physical sheets",
                    self.plan.sheets.len()
                ));
                self.push_log(format!(
                    "Loaded {title} ({page_count} pages, PDF {version})"
                ));
            }
            Some(Ok(Err(error))) => {
                self.load_rx = None;
                self.loading = false;
                self.set_status(format!("Could not load PDF: {error}"));
                self.push_log(format!("Load failed: {error}"));
                show_error_dialog("FolioFold — PDF Error", &error);
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.load_rx = None;
                self.loading = false;
                self.set_status("The PDF loader stopped unexpectedly.");
            }
            Some(Err(TryRecvError::Empty)) | None => {}
        }

        let export_result = self.export_rx.as_ref().map(Receiver::try_recv);
        match export_result {
            Some(Ok(Ok(report))) => {
                self.export_rx = None;
                self.exporting = false;
                let warnings = report.warnings.clone();
                self.set_status(format!(
                    "Exported {} sheets ({} PDF pages) to {}",
                    report.sheets,
                    report.output_pages,
                    report.combined_path.display()
                ));
                self.push_log(format!(
                    "Wrote {} ({} bytes) in {} ms",
                    report.combined_path.display(),
                    report.bytes,
                    report.duration_ms
                ));
                if let Some(fronts) = &report.fronts_path {
                    self.push_log(format!("Wrote {}", fronts.display()));
                }
                if let Some(backs) = &report.backs_path {
                    self.push_log(format!("Wrote {}", backs.display()));
                }
                for warning in warnings {
                    self.push_log(warning);
                }
                self.last_export = Some(report);
            }
            Some(Ok(Err(error))) => {
                self.export_rx = None;
                self.exporting = false;
                if error.contains("Export cancelled") {
                    self.set_status("Export cancelled. No partially written output file was kept.");
                    self.push_log("Export cancelled.");
                } else {
                    self.set_status(format!("Export failed: {error}"));
                    self.push_log(format!("Export failed: {error}"));
                    show_error_dialog("FolioFold — Export Error", &error);
                }
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.export_rx = None;
                self.exporting = false;
                self.set_status("The export worker stopped unexpectedly.");
            }
            Some(Err(TryRecvError::Empty)) | None => {}
        }
    }

    fn handle_os_drop(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .map(|file| file.path().to_path_buf())
                .collect()
        });
        if dropped.is_empty() {
            return;
        }
        if dropped.len() > 1 {
            self.push_log(format!(
                "Dropped {} files; FolioFold imposes one PDF at a time.",
                dropped.len()
            ));
        }
        if let Some(path) = dropped.into_iter().next() {
            self.start_load(path);
        }
    }
}

impl eframe::App for FolioFoldApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_workers();
        self.handle_os_drop(&ctx);

        let before = self.settings.clone();
        egui::Panel::top("folio-fold-header")
            .exact_size(64.0)
            .show(ui, |ui| crate::ui::header(self, ui));
        egui::Panel::bottom("folio-fold-status")
            .exact_size(30.0)
            .show(ui, |ui| crate::ui::status_bar(self, ui));
        egui::Panel::left("folio-fold-controls")
            .resizable(false)
            .exact_size(296.0)
            .show(ui, |ui| crate::ui::left_panel(self, ui));
        if self.settings != before {
            self.rebuild_plan();
        }
        egui::Panel::right("folio-fold-summary")
            .resizable(false)
            .exact_size(304.0)
            .show(ui, |ui| crate::ui::right_panel(self, ui));
        let before_preview = self.settings.clone();
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::same(10)),
            )
            .show(ui, |ui| crate::ui::preview_panel(self, ui));
        if self.settings != before_preview {
            self.rebuild_plan();
            ctx.request_repaint();
        }

        crate::ui::drag_overlay(&ctx);
        if self.loading || self.exporting {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}

pub fn show_error_dialog(title: &str, message: &str) {
    let _ = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(title)
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}
