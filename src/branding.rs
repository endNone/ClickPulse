//! Embed the supplied brand artwork and derive the native template from its alpha.
use std::io::Cursor;

use anyhow::{Context, Result, ensure};
const MARK: &[u8] = include_bytes!("../assets/clickpulse-mark-blue-transparent-v1-1024.png");
pub const TRAY_SIZE: u32 = 44;

pub fn ui_mark_png() -> &'static [u8] {
    include_bytes!("../assets/clickpulse-mark-blue-transparent-v1-128.png")
}

pub fn font_css() -> &'static str {
    include_str!("../ui/fonts.css")
}

pub fn tray_rgba() -> Result<Vec<u8>> {
    let mut reader = png::Decoder::new(Cursor::new(MARK)).read_info()?;
    let mut pixels = vec![
        0;
        reader
            .output_buffer_size()
            .context("Invalid logo image dimensions")?
    ];
    let info = reader.next_frame(&mut pixels)?;
    ensure!(
        info.color_type == png::ColorType::Rgba && info.bit_depth == png::BitDepth::Eight,
        "The transparent logo requires an 8-bit RGBA PNG"
    );
    let width = info.width as usize;
    let height = info.height as usize;
    let alpha = |x: usize, y: usize| pixels[(y * width + x) * 4 + 3];
    let mut left = width;
    let mut top = height;
    let mut right = 0;
    let mut bottom = 0;
    for y in 0..height {
        for x in 0..width {
            if alpha(x, y) > 32 {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x + 1);
                bottom = bottom.max(y + 1);
            }
        }
    }
    ensure!(
        right > left && bottom > top,
        "The logo has no visible pixels"
    );
    let size = TRAY_SIZE as usize;
    let scale = ((right - left).max(bottom - top)) as f64 / (size - 4) as f64;
    let center_x = (left + right) as f64 / 2.0;
    let center_y = (top + bottom) as f64 / 2.0;
    let mut rgba = vec![0; size * size * 4];
    // Area sampling keeps the original silhouette crisp at menu-bar sizes.
    // macOS uses this alpha as a template and chooses its own foreground color.
    for y in 0..size {
        for x in 0..size {
            let sx = center_x + (x as f64 - size as f64 / 2.0) * scale;
            let sy = center_y + (y as f64 - size as f64 / 2.0) * scale;
            let x0 = sx.floor().max(0.0) as usize;
            let y0 = sy.floor().max(0.0) as usize;
            let x1 = (sx + scale).ceil().max(0.0) as usize;
            let y1 = (sy + scale).ceil().max(0.0) as usize;
            let mut sum = 0u64;
            for iy in y0..y1.min(height) {
                for ix in x0..x1.min(width) {
                    sum += alpha(ix, iy) as u64;
                }
            }
            let area = (x1 - x0) * (y1 - y0);
            if area > 0 {
                rgba[(y * size + x) * 4 + 3] = (sum / area as u64) as u8;
                #[cfg(any(target_os = "windows", target_os = "linux"))]
                rgba[(y * size + x) * 4..(y * size + x) * 4 + 3].copy_from_slice(&[31, 82, 179]);
            }
        }
    }
    Ok(rgba)
}
