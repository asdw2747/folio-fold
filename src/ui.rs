pub mod panels;
pub mod preview;

pub use panels::{left_panel, right_panel};
pub use preview::preview_panel;

use egui::{Align, Color32, FontId, Frame, Layout, Margin, RichText, Stroke, StrokeKind, Vec2};

use crate::app::FolioFoldApp;
use crate::theme;

pub fn header(app: &mut FolioFoldApp, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.add(
            egui::Image::new(&app.icon_texture)
                .fit_to_exact_size(Vec2::splat(36.0))
                .corner_radius(8),
        );

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("FolioFold")
                        .strong()
                        .size(23.0)
                        .color(theme::TEXT),
                );
                badge(
                    ui,
                    env!("CARGO_PKG_VERSION"),
                    theme::ACCENT_DIM,
                    theme::ACCENT,
                );
                badge(ui, "UNLICENSE", Color32::from_rgb(34, 45, 62), theme::MUTED);
            });
            ui.label(
                RichText::new("PDF imposition and signature layout for physical bookbinding")
                    .size(9.5)
                    .color(theme::FAINT),
            );
        });

        ui.add_space(8.0);
        let pages = app
            .source
            .as_ref()
            .map(|source| source.page_count())
            .unwrap_or(0);
        header_stat(ui, "PAGES", &pages.to_string());
        header_stat(ui, "SHEETS", &app.plan.sheets.len().to_string());
        header_stat(ui, "BLANKS", &app.plan.blank_pages.to_string());

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let export_text = if app.exporting {
                "Exporting…"
            } else {
                "Export Imposed PDF"
            };
            let export_button = egui::Button::new(RichText::new(export_text).strong().size(12.0))
                .fill(if app.can_export() {
                    theme::ACCENT_DIM
                } else {
                    Color32::from_rgb(28, 38, 48)
                })
                .min_size(Vec2::new(156.0, 32.0));
            if ui
                .add_enabled(app.can_export(), export_button)
                .on_hover_text("Write the imposed sheet sequence as a print-ready PDF.")
                .clicked()
            {
                app.start_export();
            }

            let open = egui::Button::new(RichText::new("Open PDF").size(12.0))
                .min_size(Vec2::new(94.0, 32.0));
            if ui
                .add_enabled(!app.loading, open)
                .on_hover_text("Choose a PDF to impose.")
                .clicked()
            {
                app.open_pdf_dialog();
            }

            Frame::new()
                .fill(Color32::from_rgba_unmultiplied(77, 208, 200, 18))
                .corner_radius(8)
                .inner_margin(Margin::symmetric(8, 5))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.painter().circle_filled(
                            ui.cursor().min + Vec2::new(4.0, 8.0),
                            3.0,
                            theme::ACCENT,
                        );
                        ui.add_space(9.0);
                        ui.label(
                            RichText::new("PURE RUST PDF")
                                .size(9.0)
                                .color(theme::ACCENT)
                                .strong(),
                        );
                    });
                });
        });
    });
}

fn header_stat(ui: &mut egui::Ui, label: &str, value: &str) {
    Frame::new()
        .fill(Color32::from_rgb(17, 24, 35))
        .corner_radius(7)
        .inner_margin(Margin::symmetric(8, 3))
        .stroke(Stroke::new(1.0, theme::BORDER_FAINT))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(label).size(7.5).color(theme::FAINT));
                ui.label(RichText::new(value).size(12.0).color(theme::TEXT).strong());
            });
        });
}

pub fn status_bar(app: &mut FolioFoldApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add_space(3.0);
        if app.loading || app.exporting {
            ui.add(egui::Spinner::new().size(12.0).color(theme::ACCENT));
        } else {
            ui.painter().circle_filled(
                ui.cursor().min + Vec2::new(3.0, 10.0),
                3.5,
                status_color(app),
            );
            ui.add_space(10.0);
        }
        ui.label(RichText::new(&app.status).size(9.5).color(theme::MUTED));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (width, height) = app.settings.layout_settings().sheet_size_mm();
            let unit = app.settings.unit;
            ui.label(
                RichText::new(format!(
                    "{} · {} × {} {} · {:?}",
                    app.settings.mode.short_label(),
                    unit.format_value(width),
                    unit.format_value(height),
                    unit.label(),
                    app.settings.duplex.label()
                ))
                .size(8.5)
                .color(theme::FAINT),
            );
            ui.separator();
            ui.label(
                RichText::new(format!("{} source pages", app.plan.source_pages))
                    .size(8.5)
                    .color(theme::FAINT),
            );
        });
    });
}

fn status_color(app: &FolioFoldApp) -> Color32 {
    if app.status.starts_with("Could not") || app.status.starts_with("Export failed") {
        theme::ERROR
    } else if app.status.starts_with("Loaded") || app.status.starts_with("Exported") {
        theme::SUCCESS
    } else {
        theme::ACCENT
    }
}

pub fn drag_overlay(ctx: &egui::Context) {
    let count = ctx.input(|input| input.raw.hovered_files.len());
    if count == 0 {
        return;
    }
    let rect = ctx.content_rect();
    let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("folio-drop"));
    let painter = ctx.layer_painter(layer);
    painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(5, 8, 13, 230));
    painter.rect_stroke(
        rect.shrink(22.0),
        18,
        Stroke::new(2.0, theme::ACCENT),
        StrokeKind::Inside,
    );
    let center = rect.center();
    painter.circle_filled(center + Vec2::new(0.0, -52.0), 28.0, theme::ACCENT_GLOW);
    painter.rect_stroke(
        egui::Rect::from_center_size(center + Vec2::new(0.0, -52.0), Vec2::splat(30.0)),
        4,
        Stroke::new(2.0, theme::ACCENT),
        StrokeKind::Inside,
    );
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "DROP PDF TO IMPOSE",
        FontId::proportional(20.0),
        theme::TEXT,
    );
    painter.text(
        center + Vec2::new(0.0, 24.0),
        egui::Align2::CENTER_CENTER,
        if count == 1 {
            "Release to read the page tree".to_string()
        } else {
            format!("Release to load the first of {count} files")
        },
        FontId::proportional(11.0),
        theme::MUTED,
    );
}

pub(crate) fn badge(ui: &mut egui::Ui, text: &str, fill: Color32, color: Color32) {
    Frame::new()
        .fill(fill)
        .corner_radius(6)
        .inner_margin(Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(8.5).color(color));
        });
}

pub(crate) fn section_title(ui: &mut egui::Ui, title: &str, trailing: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(theme::letter_spaced(title))
                .strong()
                .size(9.5)
                .color(theme::MUTED),
        );
        if let Some(trailing) = trailing {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(RichText::new(trailing).size(8.5).color(theme::FAINT));
            });
        }
    });
}

pub(crate) fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    Frame::new()
        .fill(theme::CARD)
        .corner_radius(10)
        .inner_margin(Margin::same(10))
        .stroke(Stroke::new(1.0, theme::BORDER))
        .show(ui, add);
}

pub(crate) fn selectable_chip(ui: &mut egui::Ui, selected: bool, label: &str) -> bool {
    ui.add(
        egui::Button::new(RichText::new(label).size(10.0).color(if selected {
            theme::TEXT
        } else {
            theme::MUTED
        }))
        .fill(if selected {
            theme::ACCENT_DIM
        } else {
            Color32::from_rgb(24, 32, 44)
        })
        .stroke(Stroke::new(
            1.0,
            if selected {
                theme::ACCENT
            } else {
                theme::BORDER
            },
        ))
        .corner_radius(7)
        .min_size(Vec2::new(0.0, 26.0)),
    )
    .clicked()
}
