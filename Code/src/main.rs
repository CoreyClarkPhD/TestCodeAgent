use clap::Parser;
use dotenv::dotenv;
use std::{env, path::PathBuf};

use anyhow::Result;

mod ai;
mod compiler;
mod files;
mod nodes;
mod parser;
mod system;
mod transform;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(num_args = 1.., value_delimiter = ' ', help = "Input files")]
    files: Vec<PathBuf>, // Change to Vec<PathBuf> to accept multiple positional values

    #[arg(short, long, default_value = "false")]
    reprompt_flowscript: bool,

    #[arg(short, long, name = "Directory", default_value = ".")]
    directory: PathBuf,
}

fn main() -> Result<()> {
    let _ = dotenv();
    if let Err(e) = env::var("OPENAI_TOKEN") {
        // Load env file from home directory
        let home = env::var("HOME").expect("HOME not set");
        let env_file = format!("{}/.env", home);
        let _ = dotenv::from_path(env_file.as_str());

        println!("OPENAI_TOKEN not set");
        println!("Error: {}", e);
        println!("Please set OPENAI_TOKEN in .env");
        return Ok(());
    }

    let args = Args::parse();

    let mut file_paths = args.files;

    if file_paths.len() == 0 {
        // Use the directory flag instead
        file_paths = files::get_all_cpp_files_in_folder_path(&args.directory)?;
        if file_paths.len() == 0 {
            println!("No C++ files found in directory");
            return Ok(());
        }
    }

    println!("Files: {:?}", file_paths);


    // system::create_worker_thread();
    //
    // // Open a json file of multiple errors in multiple files
    // let input_file = fs::read_to_string("../Data/example.json").expect("Read json file to string");
    //
    // let input_json: serde_json::Map<String, Value> =
    //     serde_json::from_str(input_file.as_str().trim()).expect("Valid json");
    //
    // // Iterate through all fields in the json
    // for (_, value) in input_json.iter() {
    //     let file_errors: Vec<CompileJsonOutput> =
    //         serde_json::from_value(value.clone()).expect("Valid json");
    //
    //     for file_error in file_errors {
    //         let model = ai::Model::MiniOrca;
    //         let output_json = file_error.clone();
    //         let file_contents = fs::read_to_string(file_error.filepath)
    //             .expect("Compiler error json has legitimate path");
    //
    //         let input = ai::CompileFix {
    //             model,
    //             output_json,
    //             file_contents,
    //         };
    //
    //         let result = system::run_job("CompileFix", input);
    //         println!("result: {}", result);
    //     }
    // }

    Ok(())
}
