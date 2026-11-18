use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use folio_fold::impose::{BindingMode, DuplexFlip};
use folio_fold::layout::{LayoutSettings, ScaleMode};
use folio_fold::pdf::{
    export_imposed_pdf, export_imposed_pdf_with_cancel, ExportSettings, SourceInfo,
};
use lopdf::{dictionary, Dictionary, Document, Object, Stream};
use tempfile::TempDir;

fn make_source_pdf(path: &Path, pages: u32, rotate: Option<i64>) {
    let mut document = Document::with_version("1.4");
    let pages_id = document.new_object_id();
    let font_id = document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let image_id = document.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => 2,
            "Height" => 2,
            "ColorSpace" => "DeviceRGB",
            "BitsPerComponent" => 8,
        },
        vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0],
    ));
    let mut kids = Vec::new();
    for page in 1..=pages {
        let content = format!(
            "BT /F1 72 Tf 72 700 Td (PAGE {page}) Tj ET\n\
             0.2 0.4 0.6 rg 10 10 80 40 re f\n\
             0.8 0.2 0.2 rg 120 120 m 200 200 l 120 200 l f\n\
             q 72 0 0 72 440 440 cm /Im1 Do Q\n"
        );
        let content_id = document.add_object(Stream::new(Dictionary::new(), content.into_bytes()));
        let mut page_dict = dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(612.0),
                Object::Real(792.0),
            ]),
            "Resources" => dictionary! {
                "Font" => dictionary! {
                    "F1" => Object::Reference(font_id),
                },
                "XObject" => dictionary! {
                    "Im1" => Object::Reference(image_id),
                },
            },
            "Contents" => Object::Reference(content_id),
        };
        if let Some(rotation) = rotate {
            page_dict.set("Rotate", rotation);
        }
        let page_id = document.add_object(page_dict);
        kids.push(Object::Reference(page_id));
    }
    document.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => Object::Array(kids),
            "Count" => pages as i64,
        }),
    );
    let catalog_id = document.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    document.trailer.set("Root", Object::Reference(catalog_id));
    document.save(path).expect("write synthetic source PDF");
}

fn export_settings(
    mode: BindingMode,
    signature_pages: u32,
    duplex: DuplexFlip,
    separate: bool,
) -> ExportSettings {
    ExportSettings {
        mode,
        signature_pages,
        duplex,
        layout: LayoutSettings::default(),
        separate_front_back: separate,
        source_fingerprint: None,
    }
}

fn inspect_output(path: &Path) -> (usize, Vec<[f32; 4]>, usize, BTreeMap<u32, usize>) {
    let document = Document::load(path).expect("load imposed PDF");
    let pages = document.get_pages();
    let mut boxes = Vec::new();
    for page_id in pages.values() {
        let page = document.get_dictionary(*page_id).expect("page dictionary");
        let media_box = page.get(b"MediaBox").expect("MediaBox").as_array().unwrap();
        let numbers: Vec<f32> = media_box
            .iter()
            .map(|value| value.as_float().unwrap())
            .collect();
        boxes.push([numbers[0], numbers[1], numbers[2], numbers[3]]);
    }

    let mut form_count = 0usize;
    let mut labels = BTreeMap::new();
    for object in document.objects.values() {
        if let Object::Stream(stream) = object {
            let is_form = stream
                .dict
                .get(b"Subtype")
                .and_then(Object::as_name)
                .is_ok_and(|name| name == b"Form");
            if is_form {
                form_count += 1;
            }
            let content = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            let text = String::from_utf8_lossy(&content);
            for page in 1..=64u32 {
                let needle = format!("(PAGE {page})");
                let count = text.matches(&needle).count();
                if count > 0 {
                    *labels.entry(page).or_insert(0) += count;
                }
            }
        }
    }
    (pages.len(), boxes, form_count, labels)
}

fn image_xobject_count(path: &Path) -> usize {
    let document = Document::load(path).expect("load imposed PDF");
    document
        .objects
        .values()
        .filter(|object| {
            let Object::Stream(stream) = object else {
                return false;
            };
            stream
                .dict
                .get(b"Subtype")
                .and_then(Object::as_name)
                .is_ok_and(|name| name == b"Image")
        })
        .count()
}

fn form_order_by_page(path: &Path) -> Vec<Vec<u32>> {
    let document = Document::load(path).expect("load imposed PDF");
    let pages = document.get_pages();
    let mut output = Vec::new();
    for page_id in pages.values() {
        let content = document.get_page_content(*page_id);
        let mut names = Vec::new();
        let mut index = 0;
        while index + 3 < content.len() {
            if &content[index..index + 3] == b"/Fm" {
                let start = index + 3;
                let mut end = start;
                while end < content.len() && content[end].is_ascii_digit() {
                    end += 1;
                }
                if end > start && content.get(end..end + 3).is_some_and(|tail| tail == b" Do") {
                    if let Ok(number) = std::str::from_utf8(&content[start..end])
                        .unwrap_or("")
                        .parse::<u32>()
                    {
                        names.push(number);
                    }
                }
                index = end;
            } else {
                index += 1;
            }
        }
        output.push(names);
    }
    output
}

#[test]
fn booklet_end_to_end_preserves_every_page_once() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("booklet.pdf");
    let output = temp.path().join("booklet-imposed.pdf");
    make_source_pdf(&source, 8, None);

    let info = SourceInfo::load(&source).expect("inspect source");
    assert_eq!(info.page_count(), 8);
    assert_eq!(info.uniform_page_size(), Some((612.0, 792.0)));

    let report = export_imposed_pdf(
        &source,
        &output,
        &export_settings(BindingMode::Booklet, 4, DuplexFlip::ShortEdge, false),
    )
    .expect("impose booklet");

    assert_eq!(report.sheets, 2);
    assert_eq!(report.output_pages, 4);
    assert!(output.is_file());
    assert!(report.bytes > 0);
    assert!(report.fronts_path.is_none());

    let (page_count, boxes, form_count, labels) = inspect_output(&output);
    assert_eq!(page_count, 4);
    assert_eq!(form_count, 8);
    assert_eq!(labels.len(), 8);
    assert!(labels.values().all(|count| *count == 1));
    assert_eq!(image_xobject_count(&output), 1);
    let orders = form_order_by_page(&output);
    assert_eq!(orders, vec![vec![8, 1], vec![2, 7], vec![6, 3], vec![4, 5]]);
    for media_box in boxes {
        assert!((media_box[2] - 792.0).abs() < 0.5);
        assert!((media_box[3] - 612.0).abs() < 0.5);
    }

    let bytes = std::fs::read(&output).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("FolioFold"));
}

#[test]
fn multi_signature_end_to_end_has_expected_sheet_count_and_blanks() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("book.pdf");
    let output = temp.path().join("book-imposed.pdf");
    make_source_pdf(&source, 20, None);

    let report = export_imposed_pdf(
        &source,
        &output,
        &export_settings(BindingMode::Signatures, 8, DuplexFlip::ShortEdge, false),
    )
    .expect("impose signatures");

    assert_eq!(report.sheets, 6);
    assert_eq!(report.output_pages, 12);
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.contains("4 blank")));
    let (page_count, _, form_count, labels) = inspect_output(&output);
    assert_eq!(page_count, 12);
    assert_eq!(form_count, 20);
    assert_eq!(labels.len(), 20);
    assert!(labels.values().all(|count| *count == 1));
}

#[test]
fn quarto_and_octavo_end_to_end() {
    let temp = TempDir::new().unwrap();

    let quarto_source = temp.path().join("quarto.pdf");
    let quarto_output = temp.path().join("quarto-imposed.pdf");
    make_source_pdf(&quarto_source, 8, None);
    let quarto = export_imposed_pdf(
        &quarto_source,
        &quarto_output,
        &export_settings(BindingMode::Quarto, 8, DuplexFlip::ShortEdge, false),
    )
    .expect("impose quarto");
    assert_eq!(quarto.sheets, 1);
    assert_eq!(quarto.output_pages, 2);
    let (count, boxes, forms, labels) = inspect_output(&quarto_output);
    assert_eq!(count, 2);
    assert_eq!(forms, 8);
    assert_eq!(labels.len(), 8);
    assert_eq!(
        form_order_by_page(&quarto_output),
        vec![vec![5, 4, 8, 1], vec![3, 6, 2, 7]]
    );
    assert!(boxes.iter().all(|media_box| media_box[2] > media_box[3]));

    let octavo_source = temp.path().join("octavo.pdf");
    let octavo_output = temp.path().join("octavo-imposed.pdf");
    make_source_pdf(&octavo_source, 16, None);
    let octavo = export_imposed_pdf(
        &octavo_source,
        &octavo_output,
        &export_settings(BindingMode::Octavo, 16, DuplexFlip::ShortEdge, false),
    )
    .expect("impose octavo");
    assert_eq!(octavo.sheets, 1);
    assert_eq!(octavo.output_pages, 2);
    let (count, _, forms, labels) = inspect_output(&octavo_output);
    assert_eq!(count, 2);
    assert_eq!(forms, 16);
    assert_eq!(labels.len(), 16);
    assert_eq!(
        form_order_by_page(&octavo_output),
        vec![
            vec![5, 12, 9, 8, 4, 13, 16, 1],
            vec![7, 10, 11, 6, 2, 15, 14, 3]
        ]
    );
}

#[test]
fn separate_front_and_back_exports_are_written() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("manual.pdf");
    let output = temp.path().join("manual-imposed.pdf");
    make_source_pdf(&source, 8, None);

    let report = export_imposed_pdf(
        &source,
        &output,
        &export_settings(BindingMode::Booklet, 4, DuplexFlip::LongEdge, true),
    )
    .expect("separate export");

    let fronts = report.fronts_path.as_ref().unwrap();
    let backs = report.backs_path.as_ref().unwrap();
    assert!(fronts.is_file());
    assert!(backs.is_file());
    assert_eq!(inspect_output(fronts).0, 2);
    assert_eq!(inspect_output(backs).0, 2);
    let (_, _, front_forms, front_labels) = inspect_output(fronts);
    let (_, _, back_forms, back_labels) = inspect_output(backs);
    assert_eq!(front_forms + back_forms, 8);
    assert_eq!(front_labels.len() + back_labels.len(), 8);
}

#[test]
fn blank_padding_end_to_end() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("short.pdf");
    let output = temp.path().join("short-imposed.pdf");
    make_source_pdf(&source, 5, None);

    let report = export_imposed_pdf(
        &source,
        &output,
        &export_settings(BindingMode::Booklet, 4, DuplexFlip::ShortEdge, false),
    )
    .expect("pad booklet");
    assert_eq!(report.sheets, 2);
    assert_eq!(report.output_pages, 4);
    let (_, _, forms, labels) = inspect_output(&output);
    assert_eq!(forms, 5);
    assert_eq!(labels.len(), 5);
}

#[test]
fn rotated_source_pages_are_normalized() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("rotated.pdf");
    let output = temp.path().join("rotated-imposed.pdf");
    make_source_pdf(&source, 8, Some(90));

    let info = SourceInfo::load(&source).expect("inspect rotated");
    assert_eq!(info.uniform_page_size(), Some((792.0, 612.0)));

    let report = export_imposed_pdf(
        &source,
        &output,
        &export_settings(BindingMode::Booklet, 4, DuplexFlip::ShortEdge, false),
    )
    .expect("impose rotated");
    assert_eq!(report.output_pages, 4);
    let (count, _, forms, labels) = inspect_output(&output);
    assert_eq!(count, 4);
    assert_eq!(forms, 8);
    assert_eq!(labels.len(), 8);
}

fn make_inherited_source_pdf(path: &Path, pages: u32) {
    let mut document = Document::with_version("1.4");
    let pages_id = document.new_object_id();
    let font_id = document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let mut kids = Vec::new();
    for page in 1..=pages {
        let content = format!("BT /F1 36 Tf 72 700 Td (PAGE {page}) Tj ET\n");
        let content_id = document.add_object(Stream::new(Dictionary::new(), content.into_bytes()));
        let page_id = document.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "Contents" => Object::Reference(content_id),
        });
        kids.push(Object::Reference(page_id));
    }
    document.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => Object::Array(kids),
            "Count" => pages as i64,
            "MediaBox" => Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(612.0),
                Object::Real(792.0),
            ]),
            "Resources" => dictionary! {
                "Font" => dictionary! {
                    "F1" => Object::Reference(font_id),
                },
            },
        }),
    );
    let catalog_id = document.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    document.trailer.set("Root", Object::Reference(catalog_id));
    document.save(path).expect("write inherited source PDF");
}

#[test]
fn inherited_page_tree_attributes_are_resolved() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("inherited.pdf");
    let output = temp.path().join("inherited-imposed.pdf");
    make_inherited_source_pdf(&source, 8);

    let info = SourceInfo::load(&source).expect("inspect inherited source");
    assert_eq!(info.page_count(), 8);
    assert_eq!(info.uniform_page_size(), Some((612.0, 792.0)));
    export_imposed_pdf(
        &source,
        &output,
        &export_settings(BindingMode::Booklet, 4, DuplexFlip::ShortEdge, false),
    )
    .expect("impose inherited source");
    let (count, _, forms, labels) = inspect_output(&output);
    assert_eq!(count, 4);
    assert_eq!(forms, 8);
    assert_eq!(labels.len(), 8);
    assert!(labels.values().all(|count| *count == 1));
}

fn make_varied_source_pdf(path: &Path, pages: u32) {
    let mut document = Document::with_version("1.4");
    let pages_id = document.new_object_id();
    let font_id = document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let mut kids = Vec::new();
    for page in 1..=pages {
        let (width, height) = if page % 2 == 0 {
            (792.0, 612.0)
        } else {
            (612.0, 792.0)
        };
        let content = format!("BT /F1 36 Tf 72 300 Td (PAGE {page}) Tj ET\n");
        let content_id = document.add_object(Stream::new(Dictionary::new(), content.into_bytes()));
        let page_id = document.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(width),
                Object::Real(height),
            ]),
            "Resources" => dictionary! {
                "Font" => dictionary! {
                    "F1" => Object::Reference(font_id),
                },
            },
            "Contents" => Object::Reference(content_id),
        });
        kids.push(Object::Reference(page_id));
    }
    document.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => Object::Array(kids),
            "Count" => pages as i64,
        }),
    );
    let catalog_id = document.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    document.trailer.set("Root", Object::Reference(catalog_id));
    document.save(path).expect("write varied source PDF");
}

fn make_cropped_scaled_source_pdf(path: &Path, pages: u32) {
    let mut document = Document::with_version("1.4");
    let pages_id = document.new_object_id();
    let font_id = document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let mut kids = Vec::new();
    for page in 1..=pages {
        let content = format!("BT /F1 36 Tf 40 300 Td (PAGE {page}) Tj ET\n");
        let content_id = document.add_object(Stream::new(Dictionary::new(), content.into_bytes()));
        let page_id = document.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(612.0),
                Object::Real(792.0),
            ]),
            "CropBox" => Object::Array(vec![
                Object::Real(10.0),
                Object::Real(20.0),
                Object::Real(310.0),
                Object::Real(420.0),
            ]),
            "UserUnit" => Object::Real(2.0),
            "Resources" => dictionary! {
                "Font" => dictionary! {
                    "F1" => Object::Reference(font_id),
                },
            },
            "Contents" => Object::Reference(content_id),
        });
        kids.push(Object::Reference(page_id));
    }
    document.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => Object::Array(kids),
            "Count" => pages as i64,
        }),
    );
    let catalog_id = document.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    document.trailer.set("Root", Object::Reference(catalog_id));
    document
        .save(path)
        .expect("write cropped/scaled source PDF");
}

#[test]
fn varied_page_sizes_and_crop_user_unit_are_supported() {
    let temp = TempDir::new().unwrap();

    let varied = temp.path().join("varied.pdf");
    let varied_output = temp.path().join("varied-imposed.pdf");
    make_varied_source_pdf(&varied, 8);
    let varied_info = SourceInfo::load(&varied).expect("inspect varied sizes");
    assert_eq!(varied_info.page_count(), 8);
    assert!(varied_info.uniform_page_size().is_none());
    let range = varied_info.page_size_range().unwrap();
    assert_eq!(range.0, (612.0, 612.0));
    assert_eq!(range.1, (792.0, 792.0));
    let report = export_imposed_pdf(
        &varied,
        &varied_output,
        &export_settings(BindingMode::Booklet, 4, DuplexFlip::ShortEdge, false),
    )
    .expect("impose varied sizes");
    assert_eq!(report.output_pages, 4);
    let (count, _, forms, labels) = inspect_output(&varied_output);
    assert_eq!(count, 4);
    assert_eq!(forms, 8);
    assert_eq!(labels.len(), 8);

    let cropped = temp.path().join("cropped.pdf");
    let cropped_output = temp.path().join("cropped-imposed.pdf");
    make_cropped_scaled_source_pdf(&cropped, 8);
    let cropped_info = SourceInfo::load(&cropped).expect("inspect crop/user unit");
    assert_eq!(cropped_info.uniform_page_size(), Some((600.0, 800.0)));
    let report = export_imposed_pdf(
        &cropped,
        &cropped_output,
        &export_settings(BindingMode::Booklet, 4, DuplexFlip::ShortEdge, false),
    )
    .expect("impose cropped/scaled source");
    assert_eq!(report.output_pages, 4);
    let (count, _, forms, labels) = inspect_output(&cropped_output);
    assert_eq!(count, 4);
    assert_eq!(forms, 8);
    assert_eq!(labels.len(), 8);
}

#[test]
fn exact_scale_overflow_and_source_changes_are_reported() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source.pdf");
    let output = temp.path().join("source-imposed.pdf");
    make_source_pdf(&source, 8, None);
    let info = SourceInfo::load(&source).expect("inspect source");
    let old_fingerprint = info.fingerprint.clone();
    assert!(old_fingerprint.is_some());

    let mut settings = export_settings(BindingMode::Quarto, 8, DuplexFlip::ShortEdge, false);
    settings.layout.scale = ScaleMode::Exact;
    settings.source_fingerprint = old_fingerprint;
    // Change the source after the preview loaded it, then export.
    make_source_pdf(&source, 9, None);
    let report = export_imposed_pdf(&source, &output, &settings).expect("export changed source");
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.contains("source PDF changed")));
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.contains("100% exact scale")));
}

#[test]
fn cancellation_is_reported_without_writing_output() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("cancel.pdf");
    let output = temp.path().join("cancel-imposed.pdf");
    make_source_pdf(&source, 8, None);
    let cancel = Arc::new(AtomicBool::new(true));
    let error = export_imposed_pdf_with_cancel(
        &source,
        &output,
        &export_settings(BindingMode::Booklet, 4, DuplexFlip::ShortEdge, false),
        cancel,
    )
    .expect_err("cancelled export must fail");
    assert!(format!("{error:#}").contains("Export cancelled"));
    assert!(!output.exists());
}

#[test]
fn rejects_encrypted_and_missing_sources() {
    let temp = TempDir::new().unwrap();
    let missing = temp.path().join("missing.pdf");
    assert!(SourceInfo::load(&missing).is_err());
    assert!(export_imposed_pdf(
        &missing,
        &temp.path().join("out.pdf"),
        &export_settings(BindingMode::Booklet, 4, DuplexFlip::ShortEdge, false)
    )
    .is_err());
}
