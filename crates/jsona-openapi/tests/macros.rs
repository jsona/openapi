#[macro_export]
macro_rules! snapshot {
    ($source:literal) => {
        let input = include_str!($source);
        let openapi: jsona_openapi::Openapi = input.parse().unwrap();
        let output = serde_json::to_string_pretty(&openapi).unwrap();
        insta::assert_snapshot!(output);
    };
}
