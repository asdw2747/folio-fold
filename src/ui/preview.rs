use egui::{
    Align, Color32, FontId, Layout, PointerButton, Pos2, Rect, RichText, Sense, Stroke, StrokeKind,
    Ui, Vec2,
};

use crate::app::{FolioFoldApp, PreviewSelection, SideView};
use crate::impose::{PhysicalSheet, Side};
use crate::layout::{
    layout_sheet, trim_marks, LineSegment, Point, RectPt, SheetGeometry, PT_PER_MM,
};
use crate::theme;

use super::{badge, selectable_chip};

pub fn preview_panel(app: &mut FolioFoldApp, ui: &mut Ui) {
    ui.add_space(2.0);
    preview_header(app, ui);
    ui.add_space(6.0);

    let available = ui.available_size();
    let (canvas, response) = ui.allocate_exact_size(
        available.max(Vec2::new(100.0, 100.0)),
        Sense::click_and_drag(),
    );
    let painter = ui.painter_at(canvas);
    painter.rect_filled(canvas, 12, Color32::from_rgb(11, 15, 23));
    painter.rect_stroke(
        canvas,
        12,
        Stroke::new(1.0, theme::BORDER),
        StrokeKind::Inside,
    );

    if response.hovered() {
        let scroll = ui.input(|input| input.smooth_scroll_delta.y);
        if scroll.abs() > 0.1 {
            app.zoom = (app.zoom * (1.0 + scroll * 0.0015)).clamp(1.0, 4.0);
            if app.zoom <= 1.001 {
                app.pan = Vec2::ZERO;
            }
        }
    }
    if response.dragged_by(PointerButton::Primary) && app.zoom > 1.001 {
        app.pan += response.drag_delta();
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    } else if response.hovered() && app.zoom > 1.001 {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
    }

    let Some(sheet) = app.current_sheet().cloned() else {
        draw_empty_state(app, ui, &painter, canvas, &response);
        return;
    };

    let sides = match app.side_view {
        SideView::Both => vec![Side::Front, Side::Back],
        SideView::Front => vec![Side::Front],
        SideView::Back => vec![Side::Back],
    };
    let settings = app.settings.layout_settings();
    let geometries: Vec<(Side, SheetGeometry)> = sides
        .iter()
        .copied()
        .map(|side| {
            (
                side,
                layout_sheet(
                    app.settings.mode,
                    &sheet,
                    side,
                    &settings,
                    &app.source_sizes,
                ),
            )
        })
        .collect();
    let count = geometries.len() as f32;
    let gap = 28.0;
    let inset = Vec2::new(34.0, 34.0);
    let fit_area = Rect::from_center_size(canvas.center(), canvas.size() - inset * 2.0);
    let paper_width = geometries[0].1.width.max(1.0);
    let paper_height = geometries[0].1.height.max(1.0);
    let fit_scale = ((f64::from(fit_area.width()) - f64::from(gap) * (f64::from(count) - 1.0))
        / (paper_width * f64::from(count)))
    .min(f64::from(fit_area.height()) / paper_height)
    .max(0.01);
    let scale = fit_scale * f64::from(app.zoom);
    let sheet_size = Vec2::new((paper_width * scale) as f32, (paper_height * scale) as f32);
    let total_width = sheet_size.x * count + gap * (count - 1.0);
    let left = canvas.center().x - total_width * 0.5 + app.pan.x;
    let top = canvas.center().y - sheet_size.y * 0.5 + app.pan.y;

    let mut hit_rects = Vec::new();
    for (index, (side, geometry)) in geometries.iter().enumerate() {
        let sheet_rect = Rect::from_min_size(
            Pos2::new(left + index as f32 * (sheet_size.x + gap), top),
            sheet_size,
        );
        let header = format!(
            "SHEET {:02}  ·  {}",
            sheet.number,
            side.label().to_uppercase()
        );
        painter.text(
            sheet_rect.left_top() + Vec2::new(0.0, -19.0),
            egui::Align2::LEFT_TOP,
            header,
            FontId::proportional(9.0),
            theme::TEXT,
        );
        let subheader = if *side == Side::Back {
            format!("{} · {}", app.settings.duplex.detail(), sheet_label(&sheet))
        } else {
            sheet_label(&sheet)
        };
        painter.text(
            sheet_rect.left_top() + Vec2::new(0.0, -8.0),
            egui::Align2::LEFT_TOP,
            subheader,
            FontId::proportional(8.0),
            if *side == Side::Back {
                theme::ACCENT
            } else {
                theme::FAINT
            },
        );
        if *side == Side::Back {
            draw_flip_glyph(
                &painter,
                sheet_rect.right_top() + Vec2::new(-10.0, -14.0),
                app.settings.duplex,
                theme::ACCENT,
            );
        }
        draw_sheet(
            app,
            &painter,
            sheet_rect,
            geometry,
            *side,
            &mut hit_rects,
            response.hover_pos(),
        );
    }

    if response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            app.selection = hit_rects
                .iter()
                .rev()
                .find(|(_, _, rect)| rect.contains(pointer))
                .map(|(side, slot, _)| PreviewSelection {
                    side: *side,
                    slot: *slot,
                });
        }
    }
    if response.double_clicked() {
        app.zoom = 1.0;
        app.pan = Vec2::ZERO;
    }

    draw_busy_overlay(app, &painter, canvas);
}

fn preview_header(app: &mut FolioFoldApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("SHEET PREVIEW")
                .strong()
                .size(11.0)
                .color(theme::TEXT),
        );
        let sheet_count = app.plan.sheets.len();
        badge(
            ui,
            &format!(
                "{} / {}",
                if sheet_count == 0 {
                    0
                } else {
                    app.selected_sheet + 1
                },
                sheet_count
            ),
            Color32::from_rgb(26, 34, 47),
            theme::MUTED,
        );
        ui.label(
            RichText::new(format!(
                "{} · {}",
                app.settings.mode.label(),
                app.settings.paper.label()
            ))
            .size(8.5)
            .color(theme::FAINT),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let next = ui.add_enabled(
                app.selected_sheet + 1 < app.plan.sheets.len(),
                egui::Button::new(RichText::new("Next Sheet ›").size(9.5))
                    .min_size(Vec2::new(88.0, 25.0)),
            );
            if next.clicked() {
                app.selected_sheet += 1;
                app.selection = None;
            }
            let prev = ui.add_enabled(
                app.selected_sheet > 0,
                egui::Button::new(RichText::new("‹ Prev Sheet").size(9.5))
                    .min_size(Vec2::new(88.0, 25.0)),
            );
            if prev.clicked() {
                app.selected_sheet -= 1;
                app.selection = None;
            }
            if app.plan.signatures > 1 {
                let mut signature = app
                    .current_sheet()
                    .map(|sheet| sheet.signature)
                    .unwrap_or(1);
                egui::ComboBox::from_id_salt("signature-jump")
                    .selected_text(format!("Sig {:02}", signature))
                    .width(84.0)
                    .show_ui(ui, |ui| {
                        for value in 1..=app.plan.signatures {
                            ui.selectable_value(
                                &mut signature,
                                value,
                                format!("Sig {:02} / {:02}", value, app.plan.signatures),
                            );
                        }
                    });
                if signature
                    != app
                        .current_sheet()
                        .map(|sheet| sheet.signature)
                        .unwrap_or(1)
                {
                    if let Some(index) = app
                        .plan
                        .sheets
                        .iter()
                        .position(|sheet| sheet.signature == signature)
                    {
                        app.selected_sheet = index;
                        app.selection = None;
                    }
                }
            }
        });
    });
    ui.horizontal(|ui| {
        for view in SideView::ALL {
            if selectable_chip(ui, app.side_view == view, view.label()) {
                app.side_view = view;
            }
        }
        ui.separator();
        ui.label(RichText::new("DUPLEX").size(8.0).color(theme::FAINT));
        for flip in crate::impose::DuplexFlip::ALL {
            let selected = app.settings.duplex == flip;
            if selectable_chip(ui, selected, flip.label()) {
                app.set_duplex(flip);
            }
        }
    });
    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new("Fit").size(9.0))
                        .min_size(Vec2::new(36.0, 23.0)),
                )
                .on_hover_text("Reset zoom and pan")
                .clicked()
            {
                app.zoom = 1.0;
                app.pan = Vec2::ZERO;
            }
            if ui
                .add(
                    egui::Button::new(RichText::new("+").size(10.0))
                        .min_size(Vec2::new(28.0, 23.0)),
                )
                .clicked()
            {
                app.zoom = (app.zoom * 1.2).clamp(1.0, 4.0);
            }
            ui.label(
                RichText::new(format!("{:.0}%", app.zoom * 100.0))
                    .size(8.5)
                    .color(theme::MUTED),
            );
            if ui
                .add(
                    egui::Button::new(RichText::new("−").size(10.0))
                        .min_size(Vec2::new(28.0, 23.0)),
                )
                .clicked()
            {
                app.zoom = (app.zoom / 1.2).max(1.0);
                if app.zoom <= 1.001 {
                    app.pan = Vec2::ZERO;
                }
            }
        });
    });
}

fn sheet_label(sheet: &PhysicalSheet) -> String {
    format!(
        "Sig {:02} / Sheet {:02} of {:02}",
        sheet.signature, sheet.sheet_in_signature, sheet.sheets_in_signature
    )
}

#[allow(clippy::too_many_arguments)]
fn draw_sheet(
    app: &FolioFoldApp,
    painter: &egui::Painter,
    sheet_rect: Rect,
    geometry: &SheetGeometry,
    side: Side,
    hit_rects: &mut Vec<(Side, usize, Rect)>,
    hover: Option<Pos2>,
) {
    painter.rect_filled(
        sheet_rect.translate(Vec2::new(3.0, 6.0)),
        4,
        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
    );
    painter.rect_filled(sheet_rect, 3, theme::PAPER);
    painter.rect_stroke(
        sheet_rect,
        3,
        Stroke::new(1.0, theme::PAPER_EDGE),
        StrokeKind::Inside,
    );

    let mut page_rects = Vec::new();
    for placement in &geometry.placements {
        let rect = map_rect(
            placement.page_rect,
            sheet_rect,
            geometry.width,
            geometry.height,
        );
        page_rects.push((placement.slot, rect, placement));
    }
    let hovered = hover.and_then(|point| {
        page_rects
            .iter()
            .find(|(_, rect, _)| rect.contains(point))
            .map(|(slot, _, _)| *slot)
    });

    for (slot, rect, placement) in &page_rects {
        let selected = app.selection == Some(PreviewSelection { side, slot: *slot });
        let is_hovered = hovered == Some(*slot);
        if placement.blank {
            painter.rect_filled(*rect, 2, Color32::from_rgb(233, 230, 221));
            hatch(
                painter,
                *rect,
                Color32::from_rgba_unmultiplied(120, 118, 110, 24),
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "BLANK",
                FontId::proportional((rect.width().min(rect.height()) * 0.08).clamp(7.0, 12.0)),
                theme::PAPER_INK_FAINT,
            );
        } else if let Some(page) = placement.page {
            let font_size = (rect.width().min(rect.height()) * 0.24).clamp(12.0, 42.0);
            painter.text(
                rect.center() + Vec2::new(0.0, -3.0),
                egui::Align2::CENTER_CENTER,
                page.to_string(),
                FontId::proportional(font_size),
                theme::PAPER_INK,
            );
            painter.text(
                rect.center() + Vec2::new(0.0, font_size * 0.65),
                egui::Align2::CENTER_CENTER,
                "PAGE",
                FontId::proportional((font_size * 0.24).clamp(6.0, 9.0)),
                theme::PAPER_INK_FAINT,
            );
            if placement.rotation % 360 == 180 {
                draw_orientation_marker(
                    painter,
                    rect.center_top() + Vec2::new(0.0, 8.0),
                    false,
                    theme::VIOLET,
                );
                let badge_rect = Rect::from_min_size(
                    rect.right_top() + Vec2::new(-38.0, 7.0),
                    Vec2::new(31.0, 13.0),
                );
                painter.rect_filled(badge_rect, 4, theme::VIOLET);
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "180°",
                    FontId::proportional(7.5),
                    Color32::WHITE,
                );
            } else {
                draw_orientation_marker(
                    painter,
                    rect.center_top() + Vec2::new(0.0, 6.0),
                    true,
                    theme::PAPER_INK_FAINT,
                );
            }
        }

        if placement.rotation % 360 == 180 || placement.blank {
            painter.rect_stroke(
                *rect,
                2,
                Stroke::new(0.8, theme::PAPER_EDGE),
                StrokeKind::Inside,
            );
        }

        if is_hovered || selected {
            painter.rect_stroke(
                *rect,
                2,
                Stroke::new(
                    if selected { 2.0 } else { 1.0 },
                    if selected {
                        theme::ACCENT
                    } else {
                        Color32::from_rgba_unmultiplied(77, 208, 200, 140)
                    },
                ),
                StrokeKind::Inside,
            );
        }
        hit_rects.push((side, *slot, *rect));
    }

    if app.settings.crop_marks {
        let gap = 2.0 * PT_PER_MM;
        let length = 4.0 * PT_PER_MM;
        for placement in &geometry.placements {
            for mark in trim_marks(placement.page_rect, gap, length) {
                let line = map_line(mark, sheet_rect, geometry.width, geometry.height);
                painter.line_segment([line.0, line.1], Stroke::new(0.7, theme::TRIM_MARK));
            }
        }
    }

    if app.settings.fold_lines {
        for fold in &geometry.fold_lines {
            let (a, b) = map_line(fold.line, sheet_rect, geometry.width, geometry.height);
            dashed_line(
                painter,
                a,
                b,
                if fold.center { 6.0 } else { 5.0 },
                4.0,
                Stroke::new(
                    if fold.center { 1.2 } else { 0.8 },
                    if fold.center {
                        theme::FOLD_MARK
                    } else {
                        Color32::from_rgba_unmultiplied(60, 122, 128, 120)
                    },
                ),
            );
        }
    }

    if app.settings.sequence_labels {
        let point = map_point(
            geometry.label_position,
            sheet_rect,
            geometry.width,
            geometry.height,
        );
        painter.text(
            point,
            egui::Align2::LEFT_CENTER,
            format!(
                "Sig {:02}/{:02}  /  Sheet {:02}/{:02}  /  {}",
                sheet_signature(app, side),
                app.plan.signatures.max(sheet_signature(app, side)),
                app.current_sheet()
                    .map(|sheet| sheet.sheet_in_signature)
                    .unwrap_or(1),
                app.current_sheet()
                    .map(|sheet| sheet.sheets_in_signature)
                    .unwrap_or(1),
                side.short_label()
            ),
            FontId::proportional(5.8),
            theme::PAPER_INK_FAINT,
        );
    }
}

fn sheet_signature(app: &FolioFoldApp, _side: Side) -> u32 {
    app.current_sheet()
        .map(|sheet| sheet.signature)
        .unwrap_or(1)
}

fn draw_empty_state(
    app: &mut FolioFoldApp,
    ui: &egui::Ui,
    painter: &egui::Painter,
    canvas: Rect,
    response: &egui::Response,
) {
    let settings = app.settings.layout_settings();
    let (width, height) = settings.sheet_size_pt();
    let available = canvas.shrink2(Vec2::new(52.0, 44.0));
    let scale =
        ((available.width() as f64 / width).min(available.height() as f64 / height) * 0.9) as f32;
    let sheet_rect = Rect::from_center_size(
        canvas.center(),
        Vec2::new(
            (width * f64::from(scale)) as f32,
            (height * f64::from(scale)) as f32,
        ),
    );
    painter.rect_filled(
        sheet_rect.translate(Vec2::new(3.0, 6.0)),
        4,
        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
    );
    painter.rect_filled(sheet_rect, 3, Color32::from_rgb(233, 230, 221));
    painter.rect_stroke(
        sheet_rect,
        3,
        Stroke::new(1.0, theme::PAPER_EDGE),
        StrokeKind::Inside,
    );
    let (rows, cols) = match app.settings.mode {
        crate::impose::BindingMode::Booklet | crate::impose::BindingMode::Signatures => (1, 2),
        crate::impose::BindingMode::Quarto => (2, 2),
        crate::impose::BindingMode::Octavo => (2, 4),
    };
    for row in 0..rows {
        for col in 0..cols {
            let rect = Rect::from_min_max(
                Pos2::new(
                    sheet_rect.left() + sheet_rect.width() * col as f32 / cols as f32 + 10.0,
                    sheet_rect.top() + sheet_rect.height() * row as f32 / rows as f32 + 10.0,
                ),
                Pos2::new(
                    sheet_rect.left() + sheet_rect.width() * (col + 1) as f32 / cols as f32 - 10.0,
                    sheet_rect.top() + sheet_rect.height() * (row + 1) as f32 / rows as f32 - 10.0,
                ),
            );
            painter.rect_stroke(
                rect,
                2,
                Stroke::new(0.8, Color32::from_rgba_unmultiplied(120, 118, 110, 80)),
                StrokeKind::Inside,
            );
        }
    }
    painter.text(
        sheet_rect.center() + Vec2::new(0.0, -8.0),
        egui::Align2::CENTER_CENTER,
        "DROP A PDF TO BEGIN",
        FontId::proportional(18.0),
        theme::PAPER_INK,
    );
    painter.text(
        sheet_rect.center() + Vec2::new(0.0, 16.0),
        egui::Align2::CENTER_CENTER,
        "Vector pages become folded sheets, not screenshots",
        FontId::proportional(9.0),
        theme::PAPER_INK_FAINT,
    );
    if response.clicked() {
        app.open_pdf_dialog();
    }
    draw_busy_overlay(app, painter, canvas);
    let _ = ui;
}

fn draw_busy_overlay(app: &FolioFoldApp, painter: &egui::Painter, canvas: Rect) {
    if !(app.loading || app.exporting) {
        return;
    }
    let rect = Rect::from_center_size(canvas.center(), Vec2::new(290.0, 74.0));
    painter.rect_filled(rect, 10, Color32::from_rgba_unmultiplied(9, 13, 20, 235));
    painter.rect_stroke(
        rect,
        10,
        Stroke::new(1.0, theme::ACCENT_DIM),
        StrokeKind::Inside,
    );
    painter.text(
        rect.center() + Vec2::new(0.0, -9.0),
        egui::Align2::CENTER_CENTER,
        if app.loading {
            "Reading page tree…"
        } else {
            "Building physical sheets…"
        },
        FontId::proportional(12.0),
        theme::TEXT,
    );
    painter.text(
        rect.center() + Vec2::new(0.0, 15.0),
        egui::Align2::CENTER_CENTER,
        "The preview updates when the worker finishes",
        FontId::proportional(8.5),
        theme::FAINT,
    );
}

fn draw_flip_glyph(
    painter: &egui::Painter,
    center: Pos2,
    flip: crate::impose::DuplexFlip,
    color: Color32,
) {
    let sheet = Rect::from_center_size(center, Vec2::new(11.0, 14.0));
    painter.rect_stroke(sheet, 2, Stroke::new(0.9, color), StrokeKind::Inside);
    let stroke = Stroke::new(1.0, color);
    match flip {
        crate::impose::DuplexFlip::ShortEdge => {
            painter.line_segment(
                [
                    Pos2::new(sheet.left() - 5.0, center.y),
                    Pos2::new(sheet.right() + 5.0, center.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(sheet.left() - 5.0, center.y),
                    Pos2::new(sheet.left() - 1.0, center.y - 3.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(sheet.left() - 5.0, center.y),
                    Pos2::new(sheet.left() - 1.0, center.y + 3.0),
                ],
                stroke,
            );
        }
        crate::impose::DuplexFlip::LongEdge => {
            painter.line_segment(
                [
                    Pos2::new(center.x, sheet.top() - 5.0),
                    Pos2::new(center.x, sheet.bottom() + 5.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x, sheet.top() - 5.0),
                    Pos2::new(center.x - 3.0, sheet.top() - 1.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x, sheet.top() - 5.0),
                    Pos2::new(center.x + 3.0, sheet.top() - 1.0),
                ],
                stroke,
            );
        }
    }
}

fn draw_orientation_marker(painter: &egui::Painter, top_center: Pos2, up: bool, color: Color32) {
    let points = if up {
        vec![
            Pos2::new(top_center.x, top_center.y - 1.0),
            Pos2::new(top_center.x - 4.0, top_center.y + 6.0),
            Pos2::new(top_center.x + 4.0, top_center.y + 6.0),
        ]
    } else {
        vec![
            Pos2::new(top_center.x, top_center.y + 7.0),
            Pos2::new(top_center.x - 4.0, top_center.y),
            Pos2::new(top_center.x + 4.0, top_center.y),
        ]
    };
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

fn map_point(point: Point, sheet_rect: Rect, width: f64, height: f64) -> Pos2 {
    Pos2::new(
        sheet_rect.left() + (point.x / width) as f32 * sheet_rect.width(),
        sheet_rect.bottom() - (point.y / height) as f32 * sheet_rect.height(),
    )
}

fn map_rect(rect: RectPt, sheet_rect: Rect, width: f64, height: f64) -> Rect {
    let min = map_point(Point::new(rect.x0, rect.y0), sheet_rect, width, height);
    let max = map_point(Point::new(rect.x1, rect.y1), sheet_rect, width, height);
    Rect::from_two_pos(min, max)
}

fn map_line(line: LineSegment, sheet_rect: Rect, width: f64, height: f64) -> (Pos2, Pos2) {
    (
        map_point(line.a, sheet_rect, width, height),
        map_point(line.b, sheet_rect, width, height),
    )
}

fn dashed_line(painter: &egui::Painter, a: Pos2, b: Pos2, dash: f32, gap: f32, stroke: Stroke) {
    let vector = b - a;
    let length = vector.length();
    if length <= 0.01 {
        return;
    }
    let direction = vector / length;
    let mut position = 0.0;
    while position < length {
        let end = (position + dash).min(length);
        painter.line_segment([a + direction * position, a + direction * end], stroke);
        position += dash + gap;
    }
}

fn hatch(painter: &egui::Painter, rect: Rect, color: Color32) {
    let clipped = painter.with_clip_rect(rect);
    let step = 9.0;
    let mut offset = -rect.height();
    while offset < rect.width() {
        clipped.line_segment(
            [
                Pos2::new(rect.left() + offset, rect.bottom()),
                Pos2::new(rect.left() + offset + rect.height(), rect.top()),
            ],
            Stroke::new(0.7, color),
        );
        offset += step;
    }
}
