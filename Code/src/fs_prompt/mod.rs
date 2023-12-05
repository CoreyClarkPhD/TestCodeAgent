use std::{env, fs, vec};

use anyhow::Result;

use crate::ai::{make_ai_request, Message, Model, Role};

pub fn get_flowscript_compile(reprompt: bool) -> Result<String> {
    if reprompt {
        get_flowscript_from_gpt()
    } else if let Some(flowscript) = get_saved_flowscript() {
        Ok(flowscript)
    } else {
        get_flowscript_from_gpt()
    }
}

pub fn save_flowscript(prompt: &str) -> Result<()> {
    if let Ok(home) = env::var("HOME") {
        let path = format!("{}/.fsprompt", home);
        fs::write(path, prompt)?;
    }
    Ok(())
}

fn get_flowscript_from_gpt() -> Result<String> {
    let prompt = get_prompt();

    let response = make_ai_request(&prompt, &Model::ChatGpt)?;

    // Get the first choice
    let choice = response.choices.first().unwrap();

    // Get the content from the message
    let content = choice.message.content.clone();

    Ok(content)
}

fn get_prompt() -> Vec<Message> {
    let mut result: Vec<Message> = vec::Vec::new();

    result.push(Message {
        role: Role::System,
        content: include_str!("./prompt.txt").to_string(),
    });

    result
}

fn get_saved_flowscript() -> Option<String> {
    // Check for .fsprompt in home directory
    // If it exists, return the contents
    // If it doesn't exist, return None

    if let Ok(home) = env::var("HOME") {
        let path = format!("{}/.fsprompt", home);
        if let Ok(contents) = fs::read_to_string(path) {
            if !contents.is_empty() {
                return Some(contents);
            }
        }
    }
    None
}
