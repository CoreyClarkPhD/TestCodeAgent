use std::{env, fs};

use anyhow::Result;
pub fn get_flowscript_compile(reprompt: bool) -> Result<String> {
    if reprompt {
        get_flowscript_from_gpt()
    } else {
        if let Some(flowscript) = get_saved_flowscript() {
            Ok(flowscript)
        } else {
            get_flowscript_from_gpt()
        }
    }
}

fn get_flowscript_from_gpt() -> Result<String> {
    todo!()
}

fn get_saved_flowscript() -> Option<String> {
    // Check for .fsprompt in home directory
    // If it exists, return the contents
    // If it doesn't exist, return None

    if let Ok(home) = env::var("HOME") {
        let path = format!("{}/.fsprompt", home);
        if let Ok(contents) = fs::read_to_string(path) {
            if contents.len() > 0 {
                return Some(contents);
            }
        }
    }
    None
}
