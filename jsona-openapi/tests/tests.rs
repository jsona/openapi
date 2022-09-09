#[macro_use]
mod macros;

#[test]
fn all_case() {
    snapshot!("fixtures/all_cases.jsona");
}

#[test]
fn petstore() {
    snapshot!("fixtures/petstore.jsona");
}
