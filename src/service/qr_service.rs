use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::qr::QrGenerator;

/// Service for QR code generation operations.
#[derive(Clone)]
pub struct QrService {
    generator: QrGenerator,
}

impl QrService {
    pub fn new(config: &Config) -> AppResult<Self> {
        let generator = QrGenerator::new(config.qr_size, config.qr_branding_logo.clone())
            .map_err(AppError::QrGeneration)?;

        Ok(Self { generator })
    }

    /// Generate a QR code PNG for a raw URL (no shortening).
    pub fn generate_for_url(&self, url: &str) -> AppResult<Vec<u8>> {
        self.generator.generate(url).map_err(AppError::QrGeneration)
    }
}
