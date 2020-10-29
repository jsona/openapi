use jsona_openapi::parse;

#[test]
fn test_cases() {
    let data = include_str!("spec/test_cases.jsona");
    let expect = include_str!("spec/test_cases.json");
    let spec = parse(data).unwrap();
    let output = serde_json::to_string_pretty(&spec).unwrap();
    // println!("{}", output);
    assert_eq!(expect, output);
}

#[test]
fn test_readme_snippet() {
    let data = include_str!("spec/test_readme_snippet.jsona");
    let expect = include_str!("spec/test_readme_snippet.json");
    let spec = parse(data).unwrap();
    let output = serde_json::to_string_pretty(&spec).unwrap();
    println!("{}", output);
    assert_eq!(expect, output);
}
