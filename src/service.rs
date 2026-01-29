#[path = "service/link_service.rs"]
mod link_service;
#[path = "service/qr_service.rs"]
mod qr_service;

pub use link_service::LinkService;
pub use qr_service::QrService;
