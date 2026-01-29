use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::qr::QrGenerator;

/// Service for QR code generation operations.
#[derive(Clone)]
pub struct QrService {
    generator: QrGenerator,
    base_url: String,
}

impl QrService {
    pub fn new(config: &Config) -> AppResult<Self> {
        let generator = QrGenerator::new(config.qr_size, config.qr_branding_logo.clone())
            .map_err(|e| AppError::QrGeneration(e))?;

        Ok(Self {
            generator,
            base_url: config.base_url.clone(),
        })
    }

    /// Generate a QR code PNG for the given short code.
    pub fn generate_qr(&self, short_code: &str) -> AppResult<Vec<u8>> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), short_code);
        self.generate_for_url(&url)
    }

    /// Generate a QR code PNG for a raw URL (no shortening).
    pub fn generate_for_url(&self, url: &str) -> AppResult<Vec<u8>> {
        self.generator
            .generate(url)
            .map_err(|e| AppError::QrGeneration(e))
    }
}
