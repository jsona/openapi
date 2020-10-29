pub mod error;
pub mod loader;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub use loader::Loader;
