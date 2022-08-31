#[macro_export]
macro_rules! snapshot {
    ($source:literal) => {
        let input = include_str!($source);
        let node: jsona::dom::Node = input.parse().unwrap();
        let openapi = jsona_openapi::Openapi::try_from(&node).unwrap();
        let output = serde_json::to_string_pretty(&openapi).unwrap();
        insta::assert_snapshot!(output);
    };
}
