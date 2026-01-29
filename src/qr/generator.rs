use image::{ImageEncoder, Rgba, RgbaImage};
use qrcode::{EcLevel, QrCode};
use std::path::PathBuf;

/// Maximum logo size as a fraction of QR code size (20%)
const LOGO_MAX_SCALE: f32 = 0.20;

/// QR code generator with optional branding logo.
#[derive(Clone)]
pub struct QrGenerator {
    size: u32,
    logo: Option<RgbaImage>,
}

impl QrGenerator {
    /// Create a new QR generator with the given size and optional logo path.
    pub fn new(size: u32, logo_path: Option<PathBuf>) -> Result<Self, String> {
        let logo = match logo_path {
            Some(path) => {
                let logo_image = load_logo(&path)?;
                Some(logo_image)
            }
            None => None,
        };

        Ok(Self { size, logo })
    }

    /// Generate a QR code PNG for the given content.
    pub fn generate(&self, content: &str) -> Result<Vec<u8>, String> {
        // Create QR code with high error correction (required for logo overlay)
        let qr = QrCode::with_error_correction_level(content, EcLevel::H)
            .map_err(|e| format!("Failed to create QR code: {}", e))?;

        // Render QR code to image
        let qr_image = qr
            .render::<Rgba<u8>>()
            .quiet_zone(true)
            .min_dimensions(self.size, self.size)
            .max_dimensions(self.size, self.size)
            .build();

        let mut img: RgbaImage = qr_image;

        // Overlay logo if available
        if let Some(logo) = &self.logo {
            img = overlay_logo(img, logo)?;
        }

        // Encode to PNG
        let mut png_bytes: Vec<u8> = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
        encoder
            .write_image(
                img.as_raw(),
                img.width(),
                img.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        Ok(png_bytes)
    }
}

/// Load and prepare a logo image from file (PNG or SVG).
fn load_logo(path: &PathBuf) -> Result<RgbaImage, String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match extension.as_deref() {
        Some("svg") => load_svg_logo(path),
        Some("png") | Some("jpg") | Some("jpeg") => load_raster_logo(path),
        _ => Err(format!(
            "Unsupported logo format: {:?}. Use PNG or SVG.",
            path
        )),
    }
}

/// Load an SVG logo and render it to a raster image.
fn load_svg_logo(path: &PathBuf) -> Result<RgbaImage, String> {
    let svg_data =
        std::fs::read(path).map_err(|e| format!("Failed to read SVG file: {}", e))?;

    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg_data, &options)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    let size = tree.size();
    let width = size.width() as u32;
    let height = size.height() as u32;

    // Render at a reasonable size for logo overlay
    let scale = 200.0 / width.max(height) as f32;
    let scaled_width = (width as f32 * scale) as u32;
    let scaled_height = (height as f32 * scale) as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(scaled_width, scaled_height)
        .ok_or_else(|| "Failed to create pixmap".to_string())?;

    // Fill with white background
    pixmap.fill(resvg::tiny_skia::Color::WHITE);

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert to image::RgbaImage
    let img = RgbaImage::from_raw(scaled_width, scaled_height, pixmap.take())
        .ok_or_else(|| "Failed to create image from pixmap".to_string())?;

    Ok(img)
}

/// Load a raster logo image.
fn load_raster_logo(path: &PathBuf) -> Result<RgbaImage, String> {
    let img = image::open(path).map_err(|e| format!("Failed to open logo image: {}", e))?;
    Ok(img.to_rgba8())
}

/// Overlay the logo in the center of the QR code.
fn overlay_logo(mut qr: RgbaImage, logo: &RgbaImage) -> Result<RgbaImage, String> {
    let qr_size = qr.width().min(qr.height());
    let max_logo_size = (qr_size as f32 * LOGO_MAX_SCALE) as u32;

    // Scale logo to fit
    let logo_width = logo.width();
    let logo_height = logo.height();
    let scale = (max_logo_size as f32 / logo_width.max(logo_height) as f32).min(1.0);

    let new_width = (logo_width as f32 * scale) as u32;
    let new_height = (logo_height as f32 * scale) as u32;

    let scaled_logo = image::imageops::resize(
        logo,
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    );

    // Calculate center position
    let x_offset = (qr.width() - new_width) / 2;
    let y_offset = (qr.height() - new_height) / 2;

    // Create white background padding around logo
    let padding = 4;
    let bg_width = new_width + padding * 2;
    let bg_height = new_height + padding * 2;
    let bg_x = x_offset.saturating_sub(padding);
    let bg_y = y_offset.saturating_sub(padding);

    // Draw white background rectangle
    for y in bg_y..(bg_y + bg_height).min(qr.height()) {
        for x in bg_x..(bg_x + bg_width).min(qr.width()) {
            qr.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    // Overlay the logo with alpha blending
    for (lx, ly, pixel) in scaled_logo.enumerate_pixels() {
        let qx = x_offset + lx;
        let qy = y_offset + ly;

        if qx < qr.width() && qy < qr.height() {
            let alpha = pixel[3] as f32 / 255.0;
            if alpha > 0.0 {
                let bg = qr.get_pixel(qx, qy);
                let blended = Rgba([
                    ((1.0 - alpha) * bg[0] as f32 + alpha * pixel[0] as f32) as u8,
                    ((1.0 - alpha) * bg[1] as f32 + alpha * pixel[1] as f32) as u8,
                    ((1.0 - alpha) * bg[2] as f32 + alpha * pixel[2] as f32) as u8,
                    255,
                ]);
                qr.put_pixel(qx, qy, blended);
            }
        }
    }

    Ok(qr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_without_logo() {
        let generator = QrGenerator::new(256, None).unwrap();
        let result = generator.generate("https://example.com");
        assert!(result.is_ok());

        let png_data = result.unwrap();
        assert!(!png_data.is_empty());
        // Check PNG magic bytes
        assert_eq!(&png_data[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_generate_qr_with_content() {
        let generator = QrGenerator::new(512, None).unwrap();
        let result = generator.generate("https://s.company.local/Ab3kP9x");
        assert!(result.is_ok());
    }
}
