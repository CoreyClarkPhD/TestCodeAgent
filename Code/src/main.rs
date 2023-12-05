use clap::Parser;
use dotenv::dotenv;
use git::check_unsaved_files;
use std::{env, path::PathBuf};

use anyhow::Result;

use fs_prompt::get_flowscript_compile;

use crate::{compiler::CompileJob, fs_prompt::save_flowscript, output::MappedJsonError};

mod ai;
mod compiler;
mod files;
mod flowscript;
mod fs_prompt;
mod git;
mod output;
mod system;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(num_args = 1.., value_delimiter = ' ', help = "Input files")]
    files: Vec<PathBuf>,

    #[arg(short, long, default_value = "false")]
    reprompt_flowscript: bool,

    #[arg(short, long, name = "Directory", default_value = ".")]
    directory: PathBuf,

    #[arg(short, long, name = "OpenAI API Key")]
    api_key: Option<String>,

    #[arg(short, long, name = "Fix warnings", default_value = "false")]
    fix_warnings: bool,
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

    // Check for unsaved files
    // TODO: Condense
    if check_unsaved_files(&args.directory) {
        println!("Uncommitted files found. Please commit, discard or stash them before running the code agent.");
        return Ok(());
    }

    let Ok(script) = get_flowscript_compile(args.reprompt_flowscript) else {
        println!("Error getting flowscript");
        return Ok(());
    };

    if args.reprompt_flowscript {
        save_flowscript(&script)?;
    }

    system::create_worker_thread();

    let result = flowscript::execute_flowscript(
        &script,
        CompileJob {
            files: file_paths.clone(),
            fix_warnings: args.fix_warnings,
        },
    )?;

    let errors: Vec<MappedJsonError> = serde_json::from_value(result)?;

    Ok(())
}
