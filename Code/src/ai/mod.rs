mod prompts;
use std::env;

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

fn make_ai_request(prompt: &Vec<Message>, model: &Model) -> Result<ApiResponse> {
    let url = format!("{}/v1/chat/completions", get_url_from_model(model));
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(800))
        .build()?;
    let input = ApiInput {
        model: get_api_model_from_model(model),
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
            let response_text = response.text()?;
            return Err(anyhow::anyhow!("Error: {}", response_text));
        }
    };

    serde_json::from_str(response.text()?.as_str())
        .map_err(|e| anyhow::anyhow!("Error getting response: {}", e))
}

impl Job for CompileFix {
    fn run(&self) -> Result<Value> {
        let prompt = match self.model {
            Model::ChatGpt => get_chat_gpt_prompt(&self.output_json, &self.file_contents),
            Model::Mistral => get_mistral_prompt(&self.output_json, &self.file_contents),
            Model::MiniOrca => get_mini_orca_prompt(&self.output_json, &self.file_contents),
        };

        let response = make_ai_request(&prompt, &self.model)?;

        // Get the first choice
        let choice = response.choices.first().unwrap();

        // Get the content from the message
        let content = choice.message.content.clone();
        println!("content: {}", content);
        println!("finish reason: {}", choice.finish_reason);

        let (code, explain) = extract_response_code(&content);

        Ok(json!({"code": code, "explanation": explain}))
    }
}

// TODO: FIX
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
