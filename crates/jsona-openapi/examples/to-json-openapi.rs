use jsona_openapi::Openapi;

fn main() {
	let jsona_file = std::env::args().nth(1).expect("Usage: format <jsona-file>");
    let jsona_file_path = std::path::Path::new(&jsona_file);
    let jsona_content = std::fs::read_to_string(jsona_file_path).unwrap();
    let openapi: Openapi = jsona_content.parse().unwrap();
	let output = serde_json::to_string_pretty(&openapi).unwrap();
    println!("{}", output);
}