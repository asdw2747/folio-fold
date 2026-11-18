#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use eframe::egui;
use folio_fold::app::{show_error_dialog, FolioFoldApp};
use folio_fold::icon;

fn main() {
    install_panic_hook();

    let startup_paths: Vec<PathBuf> = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .filter(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
        })
        .collect();

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_title("FolioFold")
            .with_app_id("folio-fold")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1180.0, 760.0])
            .with_icon(icon::load_icon()),
        ..Default::default()
    };

    let result = eframe::run_native(
        "FolioFold",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(FolioFoldApp::new(cc, &startup_paths)) as Box<dyn eframe::App>)
        }),
    );

    if let Err(error) = result {
        let message = format!(
            "FolioFold could not start the graphical interface.\n\n{error}\n\n\
             On Linux, check that an X11/OpenGL session is available. On Windows, \
             verify the executable was built with the included static-CRT configuration."
        );
        eprintln!("{message}");
        show_error_dialog("FolioFold — Startup Error", &message);
        std::process::exit(1);
    }
}

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        default_hook(panic_info);
        let message = format!("FolioFold encountered an unexpected error:\n\n{panic_info}");
        let _ = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("FolioFold — Unexpected Error")
            .set_description(&message)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }));
}
