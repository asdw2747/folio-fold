//! Paper, slot, margin, creep, and printer-mark geometry.
//!
//! All coordinates in this module are PDF points with the origin at the
//! bottom-left of the sheet. The preview converts to its screen coordinates at
//! draw time, so the geometry here is the single source of truth for both the
//! preview and the exported PDF.

use crate::impose::{BindingMode, ImpositionPlan, PhysicalSheet, Side};

pub const PT_PER_MM: f64 = 72.0 / 25.4;
pub const MM_PER_INCH: f64 = 25.4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Unit {
    #[default]
    Millimeters,
    Inches,
}

impl Unit {
    pub const ALL: [Self; 2] = [Self::Millimeters, Self::Inches];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Millimeters => "mm",
            Self::Inches => "in",
        }
    }

    pub const fn long_label(self) -> &'static str {
        match self {
            Self::Millimeters => "Millimeters",
            Self::Inches => "Inches",
        }
    }

    pub fn from_mm(self, millimeters: f64) -> f64 {
        match self {
            Self::Millimeters => millimeters,
            Self::Inches => millimeters / MM_PER_INCH,
        }
    }

    pub fn to_mm(self, value: f64) -> f64 {
        match self {
            Self::Millimeters => value,
            Self::Inches => value * MM_PER_INCH,
        }
    }

    pub fn format(self, millimeters: f64) -> String {
        match self {
            Self::Millimeters => format!("{millimeters:.1} mm"),
            Self::Inches => format!("{:.3} in", millimeters / MM_PER_INCH),
        }
    }

    pub fn format_value(self, millimeters: f64) -> String {
        match self {
            Self::Millimeters => format!("{millimeters:.1}"),
            Self::Inches => format!("{:.3}", millimeters / MM_PER_INCH),
        }
    }

    pub fn decimals(self) -> usize {
        match self {
            Self::Millimeters => 1,
            Self::Inches => 3,
        }
    }

    pub fn step(self) -> f64 {
        match self {
            Self::Millimeters => 0.1,
            Self::Inches => 0.01,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaperPreset {
    #[default]
    Letter,
    Legal,
    Tabloid,
    A4,
    A3,
    A5,
    B5,
    Custom,
}

impl PaperPreset {
    pub const ALL: [Self; 8] = [
        Self::Letter,
        Self::Legal,
        Self::Tabloid,
        Self::A4,
        Self::A3,
        Self::A5,
        Self::B5,
        Self::Custom,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Letter => "US Letter",
            Self::Legal => "US Legal",
            Self::Tabloid => "US Tabloid",
            Self::A4 => "A4",
            Self::A3 => "A3",
            Self::A5 => "A5",
            Self::B5 => "B5",
            Self::Custom => "Custom",
        }
    }

    pub const fn detail(self) -> &'static str {
        match self {
            Self::Letter => "8.5 x 11 in",
            Self::Legal => "8.5 x 14 in",
            Self::Tabloid => "11 x 17 in",
            Self::A4 => "210 x 297 mm",
            Self::A3 => "297 x 420 mm",
            Self::A5 => "148 x 210 mm",
            Self::B5 => "176 x 250 mm",
            Self::Custom => "Enter width and height",
        }
    }

    /// Portrait dimensions in millimetres. `None` for custom.
    pub const fn portrait_mm(self) -> Option<(f64, f64)> {
        match self {
            Self::Letter => Some((215.9, 279.4)),
            Self::Legal => Some((215.9, 355.6)),
            Self::Tabloid => Some((279.4, 431.8)),
            Self::A4 => Some((210.0, 297.0)),
            Self::A3 => Some((297.0, 420.0)),
            Self::A5 => Some((148.0, 210.0)),
            Self::B5 => Some((176.0, 250.0)),
            Self::Custom => None,
        }
    }

    pub const fn is_custom(self) -> bool {
        matches!(self, Self::Custom)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Landscape,
    Portrait,
}

impl Orientation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Landscape => "Landscape",
            Self::Portrait => "Portrait",
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Landscape => "Land",
            Self::Portrait => "Port",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScaleMode {
    #[default]
    Fit,
    Exact,
}

impl ScaleMode {
    pub const ALL: [Self; 2] = [Self::Fit, Self::Exact];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Fit => "Fit to slot",
            Self::Exact => "100% exact",
        }
    }

    pub const fn helper(self) -> &'static str {
        match self {
            Self::Fit => "Preserve each source page's aspect ratio inside its slot.",
            Self::Exact => "Place pages at their original PDF size; overflow is not clipped.",
        }
    }
}

/// Page-layout settings shared by the preview and the PDF writer.
#[derive(Clone, Debug)]
pub struct LayoutSettings {
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
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl LayoutSettings {
    /// Sheet dimensions in PDF points, respecting the selected orientation.
    pub fn sheet_size_pt(&self) -> (f64, f64) {
        let (width_mm, height_mm) = match self.paper.portrait_mm() {
            Some((width, height)) => (width, height),
            None => (
                self.custom_width_mm.max(1.0),
                self.custom_height_mm.max(1.0),
            ),
        };
        let landscape = width_mm >= height_mm;
        let want_landscape = self.orientation == Orientation::Landscape;
        let (width_mm, height_mm) = if landscape == want_landscape {
            (width_mm, height_mm)
        } else {
            (height_mm, width_mm)
        };
        (width_mm * PT_PER_MM, height_mm * PT_PER_MM)
    }

    pub fn sheet_size_mm(&self) -> (f64, f64) {
        let (width, height) = self.sheet_size_pt();
        (width / PT_PER_MM, height / PT_PER_MM)
    }

    pub fn sheet_label(&self, unit: Unit) -> String {
        let (width, height) = self.sheet_size_mm();
        format!(
            "{} {} x {} {}",
            self.paper.label(),
            unit.format_value(width),
            unit.format_value(height),
            unit.label()
        )
    }

    pub fn creep_per_sheet_pt(&self) -> f64 {
        self.creep_mm.clamp(0.0, 0.5) * PT_PER_MM
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RectPt {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl RectPt {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    pub fn from_center(center: Point, width: f64, height: f64) -> Self {
        let half_width = width * 0.5;
        let half_height = height * 0.5;
        Self {
            x0: center.x - half_width,
            y0: center.y - half_height,
            x1: center.x + half_width,
            y1: center.y + half_height,
        }
    }

    pub fn width(self) -> f64 {
        self.x1 - self.x0
    }

    pub fn height(self) -> f64 {
        self.y1 - self.y0
    }

    pub fn center(self) -> Point {
        Point::new((self.x0 + self.x1) * 0.5, (self.y0 + self.y1) * 0.5)
    }

    pub fn translate(self, x: f64, y: f64) -> Self {
        Self {
            x0: self.x0 + x,
            y0: self.y0 + y,
            x1: self.x1 + x,
            y1: self.y1 + y,
        }
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.x0 && point.x <= self.x1 && point.y >= self.y0 && point.y <= self.y1
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LineSegment {
    pub a: Point,
    pub b: Point,
}

impl LineSegment {
    pub const fn new(a: Point, b: Point) -> Self {
        Self { a, b }
    }
}

#[derive(Clone, Debug)]
pub struct PlacementGeometry {
    pub slot: usize,
    pub page: Option<u32>,
    pub rotation: u16,
    pub blank: bool,
    pub slot_rect: RectPt,
    /// Actual page/trim rectangle in sheet coordinates. Rotation is applied
    /// around this rectangle's centre.
    pub page_rect: RectPt,
    pub source_size: Option<(f64, f64)>,
    pub scale: f64,
    pub creep_shift_pt: f64,
}

#[derive(Clone, Debug)]
pub struct FoldLine {
    pub line: LineSegment,
    pub center: bool,
}

#[derive(Clone, Debug)]
pub struct SheetGeometry {
    pub width: f64,
    pub height: f64,
    pub rows: usize,
    pub cols: usize,
    pub placements: Vec<PlacementGeometry>,
    pub fold_lines: Vec<FoldLine>,
    pub label_position: Point,
    pub slots: Vec<RectPt>,
}

/// Build the exact geometry for one side of a physical sheet.
///
/// `page_sizes` is indexed by source page number minus one and supplies each
/// page's upright visual size in points. Blank or out-of-range pages use the
/// first source page size as a placeholder so their trim marks still line up.
pub fn layout_sheet(
    mode: BindingMode,
    sheet: &PhysicalSheet,
    side: Side,
    settings: &LayoutSettings,
    page_sizes: &[(f64, f64)],
) -> SheetGeometry {
    let (width, height) = settings.sheet_size_pt();
    let (rows, cols) = match mode {
        BindingMode::Booklet | BindingMode::Signatures => (1, 2),
        BindingMode::Quarto => (2, 2),
        BindingMode::Octavo => (2, 4),
    };

    let outer = (settings.outer_margin_mm.clamp(0.0, 40.0) * PT_PER_MM)
        .min(width * 0.25)
        .min(height * 0.25);
    let gutter = (settings.gutter_mm.clamp(0.0, 40.0) * PT_PER_MM)
        .min(width * 0.1)
        .min(height * 0.1);
    let usable_width = (width - outer * 2.0).max(1.0);
    let usable_height = (height - outer * 2.0).max(1.0);
    let cell_width = usable_width / cols as f64;
    let cell_height = usable_height / rows as f64;

    let mut slots = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        for col in 0..cols {
            let x0_raw = outer + col as f64 * cell_width;
            let x1_raw = x0_raw + cell_width;
            // Slot 0 is the top-left page, matching the row-major matrices in
            // `impose.rs` and the way binders read a sheet. PDF coordinates are
            // y-up, so the top row has the highest y values.
            let y0 = outer + (rows - 1 - row) as f64 * cell_height;
            let y1 = y0 + cell_height;
            let sheet_center = width * 0.5;
            let (x0, x1) = if x1_raw <= sheet_center + 0.01 {
                (x0_raw, (x1_raw - gutter).max(x0_raw + 4.0))
            } else if x0_raw >= sheet_center - 0.01 {
                ((x0_raw + gutter).min(x1_raw - 4.0), x1_raw)
            } else {
                (x0_raw, x1_raw)
            };
            slots.push(RectPt::new(x0, y0, x1, y1));
        }
    }

    let placements = sheet.side(side);
    let nominal = page_sizes
        .first()
        .copied()
        .filter(|(width, height)| *width > 0.0 && *height > 0.0)
        .unwrap_or((595.276, 841.89));

    let creep_depth = if mode.is_two_up() {
        sheet.sheet_in_signature.saturating_sub(1) as f64
    } else {
        0.0
    };
    let creep_shift_pt = creep_depth * settings.creep_per_sheet_pt();

    let mut geometries = Vec::with_capacity(placements.len());
    for placement in placements {
        let slot_rect = slots
            .get(placement.slot)
            .copied()
            .unwrap_or_else(|| slots.last().copied().unwrap_or(RectPt::default()));
        let source_size = placement
            .page
            .and_then(|page| page_sizes.get(page.saturating_sub(1) as usize).copied())
            .filter(|(width, height)| *width > 0.0 && *height > 0.0);
        let (source_width, source_height) = source_size.unwrap_or(nominal);
        let scale = match settings.scale {
            ScaleMode::Fit => (slot_rect.width() / source_width)
                .min(slot_rect.height() / source_height)
                .min(100.0),
            ScaleMode::Exact => 1.0,
        };
        let placed_width = (source_width * scale).max(1.0);
        let placed_height = (source_height * scale).max(1.0);
        let slot_center = slot_rect.center();
        let direction = if slot_center.x < width * 0.5 {
            1.0
        } else if slot_center.x > width * 0.5 {
            -1.0
        } else {
            0.0
        };
        let creep_shift = creep_shift_pt * direction;
        let page_rect = RectPt::from_center(
            Point::new(slot_center.x + creep_shift, slot_center.y),
            placed_width,
            placed_height,
        );

        geometries.push(PlacementGeometry {
            slot: placement.slot,
            page: placement.page,
            rotation: placement.rotation % 360,
            blank: placement.page.is_none(),
            slot_rect,
            page_rect,
            source_size,
            scale,
            creep_shift_pt: creep_shift,
        });
    }

    let mut fold_lines = Vec::new();
    if settings.fold_lines {
        for col in 1..cols {
            let x = width * col as f64 / cols as f64;
            fold_lines.push(FoldLine {
                line: LineSegment::new(Point::new(x, 0.0), Point::new(x, height)),
                center: col * 2 == cols,
            });
        }
        for row in 1..rows {
            let y = height * row as f64 / rows as f64;
            fold_lines.push(FoldLine {
                line: LineSegment::new(Point::new(0.0, y), Point::new(width, y)),
                center: row * 2 == rows,
            });
        }
    }

    SheetGeometry {
        width,
        height,
        rows,
        cols,
        placements: geometries,
        fold_lines,
        label_position: Point::new(
            width * 0.5 + 2.0 * PT_PER_MM,
            (outer * 0.5).max(2.0 * PT_PER_MM),
        ),
        slots,
    }
}

/// Returns true when any real source page is placed at its exact size but does
/// not fit inside its assigned slot. This is used both for the live UI warning
/// and the export report; blank flyleaves are ignored.
pub fn exact_pages_overflow(
    mode: BindingMode,
    plan: &ImpositionPlan,
    settings: &LayoutSettings,
    page_sizes: &[(f64, f64)],
) -> bool {
    if settings.scale != ScaleMode::Exact {
        return false;
    }
    plan.sheets.iter().any(|sheet| {
        [Side::Front, Side::Back].into_iter().any(|side| {
            let geometry = layout_sheet(mode, sheet, side, settings, page_sizes);
            geometry.placements.iter().any(|placement| {
                placement.page.is_some() && !rect_fits(placement.page_rect, placement.slot_rect)
            })
        })
    })
}

fn rect_fits(inner: RectPt, outer: RectPt) -> bool {
    const EPSILON: f64 = 0.01;
    inner.x0 >= outer.x0 - EPSILON
        && inner.y0 >= outer.y0 - EPSILON
        && inner.x1 <= outer.x1 + EPSILON
        && inner.y1 <= outer.y1 + EPSILON
}

/// Eight hairline segments that form corner trim marks around a page.
pub fn trim_marks(rect: RectPt, gap_pt: f64, length_pt: f64) -> Vec<LineSegment> {
    let gap = gap_pt.max(0.5);
    let length = length_pt.max(0.5);
    let mut marks = Vec::with_capacity(8);
    let corners = [
        (rect.x0, rect.y0, -1.0, -1.0),
        (rect.x1, rect.y0, 1.0, -1.0),
        (rect.x0, rect.y1, -1.0, 1.0),
        (rect.x1, rect.y1, 1.0, 1.0),
    ];
    for (x, y, sx, sy) in corners {
        let x_edge = x + sx * gap;
        let y_edge = y + sy * gap;
        marks.push(LineSegment::new(
            Point::new(x_edge, y_edge),
            Point::new(x_edge + sx * length, y_edge),
        ));
        marks.push(LineSegment::new(
            Point::new(x_edge, y_edge),
            Point::new(x_edge, y_edge + sy * length),
        ));
    }
    marks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impose::{build_plan, DuplexFlip};

    fn sizes(count: u32, width: f64, height: f64) -> Vec<(f64, f64)> {
        vec![(width, height); count as usize]
    }

    #[test]
    fn paper_presets_and_orientation() {
        let mut settings = LayoutSettings::default();
        let (width, height) = settings.sheet_size_pt();
        assert!(width > height);
        assert!((width - 792.0).abs() < 0.1);
        assert!((height - 612.0).abs() < 0.1);

        settings.paper = PaperPreset::A4;
        settings.orientation = Orientation::Portrait;
        let (width, height) = settings.sheet_size_pt();
        assert!((width - 595.276).abs() < 0.2);
        assert!((height - 841.89).abs() < 0.2);

        settings.paper = PaperPreset::Custom;
        settings.custom_width_mm = 200.0;
        settings.custom_height_mm = 300.0;
        settings.orientation = Orientation::Landscape;
        let (width, height) = settings.sheet_size_pt();
        assert!(width > height);
        assert!((width - 300.0 * PT_PER_MM).abs() < 0.01);
    }

    #[test]
    fn unit_conversion_round_trips() {
        assert!((Unit::Inches.to_mm(1.0) - 25.4).abs() < 1e-12);
        assert!((Unit::Inches.from_mm(25.4) - 1.0).abs() < 1e-12);
        assert_eq!(Unit::Millimeters.format_value(10.0), "10.0");
    }

    #[test]
    fn booklet_slots_are_centered_and_gutter_clear() {
        let plan = build_plan(BindingMode::Booklet, 4, 8, DuplexFlip::ShortEdge);
        let settings = LayoutSettings::default();
        let geometry = layout_sheet(
            BindingMode::Booklet,
            &plan.sheets[0],
            Side::Front,
            &settings,
            &sizes(8, 612.0, 792.0),
        );
        assert_eq!(geometry.slots.len(), 2);
        assert!(geometry.slots[0].x1 < geometry.slots[1].x0);
        let gap = geometry.slots[1].x0 - geometry.slots[0].x1;
        assert!((gap - settings.gutter_mm * PT_PER_MM * 2.0).abs() < 0.01);
        assert_eq!(geometry.fold_lines.len(), 1);
        assert!(geometry.fold_lines[0].center);
    }

    #[test]
    fn fit_preserves_aspect_and_exact_uses_points() {
        let plan = build_plan(BindingMode::Booklet, 4, 8, DuplexFlip::ShortEdge);
        let mut settings = LayoutSettings::default();
        let geometry = layout_sheet(
            BindingMode::Booklet,
            &plan.sheets[0],
            Side::Front,
            &settings,
            &sizes(8, 400.0, 600.0),
        );
        let placement = &geometry.placements[0];
        let aspect = placement.page_rect.width() / placement.page_rect.height();
        assert!((aspect - 400.0 / 600.0).abs() < 1e-9);
        assert!(placement.page_rect.width() <= placement.slot_rect.width() + 0.01);
        assert!(placement.page_rect.height() <= placement.slot_rect.height() + 0.01);

        settings.scale = ScaleMode::Exact;
        let geometry = layout_sheet(
            BindingMode::Booklet,
            &plan.sheets[0],
            Side::Front,
            &settings,
            &sizes(8, 400.0, 600.0),
        );
        assert!((geometry.placements[0].page_rect.width() - 400.0).abs() < 0.01);
        assert!((geometry.placements[0].page_rect.height() - 600.0).abs() < 0.01);
    }

    #[test]
    fn creep_moves_inner_sheet_toward_spine() {
        let plan = build_plan(BindingMode::Signatures, 16, 16, DuplexFlip::ShortEdge);
        let settings = LayoutSettings {
            creep_mm: 0.25,
            ..LayoutSettings::default()
        };
        let inner_sheet = &plan.sheets[1];
        let geometry = layout_sheet(
            BindingMode::Signatures,
            inner_sheet,
            Side::Front,
            &settings,
            &sizes(16, 400.0, 600.0),
        );
        let left = &geometry.placements[0];
        let right = &geometry.placements[1];
        assert!((left.creep_shift_pt - 0.25 * PT_PER_MM).abs() < 1e-9);
        assert!((right.creep_shift_pt + 0.25 * PT_PER_MM).abs() < 1e-9);
        assert!(left.page_rect.center().x > left.slot_rect.center().x);
        assert!(right.page_rect.center().x < right.slot_rect.center().x);
    }

    #[test]
    fn quarto_and_octavo_have_expected_grids() {
        let quarto = build_plan(BindingMode::Quarto, 8, 8, DuplexFlip::ShortEdge);
        let geometry = layout_sheet(
            BindingMode::Quarto,
            &quarto.sheets[0],
            Side::Front,
            &LayoutSettings::default(),
            &sizes(8, 300.0, 400.0),
        );
        assert_eq!(geometry.rows, 2);
        assert_eq!(geometry.cols, 2);
        assert_eq!(geometry.fold_lines.len(), 2);
        assert!(
            geometry.placements[0].page_rect.center().y
                > geometry.placements[2].page_rect.center().y
        );
        assert_eq!(geometry.placements[0].page, Some(5));
        assert_eq!(geometry.placements[2].page, Some(8));

        let octavo = build_plan(BindingMode::Octavo, 16, 16, DuplexFlip::ShortEdge);
        let geometry = layout_sheet(
            BindingMode::Octavo,
            &octavo.sheets[0],
            Side::Front,
            &LayoutSettings::default(),
            &sizes(16, 300.0, 400.0),
        );
        assert_eq!(geometry.rows, 2);
        assert_eq!(geometry.cols, 4);
        assert_eq!(geometry.fold_lines.len(), 4);
    }

    #[test]
    fn exact_overflow_is_detected_in_all_modes() {
        for mode in [
            BindingMode::Booklet,
            BindingMode::Signatures,
            BindingMode::Quarto,
            BindingMode::Octavo,
        ] {
            let plan = build_plan(mode, 16, 16, DuplexFlip::ShortEdge);
            let page_sizes = sizes(16, 612.0, 792.0);
            let mut settings = LayoutSettings {
                scale: ScaleMode::Exact,
                ..LayoutSettings::default()
            };
            assert!(
                exact_pages_overflow(mode, &plan, &settings, &page_sizes),
                "{mode:?} should report exact overflow"
            );
            settings.scale = ScaleMode::Fit;
            assert!(!exact_pages_overflow(mode, &plan, &settings, &page_sizes));
        }
    }

    #[test]
    fn trim_marks_produce_eight_segments_outside_trim() {
        let rect = RectPt::new(100.0, 100.0, 300.0, 500.0);
        let marks = trim_marks(rect, 5.0, 10.0);
        assert_eq!(marks.len(), 8);
        assert!(marks.iter().all(|mark| {
            mark.a.x < rect.x0
                || mark.a.x > rect.x1
                || mark.a.y < rect.y0
                || mark.a.y > rect.y1
                || mark.b.x < rect.x0
                || mark.b.x > rect.x1
                || mark.b.y < rect.y0
                || mark.b.y > rect.y1
        }));
    }

    #[test]
    fn blank_placement_uses_nominal_page_size() {
        let plan = build_plan(BindingMode::Booklet, 4, 5, DuplexFlip::ShortEdge);
        let geometry = layout_sheet(
            BindingMode::Booklet,
            &plan.sheets[0],
            Side::Front,
            &LayoutSettings::default(),
            &sizes(5, 400.0, 600.0),
        );
        assert!(geometry.placements[0].blank);
        assert!(geometry.placements[0].page_rect.width() > 0.0);
    }
}
