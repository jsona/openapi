pub mod error;
pub mod loader;
pub mod spec;

#[doc(inline)]
pub use error::Error;

pub fn from_str(input: &str) -> Result<spec::Spec, Error> {
    loader::Loader::load_from_str(input)
}
