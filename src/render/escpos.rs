use super::qr::qr_bytes;
use crate::document::model::{Alignment, DocumentElement, ReceiptDocument, SizeMode};
use anyhow::Result;
use image::GenericImageView;

pub fn render(doc: &ReceiptDocument, cpl: usize, margin: usize) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&[0x1B, 0x40]); // ESC @ : initialize

    let mut current_align: Option<u8> = None;
    let mut current_bold: Option<bool> = None;
    let mut current_size: Option<u8> = None;
    let mut current_underline: Option<bool> = None;

    for el in doc {
        match el {
            DocumentElement::Text {
                content,
                align,
                bold,
                size,
                underline,
            } => {
                let align_n = align_code(align);
                if current_align != Some(align_n) {
                    buf.extend_from_slice(&[0x1B, 0x61, align_n]);
                    current_align = Some(align_n);
                }
                if current_bold != Some(*bold) {
                    buf.extend_from_slice(&[0x1B, 0x45, if *bold { 1 } else { 0 }]);
                    current_bold = Some(*bold);
                }
                let size_n = size_code(size);
                if current_size != Some(size_n) {
                    buf.extend_from_slice(&[0x1D, 0x21, size_n]);
                    current_size = Some(size_n);
                }
                if current_underline != Some(*underline) {
                    buf.extend_from_slice(&[0x1B, 0x2D, if *underline { 1 } else { 0 }]);
                    current_underline = Some(*underline);
                }
                let padded = match align {
                    Alignment::Left => format!("{}{}", " ".repeat(margin), content),
                    Alignment::Right => format!("{}{}", content, " ".repeat(margin)),
                    Alignment::Center => content.clone(),
                };
                buf.extend_from_slice(padded.as_bytes());
                buf.push(b'\n');
            }
            DocumentElement::Divider => {
                let width = cpl.saturating_sub(margin * 2);
                buf.extend_from_slice(&[0x1B, 0x61, 1]); // force-center the (shorter) divider
                current_align = Some(1);
                buf.extend_from_slice("-".repeat(width).as_bytes());
                buf.push(b'\n');
            }
            DocumentElement::Blank => {
                // A bare LF (and ESC d) can be swallowed by emulators that
                // only flush spacing when there's buffered content. Printing
                // a single space character guarantees a non-empty line that
                // takes the same code path as normal text, so it reliably
                // produces a visible blank row.
                buf.push(b' ');
                buf.push(b'\n');
            }
            DocumentElement::Image(path) => match render_image(path) {
                Ok(bytes) => buf.extend_from_slice(&bytes),
                Err(e) => eprintln!("warning: failed to render image '{path}': {e}"),
            },
            DocumentElement::Qr(content) => match qr_bytes(content) {
                Ok(bytes) => buf.extend_from_slice(&bytes),
                Err(e) => eprintln!("warning: failed to render QR '{content}': {e}"),
            },
            DocumentElement::Row { left, right, bold } => {
                // Rows always span the full printable width left-aligned,
                // regardless of the current @align state (that only
                // affects plain Text/Divider elements).
                let align_n = align_code(&Alignment::Left);
                if current_align != Some(align_n) {
                    buf.extend_from_slice(&[0x1B, 0x61, align_n]);
                    current_align = Some(align_n);
                }
                if current_bold != Some(*bold) {
                    buf.extend_from_slice(&[0x1B, 0x45, if *bold { 1 } else { 0 }]);
                    current_bold = Some(*bold);
                }
                let width = cpl.saturating_sub(margin * 2);
                let line = format_row(left, right, width);
                buf.extend_from_slice(" ".repeat(margin).as_bytes());
                buf.extend_from_slice(line.as_bytes());
                buf.push(b'\n');
            }
            DocumentElement::Cut => {
                buf.extend_from_slice(&[0x1D, 0x56, 0x01]);
            }
            DocumentElement::Align(a) => {
                let align_n = align_code(a);
                if current_align != Some(align_n) {
                    buf.extend_from_slice(&[0x1B, 0x61, align_n]);
                    current_align = Some(align_n);
                }
            }
        }
    }

    Ok(buf)
}

fn align_code(a: &Alignment) -> u8 {
    match a {
        Alignment::Left => 0,
        Alignment::Center => 1,
        Alignment::Right => 2,
    }
}

fn size_code(s: &SizeMode) -> u8 {
    match s {
        SizeMode::Normal => 0x00,
        SizeMode::Double => 0x11,
    }
}

/// Lays out a two-column row: `left`, a run of '.' dot-leaders, then
/// `right`, filling exactly `width` characters. If the two columns alone
/// don't fit in `width`, `left` is truncated (no leader) so `right`
/// (usually a price) is never cut off. Byte-safe: no non-ASCII filler,
/// since this is written straight into the ESC/POS byte stream.
fn format_row(left: &str, right: &str, width: usize) -> String {
    let left_len = left.chars().count();
    let right_len = right.chars().count();

    if right_len >= width {
        return right.to_string();
    }

    if left_len + right_len >= width {
        let avail = width - right_len;
        let truncated: String = left.chars().take(avail).collect();
        return format!("{truncated}{right}");
    }

    let filler_len = width - left_len - right_len;
    format!("{left}{}{right}", ".".repeat(filler_len))
}

/// Loads an image, converts to 1-bit monochrome, and emits a GS v 0 raster command.
fn render_image(path: &str) -> Result<Vec<u8>> {
    let img = image::open(path)?.grayscale();

    let max_width = 384u32; // ~58mm paper at 203dpi
    let (w, _) = img.dimensions();
    let img = if w > max_width {
        let ratio = max_width as f64 / w as f64;
        img.resize(
            max_width,
            (img.dimensions().1 as f64 * ratio) as u32,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img
    };

    Ok(raster_bytes_from_gray(&img.to_luma8()))
}

/// Packs a grayscale image into a GS v 0 raster image command (1-bit
/// monochrome, threshold at mid-gray). Shared by logo images and QR codes so
/// both are drawn as actual pixels rather than emulator-specific command
/// bytes, avoiding parser quirks like stray characters leaking from
/// native-QR-command parameter bytes.
pub(crate) fn raster_bytes_from_gray(img: &image::GrayImage) -> Vec<u8> {
    let (w, h) = img.dimensions();
    let width_bytes = ((w + 7) / 8) as u16;

    let mut bitmap = vec![0u8; (width_bytes as u32 * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let luma = img.get_pixel(x, y).0[0];
            if luma < 128 {
                let idx = (y * width_bytes as u32 + x / 8) as usize;
                bitmap[idx] |= 0x80 >> (x % 8);
            }
        }
    }

    let mut out = Vec::new();
    out.extend_from_slice(&[0x1D, 0x76, 0x30, 0x00]); // GS v 0, normal mode
    out.extend_from_slice(&width_bytes.to_le_bytes());
    out.extend_from_slice(&(h as u16).to_le_bytes());
    out.extend_from_slice(&bitmap);
    out
}
