//! Validate card PNGs before exporting to the native clipboard or a user-chosen path.
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{
    io::{Cursor, Write},
    path::Path,
};

struct DecodedCard {
    png: Vec<u8>,
    rgba: Vec<u8>,
}

fn decode_card(encoded: &str, day: &str) -> Result<DecodedCard> {
    ensure!(
        day.len() == 10 && chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").is_ok(),
        "Invalid card date"
    );
    ensure!(encoded.len() <= 8 * 1024 * 1024, "Card image is too large");
    let encoded = encoded
        .strip_prefix("data:image/png;base64,")
        .context("Expected PNG image")?;
    let bytes = STANDARD.decode(encoded)?;
    let mut decoder = png::Decoder::new(Cursor::new(&bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    decoder.set_limits(png::Limits {
        bytes: 16 * 1024 * 1024,
    });
    let mut reader = decoder.read_info()?;
    let info = reader.info();
    ensure!(
        info.width == 1080 && info.height == 1350,
        "Invalid card dimensions"
    );
    ensure!(
        info.animation_control.is_none(),
        "Animated cards are unsupported"
    );
    let mut pixels = vec![0; reader.output_buffer_size().context("Invalid card image")?];
    let frame = reader.next_frame(&mut pixels)?;
    reader.finish()?;
    pixels.truncate(frame.buffer_size());
    let rgba = match frame.color_type {
        png::ColorType::Rgba => pixels,
        png::ColorType::Rgb => pixels
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|&[r, g, b]| [r, g, b, 255])
            .collect(),
        png::ColorType::Grayscale => pixels.into_iter().flat_map(|v| [v, v, v, 255]).collect(),
        png::ColorType::GrayscaleAlpha => pixels
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|&[gray, alpha]| [gray, gray, gray, alpha])
            .collect(),
        png::ColorType::Indexed => anyhow::bail!("Unexpanded card palette"),
    };
    ensure!(rgba.len() == 1080 * 1350 * 4, "Invalid card pixel data");
    Ok(DecodedCard { png: bytes, rgba })
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) fn copy(encoded: &str, day: &str) -> Result<()> {
    let card = decode_card(encoded, day)?;
    // OS-owned image data survives this short-lived handle on macOS/Windows.
    arboard::Clipboard::new()?.set_image(arboard::ImageData {
        width: 1080,
        height: 1350,
        bytes: std::borrow::Cow::Owned(card.rgba),
    })?;
    Ok(())
}
fn write_card(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("Invalid destination")?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
pub(super) fn save(
    window: &tao::window::Window,
    encoded: &str,
    day: &str,
    title: &str,
    period: clickpulse::report::Period,
    start: Option<chrono::NaiveDate>,
) -> Result<bool> {
    let card = decode_card(encoded, day)?;
    // Validation uses the same decoded pixels as Copy; release them before the dialog.
    drop(card.rgba);
    window.set_focus();
    let Some(path) = rfd::FileDialog::new()
        .set_parent(window)
        .set_title(title)
        .add_filter("PNG", &["png"])
        .set_file_name(if period == clickpulse::report::Period::Weekly {
            match start {
                Some(start) => format!("ClickPulse-Weekly-{start}_{day}.png"),
                None => format!("ClickPulse-Weekly-{day}.png"),
            }
        } else {
            format!("ClickPulse-{day}.png")
        })
        .save_file()
    else {
        return Ok(false);
    };
    write_card(&path, &card.png)?;
    Ok(true)
}
pub(super) fn open_shortcut_atlas() -> Result<()> {
    clickpulse::platform::open_web_page("https://endnone.github.io/ClickPulse/")
}

pub(super) fn open_reference(source: &str) -> Result<()> {
    let url = match source {
        "apple" => "https://support.apple.com/en-us/102650",
        "apple-screenshots" => "https://support.apple.com/en-us/102646",
        "mac-guide" => {
            "https://support.apple.com/guide/mac-help/intro-to-mac-keyboard-shortcuts-mchld6b9e240/mac"
        }
        "apple-safari" => {
            "https://support.apple.com/guide/mac-help/browse-the-web-mchlgtd_web01/mac"
        }
        "ms-edge" => {
            "https://support.microsoft.com/en-us/edge/keyboard-shortcuts-in-microsoft-edge"
        }
        "ms-game" => {
            "https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-record-your-screen-with-xbox-game-bar"
        }
        "gnome-settings" => "https://help.gnome.org/gnome-help/keyboard-shortcuts-set.html",
        "windows" => {
            "https://support.microsoft.com/en-us/accessibility/windows/keyboard-shortcuts-in-windows"
        }
        "gnome" => "https://help.gnome.org/gnome-help/shell-keyboard-shortcuts.html",
        "gnome-terminal" => "https://help.gnome.org/gnome-terminal/adv-keyboard-shortcuts.html",
        _ => anyhow::bail!("Unknown reference source"),
    };
    clickpulse::platform::open_web_page(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn encode_pixels(width: u32, height: u32, color: png::ColorType, pixels: &[u8]) -> String {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, width, height);
            encoder.set_color(color);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(pixels)
                .unwrap();
        }
        format!("data:image/png;base64,{}", STANDARD.encode(&bytes))
    }
    #[test]
    fn clipboard_pixels_preserve_rgba_colors_and_expand_grayscale_alpha() {
        for (color, pixel, expected) in [
            (
                png::ColorType::Rgba,
                vec![22, 88, 191, 127],
                [22, 88, 191, 127],
            ),
            (png::ColorType::Rgb, vec![22, 88, 191], [22, 88, 191, 255]),
            (
                png::ColorType::GrayscaleAlpha,
                vec![88, 127],
                [88, 88, 88, 127],
            ),
            (png::ColorType::Grayscale, vec![88], [88, 88, 88, 255]),
        ] {
            let encoded = encode_pixels(1080, 1350, color, &pixel.repeat(1080 * 1350));
            let card = decode_card(&encoded, "2026-11-17").unwrap();
            assert_eq!(card.rgba.len(), 1080 * 1350 * 4);
            assert!(
                card.rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|rgba| *rgba == expected)
            );
        }
    }
    #[test]
    fn invalid_dimensions_and_oversized_payloads_never_reach_the_clipboard() {
        let small = encode_pixels(1, 1, png::ColorType::Rgba, &[1, 2, 3, 255]);
        assert!(decode_card(&small, "2026-11-17").is_err());
        assert!(decode_card(&"x".repeat(8 * 1024 * 1024 + 1), "2026-11-17").is_err());
    }
    #[test]
    fn rejects_untrusted_card_payloads_before_opening_dialog() {
        for (payload, day) in [
            ("data:text/html;base64,AA==", "2026-11-17"),
            ("data:image/png;base64,AA==", "../x"),
            ("data:image/png;base64,AA==", "2026-02-30"),
        ] {
            assert!(decode_card(payload, day).is_err());
        }
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, 1080, 1350);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&vec![255; 1080 * 1350 * 3])
                .unwrap();
        }
        let encoded = format!("data:image/png;base64,{}", STANDARD.encode(&bytes));
        let decoded = decode_card(&encoded, "2026-11-17").unwrap();
        assert_eq!(decoded.png, bytes);
        assert_eq!(decoded.rgba.len(), 1080 * 1350 * 4);
        assert!(decoded.rgba.iter().all(|value| *value == 255));
        let truncated = format!(
            "data:image/png;base64,{}",
            STANDARD.encode(&bytes[..bytes.len() - 12])
        );
        assert!(decode_card(&truncated, "2026-11-17").is_err());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("card.png");
        std::fs::write(&path, b"old").unwrap();
        write_card(&path, &bytes).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
