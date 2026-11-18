//! Generate a synthetic, vector-only sample PDF for manual previewing:
//!
//! ```text
//! cargo run --example make_sample -- sample.pdf 24
//! ```

use std::path::PathBuf;

use lopdf::{dictionary, Dictionary, Document, Object, Stream};

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let path = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("folio-fold-sample.pdf"));
    let pages = arguments
        .next()
        .and_then(|value| value.to_string_lossy().parse::<u32>().ok())
        .unwrap_or(24)
        .max(1);

    let mut document = Document::with_version("1.4");
    let pages_id = document.new_object_id();
    let font_id = document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica-Bold",
    });
    let body_font = document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let mut kids = Vec::new();
    for page in 1..=pages {
        let content = format!(
            "0.96 0.94 0.89 rg 0 0 612 792 re f\n\
             0.10 0.14 0.19 RG 1.5 w 48 48 516 696 re S\n\
             0.18 0.20 0.24 rg BT /F1 86 Tf 72 610 Td ({page}) Tj ET\n\
             0.10 0.14 0.19 rg BT /F1 26 Tf 72 550 Td (FolioFold sample page {page}) Tj ET\n\
             0.30 0.34 0.38 rg BT /F2 12 Tf 72 510 Td (Vector text, embedded fonts and page content survive imposition.) Tj ET\n\
             0.30 0.34 0.38 rg BT /F2 12 Tf 72 490 Td (Use this file to test booklet, signatures, quarto and octavo modes.) Tj ET\n\
             0.16 0.55 0.55 RG 5 w 72 380 m 540 380 l S\n\
             0.90 0.30 0.20 rg 72 190 468 150 re f\n\
             0.20 0.40 0.75 rg 100 210 m 512 210 l 512 320 l f\n\
             0.10 0.14 0.19 rg BT /F2 10 Tf 72 90 Td (Source page {page} of {pages}) Tj ET\n"
        );
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
            "Resources" => dictionary! {
                "Font" => dictionary! {
                    "F1" => Object::Reference(font_id),
                    "F2" => Object::Reference(body_font),
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
    let info_id = document.add_object(dictionary! {
        "Title" => Object::string_literal("FolioFold Sample Book"),
        "Producer" => Object::string_literal("FolioFold sample generator"),
    });
    document.trailer.set("Info", Object::Reference(info_id));
    document.save(&path).expect("write sample PDF");
    println!("Wrote {} ({} pages)", path.display(), pages);
}
