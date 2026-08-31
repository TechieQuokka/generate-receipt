use super::escpos::raster_bytes_from_gray;
use anyhow::{Context, Result};
use image::{GrayImage, Luma};
use qrcode::{Color, QrCode};

/// Renders `content` as a QR code and returns it as a GS v 0 raster image
/// command (actual pixels), instead of native ESC/POS QR commands
/// (GS ( k series). Native QR commands are opaque parameter bytes that some
/// emulators/printers mis-parse as printable text when their command
/// filtering isn't perfect; rendering as a bitmap sidesteps that entirely
/// and works identically on real printers.
pub fn qr_bytes(content: &str) -> Result<Vec<u8>> {
    let code = QrCode::new(content.as_bytes())
        .with_context(|| format!("failed to encode QR content: {content}"))?;

    let width_modules = code.width() as u32;
    let colors = code.to_colors();

    let module_px = 6u32; // pixels per QR module
    let quiet_zone = 2u32; // modules of white border (recommended minimum)
    let img_modules = width_modules + quiet_zone * 2;
    let img_size = img_modules * module_px;

    let mut img = GrayImage::from_pixel(img_size, img_size, Luma([255u8]));

    for y in 0..width_modules {
        for x in 0..width_modules {
            let idx = (y * width_modules + x) as usize;
            if colors[idx] == Color::Dark {
                let px0 = (x + quiet_zone) * module_px;
                let py0 = (y + quiet_zone) * module_px;
                for dy in 0..module_px {
                    for dx in 0..module_px {
                        img.put_pixel(px0 + dx, py0 + dy, Luma([0u8]));
                    }
                }
            }
        }
    }

    Ok(raster_bytes_from_gray(&img))
}
