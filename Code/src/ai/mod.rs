mod prompts;
use std::env;
use std::io::prelude::*;

use std::fs::OpenOptions;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};

use crate::ai::prompts::get_chat_gpt_prompt;
use crate::ai::prompts::get_mini_orca_prompt;
use crate::ai::prompts::get_mistral_prompt;
use crate::compiler::CompileJsonOutput;
use crate::system::job_core::Job;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Model {
    ChatGpt,
    Mistral,
    MiniOrca,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub finish_reason: String,
    pub index: i32,
    message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    choices: Vec<Choice>,
    created: i32,
    id: String,
    model: String,
    object: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiInput {
    model: String,
    max_tokens: i32,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompileFix {
    pub model: Model,
    pub output_json: CompileJsonOutput,
    pub file_contents: String,
}

pub fn get_url_from_model(model: &Model) -> String {
    match model {
        Model::ChatGpt => "https://api.openai.com".to_string(),
        Model::Mistral => "http://localhost:4891".to_string(),
        Model::MiniOrca => "http://localhost:4891".to_string(),
    }
}

pub fn get_api_model_from_model(model: &Model) -> String {
    match model {
        Model::ChatGpt => "gpt-4-1106-preview".to_string(),
        Model::Mistral => "mistral".to_string(),
        Model::MiniOrca => "miniorca".to_string(),
    }
}

impl Job for CompileFix {
    fn run(&self) -> Result<Value> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(800))
            .build()?;
        let url = format!("{}/v1/chat/completions", get_url_from_model(&self.model));
        let prompt = match self.model {
            Model::ChatGpt => get_chat_gpt_prompt(&self.output_json, &self.file_contents),
            Model::Mistral => get_mistral_prompt(&self.output_json, &self.file_contents),
            Model::MiniOrca => get_mini_orca_prompt(&self.output_json, &self.file_contents),
        };
        let input = ApiInput {
            model: get_api_model_from_model(&self.model),
            max_tokens: 800,
            messages: prompt.clone(),
        };

        let auth_token = env::var("OPENAI_TOKEN")?;

        let response = client
            .post(url.as_str())
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&input)?)
            .send()?;

        match response.status() {
            reqwest::StatusCode::OK => (),
            _ => {
                println!("response: {:?}", response);
                let response_text = response.text().expect("get response text");
                println!("response text: {}", response_text);
                return Ok(json!(null));
            }
        }

        let response: ApiResponse =
            serde_json::from_str(response.text().expect("got response text").as_str())?;

        // Get the first choice
        let choice = response.choices.first().expect("Get first choice");

        // Get the content from the message
        let content = choice.message.content.clone();
        println!("content: {}", content);
        println!("finish reason: {}", choice.finish_reason);

        let (code, explain) = extract_response_code(&content);

        write_file_output(&content, &prompt);

        Ok(json!({"code": code, "explanation": explain}))
    }
}

fn extract_response_code(response: &str) -> (String, String) {
    let mut response_code = String::new();
    let mut explanation = String::new();
    let mut in_code_block = false;

    for line in response.lines() {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            response_code.push_str(line);
            response_code.push('\n');
        } else {
            explanation.push_str(line);
            explanation.push('\n');
        }
    }

    (response_code, explanation)
}

fn get_attempt_count() -> i32 {
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("attempt_count.txt")
        .unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_default();

    // Increase
    let attempt_count = contents.parse::<i32>().unwrap_or_default() + 1;
    // Replace contents
    file.set_len(0).unwrap();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    file.write_all(attempt_count.to_string().as_bytes())
        .unwrap();

    attempt_count
}

fn write_file_output(response: &str, prompt: &Vec<Message>) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("prompt_history.md")
        .unwrap();

    let attempt_count = get_attempt_count();

    let _ = writeln!(file, "## Attempt: {}", attempt_count);

    let _ = writeln!(
        file,
        "Prompt: \n ```json\n{}\n```",
        serde_json::to_string_pretty(prompt).unwrap()
    );

    let _ = writeln!(file, "Response: \n{}", response);
    let _ = writeln!(file);
    let _ = writeln!(file, "_Adjustment: _");
    let _ = writeln!(file);
    let _ = writeln!(file, "---");
}
