use jsona_openapi::Loader;

const SPEC: &str = include_str!("spec/test_spec.jsona");
#[test]
fn test_load() {
    let expect = include_str!("spec/test_spec.json");
    let spec = Loader::load_from_str(SPEC).unwrap();
    let output = serde_json::to_string_pretty(&spec).unwrap();
    // print!("{}", output);
    assert_eq!(expect, output);
}
