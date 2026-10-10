//! Embed the supplied brand artwork and derive the native template from its alpha.
use std::io::Cursor;

use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};

const MARK: &[u8] = include_bytes!("../assets/clickpulse-mark-blue-transparent-v1-1024.png");
pub const TRAY_SIZE: u32 = 44;

pub fn mark_data_url() -> String {
    format!("data:image/png;base64,{}", STANDARD.encode(MARK))
}

pub fn font_css() -> String {
    #[cfg(target_os = "windows")]
    let fonts = [
        ("ClickPulse UI", "fonts/Inter.ttf".to_owned()),
        ("ClickPulse Sans", "fonts/NotoSansSC.ttf".to_owned()),
        ("ClickPulse Mono", "fonts/NotoSansMono.ttf".to_owned()),
    ];
    #[cfg(not(target_os = "windows"))]
    let fonts = [
        (
            "ClickPulse UI",
            include_bytes!("../assets/fonts/Inter.ttf").as_slice(),
        ),
        (
            "ClickPulse Sans",
            include_bytes!("../assets/fonts/NotoSansSC.ttf").as_slice(),
        ),
        (
            "ClickPulse Mono",
            include_bytes!("../assets/fonts/NotoSansMono.ttf").as_slice(),
        ),
    ]
    .map(|(family, bytes)| {
        (
            family,
            format!("data:font/ttf;base64,{}", STANDARD.encode(bytes)),
        )
    });
    fonts.into_iter().map(|(family, source)| {
        format!("@font-face{{font-family:'{family}';src:url('{source}') format('truetype');font-weight:100 900;font-style:normal;font-display:swap}}")
    }).collect::<Vec<_>>().join("\n")
}

pub fn tray_rgba() -> Result<Vec<u8>> {
    let mut reader = png::Decoder::new(Cursor::new(MARK)).read_info()?;
    let mut pixels = vec![0; reader.output_buffer_size().context("Logo 图片尺寸无效")?];
    let info = reader.next_frame(&mut pixels)?;
    ensure!(
        info.color_type == png::ColorType::Rgba && info.bit_depth == png::BitDepth::Eight,
        "透明 Logo 需要 8 位 RGBA PNG"
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
    ensure!(right > left && bottom > top, "Logo 没有可见图形");
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
                #[cfg(target_os = "windows")]
                rgba[(y * size + x) * 4..(y * size + x) * 4 + 3].copy_from_slice(&[31, 82, 179]);
            }
        }
    }
    Ok(rgba)
}
