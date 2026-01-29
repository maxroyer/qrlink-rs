#[path = "http/router.rs"]
pub mod router;
#[path = "http/handlers.rs"]
mod handlers;

pub use router::create_router;
