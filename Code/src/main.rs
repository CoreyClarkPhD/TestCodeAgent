use dotenv::dotenv;
use std::{env, fs};

use anyhow::Result;
use serde_json::Value;

use compiler::CompileJsonOutput;

mod ai;
mod compiler;
mod nodes;
mod parser;
mod system;
mod transform;

fn main() -> Result<()> {
    let _ = dotenv();

    if let Err(e) = env::var("OPENAI_TOKEN") {
        println!("OPENAI_TOKEN not set");
        println!("Error: {}", e);
        println!("Please set OPENAI_TOKEN in .env");
        return Ok(());
    }

    system::create_worker_thread();

    // Open a json file of multiple errors in multiple files
    let input_file =
        fs::read_to_string("../Data/example.json").expect("Read json file to string");

    let input_json: serde_json::Map<String, Value> =
        serde_json::from_str(input_file.as_str().trim()).expect("Valid json");

    // Iterate through all fields in the json
    for (_, value) in input_json.iter() {
        let file_errors: Vec<CompileJsonOutput> =
            serde_json::from_value(value.clone()).expect("Valid json");

        for file_error in file_errors {
            let model = ai::Model::MiniOrca;
            let output_json = file_error.clone();
            let file_contents =
                fs::read_to_string(file_error.filepath).expect("Compiler error json has legitimate path");

            let input = ai::CompileFix {
                model,
                output_json,
                file_contents,
            };

            let result = system::run_job("CompileFix", input);
            println!("result: {}", result);
        }
    }

    Ok(())
}
