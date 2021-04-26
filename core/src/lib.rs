pub mod error;
pub mod loader;

#[doc(inline)]
pub use error::Error;

#[doc(inline)]
pub use jsona_openapi_spec::Spec;

pub fn from_str(input: &str) -> Result<Spec, Error> {
    loader::Loader::load_from_str(input)
}