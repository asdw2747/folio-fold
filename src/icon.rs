use egui::{Color32, IconData};

const ICON_SIZE: u32 = 256;
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

/// Load the baked-in application icon, resizing it to the 256 px size expected
/// by desktop window managers. The PNG is embedded in the executable; there is
/// no runtime asset lookup and no separate icon file to ship.
pub fn load_icon() -> IconData {
    decode_icon().unwrap_or_else(fallback_icon)
}

fn decode_icon() -> Option<IconData> {
    let image = image::load_from_memory_with_format(ICON_PNG, image::ImageFormat::Png).ok()?;
    let rgba = image::imageops::resize(
        &image.to_rgba8(),
        ICON_SIZE,
        ICON_SIZE,
        image::imageops::FilterType::Lanczos3,
    );
    Some(IconData {
        rgba: rgba.into_raw(),
        width: ICON_SIZE,
        height: ICON_SIZE,
    })
}

/// Fallback mark if the baked-in PNG ever fails to decode.
fn fallback_icon() -> IconData {
    let mut buffer = vec![[0u8; 4]; (ICON_SIZE * ICON_SIZE) as usize];
    let background = Color32::from_rgb(14, 19, 28);
    let border = Color32::from_rgb(42, 54, 72);
    let paper = Color32::from_rgb(239, 236, 227);
    let paper_shade = Color32::from_rgb(211, 207, 197);
    let accent = Color32::from_rgb(77, 208, 200);
    let ink = Color32::from_rgb(30, 42, 54);

    fill_rounded_rect(&mut buffer, 8.0, 8.0, 248.0, 248.0, 44.0, background);
    stroke_rounded_rect(&mut buffer, 8.0, 8.0, 248.0, 248.0, 44.0, border, 2.0);

    // Back sheet, slightly offset.
    fill_rect(&mut buffer, 72.0, 60.0, 184.0, 198.0, paper_shade);
    // Front sheet.
    fill_rect(&mut buffer, 60.0, 48.0, 172.0, 186.0, paper);
    // Fold axes.
    fill_rect(&mut buffer, 115.0, 49.0, 117.0, 185.0, accent);
    fill_rect(&mut buffer, 61.0, 116.0, 171.0, 118.0, accent);
    // Page spine shadow and a tiny text-like block.
    fill_rect(&mut buffer, 78.0, 72.0, 108.0, 77.0, ink);
    fill_rect(&mut buffer, 78.0, 86.0, 101.0, 89.0, ink);
    fill_rect(&mut buffer, 78.0, 96.0, 105.0, 99.0, ink);
    fill_rect(&mut buffer, 126.0, 72.0, 158.0, 77.0, ink);
    fill_rect(&mut buffer, 126.0, 86.0, 150.0, 89.0, ink);
    fill_rect(&mut buffer, 78.0, 137.0, 158.0, 142.0, ink);
    fill_rect(&mut buffer, 78.0, 151.0, 145.0, 154.0, ink);
    fill_rect(&mut buffer, 78.0, 162.0, 151.0, 165.0, ink);
    circle(&mut buffer, 198.0, 58.0, 12.0, accent);

    IconData {
        rgba: buffer.into_iter().flatten().collect(),
        width: ICON_SIZE,
        height: ICON_SIZE,
    }
}

fn blend(pixel: &mut [u8; 4], color: Color32) {
    let alpha = u32::from(color.a());
    if alpha == 0 {
        return;
    }
    let inverse = 255 - alpha;
    pixel[0] = ((u32::from(color.r()) * alpha + u32::from(pixel[0]) * inverse) / 255) as u8;
    pixel[1] = ((u32::from(color.g()) * alpha + u32::from(pixel[1]) * inverse) / 255) as u8;
    pixel[2] = ((u32::from(color.b()) * alpha + u32::from(pixel[2]) * inverse) / 255) as u8;
    pixel[3] = (alpha + u32::from(pixel[3]) * inverse / 255) as u8;
}

fn fill_rect(buffer: &mut [[u8; 4]], left: f32, top: f32, right: f32, bottom: f32, color: Color32) {
    let x0 = left.max(0.0).floor() as u32;
    let x1 = right.min(ICON_SIZE as f32).ceil() as u32;
    let y0 = top.max(0.0).floor() as u32;
    let y1 = bottom.min(ICON_SIZE as f32).ceil() as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            blend(&mut buffer[(y * ICON_SIZE + x) as usize], color);
        }
    }
}

fn circle(buffer: &mut [[u8; 4]], cx: f32, cy: f32, radius: f32, color: Color32) {
    let x0 = (cx - radius).floor().max(0.0) as u32;
    let x1 = (cx + radius).ceil().min(ICON_SIZE as f32) as u32;
    let y0 = (cy - radius).floor().max(0.0) as u32;
    let y1 = (cy + radius).ceil().min(ICON_SIZE as f32) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= radius * radius {
                blend(&mut buffer[(y * ICON_SIZE + x) as usize], color);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fill_rounded_rect(
    buffer: &mut [[u8; 4]],
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    radius: f32,
    color: Color32,
) {
    for y in top.max(0.0) as u32..bottom.min(ICON_SIZE as f32) as u32 {
        for x in left.max(0.0) as u32..right.min(ICON_SIZE as f32) as u32 {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let nearest_x = px.clamp(left + radius, right - radius);
            let nearest_y = py.clamp(top + radius, bottom - radius);
            let dx = px - nearest_x;
            let dy = py - nearest_y;
            if dx * dx + dy * dy <= radius * radius {
                blend(&mut buffer[(y * ICON_SIZE + x) as usize], color);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn stroke_rounded_rect(
    buffer: &mut [[u8; 4]],
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    radius: f32,
    color: Color32,
    width: f32,
) {
    let outer = radius;
    let inner = (radius - width).max(0.0);
    for y in top.max(0.0) as u32..bottom.min(ICON_SIZE as f32) as u32 {
        for x in left.max(0.0) as u32..right.min(ICON_SIZE as f32) as u32 {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let outer_x = px.clamp(left + outer, right - outer);
            let outer_y = py.clamp(top + outer, bottom - outer);
            let inner_x = px.clamp(left + inner, right - inner);
            let inner_y = py.clamp(top + inner, bottom - inner);
            let outer_dx = px - outer_x;
            let outer_dy = py - outer_y;
            let inner_dx = px - inner_x;
            let inner_dy = py - inner_y;
            let outside_inner = inner_dx * inner_dx + inner_dy * inner_dy > inner * inner;
            if outer_dx * outer_dx + outer_dy * outer_dy <= outer * outer && outside_inner {
                blend(&mut buffer[(y * ICON_SIZE + x) as usize], color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_is_valid_rgba() {
        let icon = load_icon();
        assert_eq!(icon.width, ICON_SIZE);
        assert_eq!(icon.height, ICON_SIZE);
        assert_eq!(icon.rgba.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
        assert!(icon
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| pixel[3] > 0));
    }
}
