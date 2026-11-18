use egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Frame, Margin, Stroke, TextStyle, Theme,
};

pub const BG: Color32 = Color32::from_rgb(9, 12, 18);
pub const PANEL: Color32 = Color32::from_rgb(14, 19, 28);
pub const CARD: Color32 = Color32::from_rgb(20, 27, 39);
pub const CARD_HOVER: Color32 = Color32::from_rgb(25, 35, 50);
pub const CARD_ACTIVE: Color32 = Color32::from_rgb(18, 42, 45);
pub const BORDER: Color32 = Color32::from_rgb(42, 54, 72);
pub const BORDER_FAINT: Color32 = Color32::from_rgb(29, 39, 54);
pub const ACCENT: Color32 = Color32::from_rgb(77, 208, 200);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(30, 104, 105);
pub const ACCENT_GLOW: Color32 = Color32::from_rgba_premultiplied(24, 78, 79, 60);
pub const TEXT: Color32 = Color32::from_rgb(226, 234, 244);
pub const MUTED: Color32 = Color32::from_rgb(135, 150, 170);
pub const FAINT: Color32 = Color32::from_rgb(88, 102, 122);
pub const WARNING: Color32 = Color32::from_rgb(255, 184, 76);
pub const ERROR: Color32 = Color32::from_rgb(255, 104, 112);
pub const SUCCESS: Color32 = Color32::from_rgb(92, 214, 140);
pub const VIOLET: Color32 = Color32::from_rgb(160, 140, 255);

pub const PAPER: Color32 = Color32::from_rgb(239, 236, 227);
pub const PAPER_SHADE: Color32 = Color32::from_rgb(211, 207, 197);
pub const PAPER_EDGE: Color32 = Color32::from_rgb(177, 173, 164);
pub const PAPER_INK: Color32 = Color32::from_rgb(40, 48, 58);
pub const PAPER_INK_FAINT: Color32 = Color32::from_rgb(112, 118, 120);
pub const TRIM_MARK: Color32 = Color32::from_rgb(176, 128, 72);
pub const FOLD_MARK: Color32 = Color32::from_rgb(60, 122, 128);

pub fn apply(ctx: &Context) {
    ctx.set_theme(Theme::Dark);
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(8, 11, 16);
    visuals.faint_bg_color = Color32::from_rgb(24, 32, 44);
    visuals.override_text_color = Some(TEXT);
    visuals.hyperlink_color = ACCENT;
    visuals.warn_fg_color = WARNING;
    visuals.error_fg_color = ERROR;
    visuals.selection.bg_fill = ACCENT_DIM;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(26, 34, 47);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(23, 30, 42);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(34, 45, 61);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(30, 40, 55);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_DIM);
    visuals.widgets.active.bg_fill = ACCENT_DIM;
    visuals.widgets.active.weak_bg_fill = ACCENT_DIM;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    ctx.set_visuals(visuals);

    ctx.style_mut_of(Theme::Dark, |style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(9.0, 5.0);
        style.spacing.slider_width = 118.0;
        style.spacing.interact_size.y = 22.0;
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(22.0, FontFamily::Proportional),
        );
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(13.0, FontFamily::Proportional));
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(12.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(10.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(11.0, FontFamily::Monospace),
        );
    });
}

pub fn panel_frame() -> Frame {
    Frame::new()
        .fill(PANEL)
        .inner_margin(Margin::symmetric(12, 10))
        .stroke(Stroke::new(1.0, BORDER))
}

pub fn card_frame() -> Frame {
    Frame::new()
        .fill(CARD)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(10))
        .stroke(Stroke::new(1.0, BORDER))
}

pub fn letter_spaced(text: &str) -> String {
    text.chars()
        .flat_map(|character| [character, '\u{2009}'])
        .collect::<String>()
        .trim_end()
        .to_string()
}

pub fn short_name(name: &str, max_chars: usize) -> String {
    let count = name.chars().count();
    if count <= max_chars {
        name.to_string()
    } else {
        let mut output: String = name.chars().take(max_chars.saturating_sub(1)).collect();
        output.push('…');
        output
    }
}
