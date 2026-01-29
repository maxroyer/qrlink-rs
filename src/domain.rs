#[path = "domain/link.rs"]
mod link;
#[path = "domain/short_code.rs"]
mod short_code;
#[path = "domain/ttl.rs"]
mod ttl;

pub use link::{Link, LinkResponse};
pub use short_code::ShortCode;
pub use ttl::Ttl;
