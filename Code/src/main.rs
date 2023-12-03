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
mod git;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(num_args = 1.., value_delimiter = ' ', help = "Input files")]
    files: Vec<PathBuf>, // Change to Vec<PathBuf> to accept multiple positional values

    #[arg(short, long, default_value = "false")]
    reprompt_flowscript: bool,

    #[arg(short, long, name = "Directory", default_value = ".")]
    directory: PathBuf,

    #[arg(short, long, name = "OpenAI API Key")]
    api_key: Option<String>,
}

fn main() -> Result<()> {
    let _ = dotenv();
    // Ensure API Token is set
    if let Err(e) = env::var("OPENAI_TOKEN") {
        if let Some(api_key) = Args::parse().api_key {
            env::set_var("OPENAI_TOKEN", api_key);
        } else {
            // Load env file from home directory
            let home = env::var("HOME").expect("HOME not set");
            let env_file = format!("{}/.env", home);
            let _ = dotenv::from_path(env_file.as_str());

            println!("OPENAI_TOKEN not set");
            println!("Error: {}", e);
            println!("Please set OPENAI_TOKEN in .env");
            return Ok(());
        }
    }

    let args = Args::parse();

    // Get C++ files
    let mut file_paths = args.files;
    if file_paths.is_empty() {
        // Use the directory flag instead
        file_paths = files::get_all_cpp_files_in_folder_path(&args.directory)?;
        if file_paths.is_empty() {
            println!("No C++ files found in directory");
            return Ok(());
        }
    }

    println!("Files: {:?}", file_paths);

    Ok(())
}
