use std::ops::RangeInclusive;

use egui::{
    Align, Button, Color32, ComboBox, Frame, Layout, Margin, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, Ui, Vec2,
};

use crate::app::{FolioFoldApp, PreviewSelection};
use crate::impose::{
    leaves_from_pages, pages_from_leaves, sheets_from_pages, BindingMode, DuplexFlip,
    SIGNATURE_LEAVES, SIGNATURE_SIZES,
};
use crate::layout::{
    exact_pages_overflow, layout_sheet, Orientation, PaperPreset, ScaleMode, Unit, PT_PER_MM,
};
use crate::theme;

use super::{badge, card, section_title, selectable_chip};

pub fn left_panel(app: &mut FolioFoldApp, ui: &mut Ui) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(6.0);
            document_card(app, ui);
            ui.add_space(8.0);
            binding_card(app, ui);
            ui.add_space(8.0);
            paper_card(app, ui);
            ui.add_space(8.0);
            layout_card(app, ui);
            ui.add_space(10.0);
        });
}

fn document_card(app: &mut FolioFoldApp, ui: &mut Ui) {
    section_title(ui, "Source document", None);
    let details = app.source.as_ref().map(|source| {
        (
            source.title.clone(),
            source.path.clone(),
            source.page_count(),
            source.pdf_version.clone(),
            source.size_summary(app.settings.unit),
        )
    });
    let Some((title, path, pages, version, size)) = details else {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 96.0), Sense::click());
        let painter = ui.painter();
        painter.rect_filled(rect, 10, Color32::from_rgb(17, 23, 34));
        painter.rect_stroke(
            rect,
            10,
            Stroke::new(1.0, theme::BORDER),
            StrokeKind::Inside,
        );
        painter.text(
            rect.center() + Vec2::new(0.0, -14.0),
            egui::Align2::CENTER_CENTER,
            "Drop a PDF anywhere",
            egui::FontId::proportional(13.0),
            theme::TEXT,
        );
        painter.text(
            rect.center() + Vec2::new(0.0, 8.0),
            egui::Align2::CENTER_CENTER,
            "or click to choose a file",
            egui::FontId::proportional(9.5),
            theme::MUTED,
        );
        painter.circle_filled(rect.center() + Vec2::new(0.0, 30.0), 2.5, theme::ACCENT);
        if response
            .on_hover_text("PDFs are read natively; no external tools are used.")
            .clicked()
        {
            app.open_pdf_dialog();
        }
        return;
    };

    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("PDF").strong().size(9.0).color(theme::ACCENT));
            ui.label(
                RichText::new(theme::short_name(&title, 26))
                    .strong()
                    .size(11.5)
                    .color(theme::TEXT),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                badge(
                    ui,
                    &format!("{pages} p"),
                    Color32::from_rgb(26, 34, 47),
                    theme::MUTED,
                );
            });
        });
        ui.label(
            RichText::new(theme::short_name(&path.display().to_string(), 42))
                .size(8.5)
                .color(theme::FAINT),
        );
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            status_row(ui, "SIZE", &size);
            status_row(ui, "PDF", &version);
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [ui.available_width() * 0.5, 25.0],
                    Button::new(RichText::new("Replace").size(10.0)),
                )
                .clicked()
            {
                app.open_pdf_dialog();
            }
            if ui
                .add_sized(
                    [ui.available_width(), 25.0],
                    Button::new(RichText::new("Clear").size(10.0)),
                )
                .clicked()
            {
                app.clear_source();
            }
        });
    });
}

fn status_row(ui: &mut Ui, label: &str, value: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(7.5).color(theme::FAINT));
        ui.label(RichText::new(value).size(9.0).color(theme::MUTED));
    });
}

fn binding_card(app: &mut FolioFoldApp, ui: &mut Ui) {
    section_title(ui, "Binding & imposition", None);
    card(ui, |ui| {
        ui.label(RichText::new("MODE").strong().size(8.0).color(theme::FAINT));
        ui.horizontal(|ui| {
            for mode in [BindingMode::Booklet, BindingMode::Signatures] {
                let width = (ui.available_width() - 6.0) * 0.5;
                if ui
                    .add_sized(
                        [width, 28.0],
                        Button::new(RichText::new(mode.label()).size(10.0))
                            .fill(if app.settings.mode == mode {
                                theme::ACCENT_DIM
                            } else {
                                Color32::from_rgb(24, 32, 44)
                            })
                            .stroke(Stroke::new(
                                1.0,
                                if app.settings.mode == mode {
                                    theme::ACCENT
                                } else {
                                    theme::BORDER
                                },
                            )),
                    )
                    .on_hover_text(mode.description())
                    .clicked()
                {
                    app.set_mode(mode);
                }
            }
        });
        ui.horizontal(|ui| {
            for mode in [BindingMode::Quarto, BindingMode::Octavo] {
                let width = (ui.available_width() - 6.0) * 0.5;
                if ui
                    .add_sized(
                        [width, 28.0],
                        Button::new(RichText::new(mode.label()).size(10.0))
                            .fill(if app.settings.mode == mode {
                                theme::ACCENT_DIM
                            } else {
                                Color32::from_rgb(24, 32, 44)
                            })
                            .stroke(Stroke::new(
                                1.0,
                                if app.settings.mode == mode {
                                    theme::ACCENT
                                } else {
                                    theme::BORDER
                                },
                            )),
                    )
                    .on_hover_text(mode.description())
                    .clicked()
                {
                    app.set_mode(mode);
                }
            }
        });

        ui.add_space(4.0);
        ui.label(
            RichText::new(if app.settings.mode == BindingMode::Signatures {
                "LEAVES / SIGNATURE"
            } else {
                "FOLDED SECTION"
            })
            .strong()
            .size(8.0)
            .color(theme::FAINT),
        );
        if app.settings.mode == BindingMode::Signatures {
            let current_pages = app.settings.signature_pages;
            let current_leaves = leaves_from_pages(current_pages);
            let custom = app.settings.signature_custom || !SIGNATURE_SIZES.contains(&current_pages);
            let mut selected_pages = current_pages;
            let mut selected_custom = custom;
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("signature-size")
                    .selected_text(if custom {
                        format!("Custom · {current_leaves} leaves")
                    } else {
                        format!("{current_leaves} leaves · {current_pages} pages")
                    })
                    .width(168.0)
                    .show_ui(ui, |ui| {
                        for (leaves, pages) in SIGNATURE_LEAVES.iter().zip(SIGNATURE_SIZES.iter()) {
                            let selected = !custom && current_pages == *pages;
                            if ui
                                .selectable_label(
                                    selected,
                                    format!("{leaves} leaves · {pages} pages"),
                                )
                                .clicked()
                            {
                                selected_pages = *pages;
                                selected_custom = false;
                            }
                        }
                        if ui.selectable_label(custom, "Custom…").clicked() {
                            selected_custom = true;
                        }
                    });
            });
            if selected_pages != app.settings.signature_pages || selected_custom != custom {
                app.settings.signature_pages = selected_pages;
                app.settings.signature_custom = selected_custom;
            }
            if selected_custom || app.settings.signature_custom {
                let mut leaves = leaves_from_pages(app.settings.signature_pages).max(2);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Custom leaves").size(9.5).color(theme::MUTED));
                    if ui
                        .add(
                            egui::DragValue::new(&mut leaves)
                                .range(2..=32)
                                .speed(1.0)
                                .suffix(" leaves"),
                        )
                        .on_hover_text("Even leaf counts only: 2–32 leaves (4–64 pages).")
                        .changed()
                    {
                        let even_leaves = (leaves / 2 * 2).max(2);
                        app.settings.signature_pages = pages_from_leaves(even_leaves);
                        app.settings.signature_custom = true;
                    }
                });
            }
            let pages = app.settings.signature_pages;
            ui.label(
                RichText::new(format!(
                    "{} pages · {} sheets per signature",
                    pages,
                    sheets_from_pages(pages)
                ))
                .size(8.5)
                .color(theme::ACCENT),
            );
        } else {
            let pages = app.plan.signature_pages;
            let label = if app.settings.mode == BindingMode::Booklet {
                format!(
                    "Single signature · {} pages · {} leaves",
                    pages,
                    leaves_from_pages(pages)
                )
            } else {
                format!(
                    "Fixed {} pages · {} leaves per folded sheet",
                    pages,
                    leaves_from_pages(pages)
                )
            };
            ui.label(RichText::new(label).size(9.0).color(theme::TEXT));
            if app.settings.mode == BindingMode::Booklet {
                ui.label(
                    RichText::new(
                        "Switch to Signatures for multiple independently imposed blocks.",
                    )
                    .size(8.0)
                    .color(theme::FAINT),
                );
            }
        }

        ui.add_space(6.0);
        ui.label(
            RichText::new("DUPLEX FLIP")
                .strong()
                .size(8.0)
                .color(theme::FAINT),
        );
        ui.horizontal(|ui| {
            for flip in DuplexFlip::ALL {
                let selected = app.settings.duplex == flip;
                let width = (ui.available_width() - 6.0) * 0.5;
                let label = format!("{}\n{}", flip.label(), flip.detail());
                if ui
                    .add_sized(
                        [width, 39.0],
                        Button::new(RichText::new(label).size(9.0))
                            .fill(if selected {
                                theme::CARD_ACTIVE
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
                            )),
                    )
                    .on_hover_text(flip.helper())
                    .clicked()
                {
                    app.set_duplex(flip);
                }
            }
        });
        ui.add_space(3.0);
        ui.label(
            RichText::new(app.settings.duplex.helper())
                .size(8.5)
                .color(theme::FAINT),
        );
        if !app.plan.warnings.is_empty() {
            ui.add_space(4.0);
            ui.label(
                RichText::new(&app.plan.warnings[0])
                    .size(8.5)
                    .color(theme::WARNING),
            );
        }
    });
}

fn paper_card(app: &mut FolioFoldApp, ui: &mut Ui) {
    section_title(ui, "Paper stock", None);
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Preset").size(9.5).color(theme::MUTED));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ComboBox::from_id_salt("paper-preset")
                    .selected_text(app.settings.paper.label())
                    .width(142.0)
                    .show_ui(ui, |ui| {
                        for preset in PaperPreset::ALL {
                            ui.selectable_value(
                                &mut app.settings.paper,
                                preset,
                                format!("{} · {}", preset.label(), preset.detail()),
                            );
                        }
                    });
            });
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Orientation").size(9.5).color(theme::MUTED));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                for orientation in [Orientation::Portrait, Orientation::Landscape] {
                    if selectable_chip(
                        ui,
                        app.settings.orientation == orientation,
                        orientation.short_label(),
                    ) {
                        app.settings.orientation = orientation;
                    }
                }
            });
        });

        if app.settings.paper.is_custom() {
            let unit = app.settings.unit;
            ui.horizontal(|ui| {
                ui.label(RichText::new("Width").size(9.5).color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    length_drag(
                        ui,
                        &mut app.settings.custom_width_mm,
                        unit,
                        10.0..=2000.0,
                        0,
                    );
                });
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Height").size(9.5).color(theme::MUTED));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    length_drag(
                        ui,
                        &mut app.settings.custom_height_mm,
                        unit,
                        10.0..=2000.0,
                        0,
                    );
                });
            });
        }

        ui.horizontal(|ui| {
            ui.label(RichText::new("Sheet").size(8.5).color(theme::FAINT));
            ui.label(
                RichText::new(
                    app.settings
                        .layout_settings()
                        .sheet_label(app.settings.unit),
                )
                .size(9.0)
                .color(theme::TEXT),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                for unit in Unit::ALL {
                    if selectable_chip(ui, app.settings.unit == unit, unit.label()) {
                        app.settings.unit = unit;
                    }
                }
            });
        });
    });
}

fn layout_card(app: &mut FolioFoldApp, ui: &mut Ui) {
    section_title(ui, "Layout & marks", None);
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Scaling").size(9.5).color(theme::MUTED));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                for mode in [ScaleMode::Exact, ScaleMode::Fit] {
                    if selectable_chip(ui, app.settings.scale == mode, mode.label()) {
                        app.settings.scale = mode;
                    }
                }
            });
        });
        ui.add_space(3.0);
        length_slider_row(
            ui,
            "Outer margin",
            &mut app.settings.outer_margin_mm,
            app.settings.unit,
            0.0..=30.0,
            1,
        );
        length_slider_row(
            ui,
            "Spine gutter",
            &mut app.settings.gutter_mm,
            app.settings.unit,
            0.0..=30.0,
            1,
        );
        length_slider_row(
            ui,
            "Creep / sheet",
            &mut app.settings.creep_mm,
            app.settings.unit,
            0.0..=0.5,
            2,
        );

        let sheets_per_signature = if app.settings.mode.is_two_up() {
            (app.plan.signature_pages / 4).max(1)
        } else {
            1
        };
        let total_creep = app.settings.creep_mm * sheets_per_signature.saturating_sub(1) as f64;
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(if app.settings.mode.is_two_up() {
                    format!(
                        "Inner-sheet shift {} · max {}",
                        app.settings.unit.format(app.settings.creep_mm),
                        app.settings.unit.format(total_creep)
                    )
                } else {
                    "Creep applies to nested 2-up signatures".to_string()
                })
                .size(8.0)
                .color(theme::FAINT),
            );
        });
        if app.settings.scale == ScaleMode::Exact
            && exact_pages_overflow(
                app.settings.mode,
                &app.plan,
                &app.settings.layout_settings(),
                &app.source_sizes,
            )
        {
            ui.label(
                RichText::new("100% exact pages overflow at least one slot — use Fit to Slot.")
                    .size(8.5)
                    .color(theme::WARNING),
            );
        }

        ui.add_space(5.0);
        ui.separator();
        ui.add_space(3.0);
        ui.label(
            RichText::new("BINDER & PRINTER MARKS")
                .strong()
                .size(8.0)
                .color(theme::FAINT),
        );
        mark_toggle(ui, &mut app.settings.fold_lines, "Center-fold dashed lines");
        mark_toggle(ui, &mut app.settings.crop_marks, "Corner crop / trim marks");
        mark_toggle(
            ui,
            &mut app.settings.sequence_labels,
            "Signature & sheet sequence labels",
        );
    });
}

fn mark_toggle(ui: &mut Ui, value: &mut bool, label: &str) {
    let mut changed = *value;
    ui.horizontal(|ui| {
        let response = ui.checkbox(&mut changed, "");
        ui.label(RichText::new(label).size(9.5).color(if changed {
            theme::TEXT
        } else {
            theme::MUTED
        }));
        if response.changed() {
            *value = changed;
        }
    });
}

fn length_slider_row(
    ui: &mut Ui,
    label: &str,
    value_mm: &mut f64,
    unit: Unit,
    range_mm: RangeInclusive<f64>,
    decimals: usize,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(9.5).color(theme::MUTED));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let decimals = if unit == Unit::Inches { 3 } else { decimals };
            let mut value = unit.from_mm(*value_mm);
            let range = unit.from_mm(*range_mm.start())..=unit.from_mm(*range_mm.end());
            let response = ui.add(
                egui::Slider::new(&mut value, range)
                    .fixed_decimals(decimals)
                    .suffix(format!(" {}", unit.label())),
            );
            if response.changed() {
                *value_mm = unit.to_mm(value);
            }
        });
    });
}

fn length_drag(
    ui: &mut Ui,
    value_mm: &mut f64,
    unit: Unit,
    range_mm: RangeInclusive<f64>,
    decimals: usize,
) {
    let mut value = unit.from_mm(*value_mm);
    let range = unit.from_mm(*range_mm.start())..=unit.from_mm(*range_mm.end());
    let decimals = if unit == Unit::Inches { 2 } else { decimals };
    let response = ui.add(
        egui::DragValue::new(&mut value)
            .range(range)
            .speed(unit.step())
            .fixed_decimals(decimals)
            .suffix(format!(" {}", unit.label())),
    );
    if response.changed() {
        *value_mm = unit.to_mm(value);
    }
}

pub fn right_panel(app: &mut FolioFoldApp, ui: &mut Ui) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(6.0);
            summary_card(app, ui);
            ui.add_space(8.0);
            inspector_card(app, ui);
            ui.add_space(8.0);
            export_card(app, ui);
            ui.add_space(8.0);
            legend_card(ui);
            ui.add_space(8.0);
            log_card(app, ui);
            ui.add_space(10.0);
        });
}

fn summary_card(app: &FolioFoldApp, ui: &mut Ui) {
    section_title(ui, "Imposition summary", None);
    ui.columns(2, |columns| {
        metric_tile(
            &mut columns[0],
            "PHYSICAL SHEETS",
            &app.plan.sheets.len().to_string(),
            theme::ACCENT,
        );
        metric_tile(
            &mut columns[1],
            "PDF PAGES",
            &app.plan.output_pdf_pages().to_string(),
            theme::TEXT,
        );
    });
    ui.columns(2, |columns| {
        metric_tile(
            &mut columns[0],
            "SIGNATURES",
            &app.plan.signatures.to_string(),
            theme::VIOLET,
        );
        metric_tile(
            &mut columns[1],
            "PADDED BLANKS",
            &app.plan.blank_pages.to_string(),
            theme::WARNING,
        );
    });
}

fn metric_tile(ui: &mut Ui, label: &str, value: &str, accent: Color32) {
    let width = ui.available_width().max(60.0);
    Frame::new()
        .fill(theme::CARD)
        .corner_radius(9)
        .inner_margin(Margin::symmetric(10, 8))
        .stroke(Stroke::new(1.0, theme::BORDER))
        .show(ui, |ui| {
            ui.set_width((width - 22.0).max(40.0));
            ui.label(RichText::new(label).size(7.5).color(theme::FAINT));
            ui.label(RichText::new(value).size(20.0).color(accent).strong());
        });
}

fn inspector_card(app: &mut FolioFoldApp, ui: &mut Ui) {
    section_title(ui, "Sheet inspector", None);
    let Some(sheet) = app.current_sheet().cloned() else {
        card(ui, |ui| {
            ui.label(
                RichText::new("Load a PDF to calculate signatures, sheets, and blank paddings.")
                    .size(9.5)
                    .color(theme::MUTED),
            );
        });
        return;
    };

    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "SIG {:02}/{:02}",
                    sheet.signature,
                    app.plan.signatures.max(sheet.signature)
                ))
                .strong()
                .size(10.5)
                .color(theme::ACCENT),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                badge(
                    ui,
                    &format!(
                        "SHEET {:02}/{:02}",
                        sheet.sheet_in_signature, sheet.sheets_in_signature
                    ),
                    Color32::from_rgb(26, 34, 47),
                    theme::MUTED,
                );
            });
        });
        ui.add_space(3.0);
        side_line(ui, "FRONT", &sheet.front);
        side_line(ui, "BACK", &sheet.back);
        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);
        if let Some(selection) = app.selection {
            selected_page_details(app, ui, &sheet, selection);
        } else {
            ui.label(
                RichText::new("Click a page in the preview to inspect its trim and scale.")
                    .size(8.5)
                    .color(theme::FAINT),
            );
        }
    });
}

fn side_line(ui: &mut Ui, label: &str, placements: &[crate::impose::PagePlacement]) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(label).size(8.0).color(theme::FAINT));
        for placement in placements {
            let text = match placement.page {
                Some(page) => format!("{page}"),
                None => "blank".to_string(),
            };
            let rotation = if placement.rotation % 360 == 180 {
                " · 180°"
            } else {
                ""
            };
            let color = match placement.page {
                Some(_) if placement.rotation % 360 == 180 => theme::VIOLET,
                Some(_) => theme::TEXT,
                None => theme::FAINT,
            };
            Frame::new()
                .fill(Color32::from_rgb(25, 33, 46))
                .corner_radius(5)
                .inner_margin(Margin::symmetric(5, 2))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("{text}{rotation}"))
                            .size(8.5)
                            .color(color),
                    );
                });
        }
    });
}

fn selected_page_details(
    app: &FolioFoldApp,
    ui: &mut Ui,
    sheet: &crate::impose::PhysicalSheet,
    selection: PreviewSelection,
) {
    let geometry = layout_sheet(
        app.settings.mode,
        sheet,
        selection.side,
        &app.settings.layout_settings(),
        &app.source_sizes,
    );
    let Some(placement) = geometry.placements.get(selection.slot) else {
        return;
    };
    ui.label(
        RichText::new(format!(
            "{} · slot {}",
            match placement.page {
                Some(page) => format!("Page {page}"),
                None => "Blank flyleaf".to_string(),
            },
            selection.slot + 1
        ))
        .strong()
        .size(10.0)
        .color(theme::TEXT),
    );
    let unit = app.settings.unit;
    ui.label(
        RichText::new(format!(
            "Trim {} × {} · scale {:.1}%",
            unit.format(placement.page_rect.width() / PT_PER_MM),
            unit.format(placement.page_rect.height() / PT_PER_MM),
            placement.scale * 100.0
        ))
        .size(8.5)
        .color(theme::MUTED),
    );
    if placement.rotation % 360 == 180 {
        ui.label(
            RichText::new("Rotated 180° for this fold / duplex side")
                .size(8.5)
                .color(theme::VIOLET),
        );
    }
    if placement.creep_shift_pt.abs() > 0.001 {
        ui.label(
            RichText::new(format!(
                "Creep shift {} toward the spine",
                unit.format(placement.creep_shift_pt.abs() / PT_PER_MM)
            ))
            .size(8.5)
            .color(theme::ACCENT),
        );
    }
}

fn export_card(app: &mut FolioFoldApp, ui: &mut Ui) {
    section_title(ui, "Output", None);
    card(ui, |ui| {
        ui.checkbox(
            &mut app.settings.separate_front_back,
            RichText::new("Export separate front/back PDFs").size(9.5),
        )
        .on_hover_text(
            "Also write <name>-fronts.pdf and <name>-backs.pdf for manual single-sided printing.",
        );
        ui.add_space(3.0);
        ui.label(
            RichText::new("Combined front/back sheet sequence is always written.")
                .size(8.0)
                .color(theme::FAINT),
        );
        ui.add_space(6.0);
        let button = Button::new(
            RichText::new(if app.exporting {
                "Building sheets…"
            } else {
                "Export Imposed PDF"
            })
            .strong()
            .size(11.5),
        )
        .fill(if app.can_export() {
            theme::ACCENT_DIM
        } else {
            Color32::from_rgb(28, 38, 48)
        })
        .min_size(Vec2::new(ui.available_width(), 34.0));
        if ui.add_enabled(app.can_export(), button).clicked() {
            app.start_export();
        }
        if app.exporting
            && ui
                .add_sized(
                    [ui.available_width(), 27.0],
                    Button::new(RichText::new("Cancel export").size(10.0))
                        .fill(Color32::from_rgb(52, 30, 36))
                        .stroke(Stroke::new(1.0, theme::ERROR)),
                )
                .clicked()
        {
            app.cancel_export();
        }
        if let Some(report) = &app.last_export {
            ui.add_space(5.0);
            ui.label(
                RichText::new(format!(
                    "Wrote {} · {} sheets · {:.1} KB",
                    theme::short_name(
                        report
                            .combined_path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .unwrap_or_default()
                            .as_str(),
                        28
                    ),
                    report.sheets,
                    report.bytes as f64 / 1024.0
                ))
                .size(8.5)
                .color(theme::SUCCESS),
            );
            if let Some(fronts) = &report.fronts_path {
                ui.label(
                    RichText::new(format!(
                        "Fronts: {}",
                        fronts.file_name().unwrap_or_default().to_string_lossy()
                    ))
                    .size(8.0)
                    .color(theme::MUTED),
                );
            }
            if let Some(backs) = &report.backs_path {
                ui.label(
                    RichText::new(format!(
                        "Backs: {}",
                        backs.file_name().unwrap_or_default().to_string_lossy()
                    ))
                    .size(8.0)
                    .color(theme::MUTED),
                );
            }
        }
    });
}

fn legend_card(ui: &mut Ui) {
    section_title(ui, "Legend", None);
    card(ui, |ui| {
        legend_row(ui, theme::FOLD_MARK, "Center-fold guide");
        legend_row(ui, theme::TRIM_MARK, "Crop / trim mark");
        legend_row(ui, theme::VIOLET, "180° page rotation");
        legend_row(ui, theme::ACCENT, "Duplex spine direction");
    });
}

fn legend_row(ui: &mut Ui, color: Color32, label: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(18.0, 8.0), Sense::hover());
        ui.painter().line_segment(
            [
                egui::pos2(rect.left(), rect.center().y),
                egui::pos2(rect.right(), rect.center().y),
            ],
            Stroke::new(2.0, color),
        );
        ui.label(RichText::new(label).size(8.5).color(theme::MUTED));
    });
}

fn log_card(app: &FolioFoldApp, ui: &mut Ui) {
    egui::CollapsingHeader::new(
        RichText::new("Activity log")
            .strong()
            .size(9.5)
            .color(theme::MUTED),
    )
    .id_salt("folio-log")
    .show(ui, |ui| {
        Frame::new()
            .fill(Color32::from_rgb(10, 14, 21))
            .corner_radius(7)
            .inner_margin(Margin::same(8))
            .show(ui, |ui| {
                if app.logs.is_empty() {
                    ui.label(
                        RichText::new("No activity yet.")
                            .size(8.5)
                            .color(theme::FAINT),
                    );
                }
                for line in app.logs.iter().rev().take(12) {
                    ui.label(RichText::new(line).size(8.0).color(theme::FAINT));
                }
            });
    });
}
