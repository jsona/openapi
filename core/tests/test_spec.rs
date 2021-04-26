use jsona_openapi::from_str;

#[test]
fn test_cases() {
    let data = include_str!("spec/all_cases.jsona");
    let expect = include_str!("spec/all_cases.json");
    let spec = from_str(data).unwrap();
    let output = serde_json::to_string_pretty(&spec).unwrap();
    assert_eq!(expect, output);
}

#[test]
fn test_readme_snippet() {
    let data = include_str!("spec/readme_snippet.jsona");
    let expect = include_str!("spec/readme_snippet.json");
    let spec = from_str(data).unwrap();
    let output = serde_json::to_string_pretty(&spec).unwrap();
    assert_eq!(expect, output);
}

#[test]
fn test_petstore() {
    let data = include_str!("spec/petstore.jsona");
    let expect = include_str!("spec/petstore.json");
    let spec = from_str(data).unwrap();
    let output = serde_json::to_string_pretty(&spec).unwrap();
    assert_eq!(expect, output);
}
