use anyhow::Result;
use jsona_openapi::from_str;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: jsona-openapi <jsona file> [<output file>]")
    }
    match run(&args[1], args.get(2)) {
        Ok(_) => {}
        Err(err) => {
            println!("{}", err);
            process::exit(1)
        }
    }
}

fn run(input_file: &str, output_file: Option<&String>) -> Result<()> {
    let input = fs::read_to_string(input_file)
        .map_err(|e| anyhow::anyhow!("fail to read input file, {}", e))?;
    let spec = from_str(input.as_str()).map_err(|e| anyhow::anyhow!("fail to parse spec, {}", e))?;
    let is_json = match output_file {
        Some(value) => value.ends_with(".json"),
        None => false,
    };
    let output = if is_json {
        serde_json::to_string_pretty(&spec)?
    } else {
        serde_yaml::to_string(&spec)?
    };
    if output_file.is_none() {
        println!("{}", output);
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_file.unwrap())
        .map_err(|e| anyhow::anyhow!("fail to write output file, {}", e))?;
    file.write_all(output.as_bytes())?;
    Ok(())
}
