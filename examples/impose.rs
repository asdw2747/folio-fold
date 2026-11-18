//! Small command-line harness for exercising the same engine the GUI uses:
//!
//! ```text
//! cargo run --example impose -- sample.pdf imposed.pdf booklet short
//! ```

use std::path::PathBuf;

use folio_fold::impose::{BindingMode, DuplexFlip};
use folio_fold::layout::LayoutSettings;
use folio_fold::pdf::{export_imposed_pdf, ExportSettings};

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.len() < 2 {
        eprintln!("usage: impose <source.pdf> <output.pdf> [booklet|signatures|quarto|octavo] [short|long]");
        std::process::exit(2);
    }
    let source = PathBuf::from(&arguments[0]);
    let output = PathBuf::from(&arguments[1]);
    let mode = match arguments.get(2).map(String::as_str).unwrap_or("booklet") {
        "signatures" | "signature" => BindingMode::Signatures,
        "quarto" => BindingMode::Quarto,
        "octavo" => BindingMode::Octavo,
        _ => BindingMode::Booklet,
    };
    let duplex = match arguments.get(3).map(String::as_str).unwrap_or("short") {
        "long" => DuplexFlip::LongEdge,
        _ => DuplexFlip::ShortEdge,
    };
    let settings = ExportSettings {
        mode,
        signature_pages: 16,
        duplex,
        layout: LayoutSettings::default(),
        separate_front_back: false,
        source_fingerprint: None,
    };
    match export_imposed_pdf(&source, &output, &settings) {
        Ok(report) => {
            println!(
                "Wrote {}: {} sheets, {} PDF pages, {} bytes",
                report.combined_path.display(),
                report.sheets,
                report.output_pages,
                report.bytes
            );
            for warning in report.warnings {
                println!("warning: {warning}");
            }
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}
