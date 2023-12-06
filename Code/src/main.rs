use ai::{FixCodeJob, FixCodeResult};
use clap::Parser;
use dotenv::dotenv;
use git::check_unsaved_files;
use indicatif::ProgressBar;
use std::{env, fs, path::PathBuf, time::Duration};
use system::types::JobType;
use ui::{tweak_code, MenuOption};

use anyhow::Result;

use fs_prompt::get_flowscript_compile;

use crate::{
    compiler::CompileJob,
    fs_prompt::save_flowscript,
    output::MappedJsonError,
    ui::{prompt_options, render_fix_code_result},
};

mod ai; // Sends requests to ChatGPT
mod compiler; // Compiles provides c++ source code
mod files; // Utility for default file input
mod flowscript; // Parse and execute Flowscript
mod fs_prompt; // Asks ChatGPT to write Flowscript
mod git; // Checks to make sure there are no uncommitted changes
mod output; // Maps the g++ error json shape to the desired shape
mod system; // Job System and C++ bindings
mod ui; // Renders console output

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(num_args = 1.., value_delimiter = ' ', help = "Input files")]
    files: Vec<PathBuf>,

    #[arg(short, long, help="Reprompts ChatGPT to get a new flowscript file", default_value = "false")]
    reprompt_flowscript: bool,

    #[arg(short, long, name = "Directory", help="Compile all C++ files in directory", default_value = ".")]
    directory: PathBuf,

    #[arg(short, long, name = "OpenAI API Key", help="Your OpenAI API Key")]
    api_key: Option<String>,

    #[arg(short, long, name = "Fix warnings", default_value = "false")]
    fix_warnings: bool,

    #[arg(long, name = "Allow dirty", help = "Allows running agent with uncommitted files", default_value = "false")]
    allow_dirty: bool,
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
    if !args.allow_dirty {
        if check_unsaved_files(&args.directory) {
            println!("Uncommitted files found. Please commit, discard or stash them before running the code agent, or run with --allow-dirty");
            return Ok(());
        }
    }

    let mut spinner = Option::None;
    if args.reprompt_flowscript {
        spinner = Some(ProgressBar::new_spinner());
        spinner
            .as_mut()
            .unwrap()
            .enable_steady_tick(Duration::from_millis(100));
        spinner
            .as_mut()
            .unwrap()
            .set_message("Getting flowscript...");
    }

    let Ok(script) = get_flowscript_compile(args.reprompt_flowscript) else {
        println!("Error getting flowscript");
        return Ok(());
    };

    if let Some(spinner) = spinner {
        spinner.finish_and_clear();
    }

    if args.reprompt_flowscript {
        save_flowscript(&script)?;
    }

    system::create_worker_thread();

    // Check that file_paths are cpp files
    for path in &file_paths {
        if path.extension().unwrap_or_default() != "cpp" {
            println!("Error: {} is not a cpp file", path.to_string_lossy());
            return Ok(());
        }
    }

    loop {
        let spin = ProgressBar::new_spinner();
        spin.enable_steady_tick(Duration::from_millis(100));
        spin.set_message("Compiling");
        let result = flowscript::execute_flowscript(
            &script,
            CompileJob {
                files: file_paths.clone(),
                fix_warnings: args.fix_warnings,
            },
        )?;
        spin.finish_and_clear();

        let errors: Vec<MappedJsonError> = serde_json::from_value(result)?;

        if errors.len() == 0 {
            println!("No errors found :)");
            break;
        }

        spin.finish_with_message(format!("Errors found: {}", errors.len()));

        let first_error = errors.get(0).expect("Vec has an error");

        // Restart spin
        let spin = ProgressBar::new_spinner();
        spin.enable_steady_tick(Duration::from_millis(100));
        spin.set_message(format!(
            "Asking ChatGPT to fix first error.... ({})",
            first_error.message.trim()
        ));

        let fix = FixCodeJob {
            model: ai::Model::ChatGpt,
            output_json: first_error.clone(),
            file_contents: fs::read_to_string(&first_error.filepath)?,
        };

        let Ok(result) =
            serde_json::from_value::<FixCodeResult>(system::run_job(JobType::FixCode, fix))
        else {
            println!("Error getting  code result");
            return Ok(());
        };

        spin.finish_and_clear();
        render_fix_code_result(&result);
        let choice = prompt_options();

        match choice {
            MenuOption::Quit => break,
            MenuOption::Tweak => {
                if let Some(new_code) = tweak_code(&result.code) {
                    files::replace_code(&first_error.filepath, new_code);
                } else {
                    break;
                }
            }
            MenuOption::Accept => {
                files::replace_code(&first_error.filepath, result.code);
            }
        };
    }

    Ok(())
}
