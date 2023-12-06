mod prompts;
use std::env;

use std::time::Duration;

use crate::ai::prompts::get_chat_gpt_prompt;
use crate::ai::prompts::get_mini_orca_prompt;
use crate::ai::prompts::get_mistral_prompt;
use crate::output::MappedJsonError;
use crate::system::job_core::Job;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

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
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub choices: Vec<Choice>,
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

fn get_url_from_model(model: &Model) -> String {
    match model {
        Model::ChatGpt => "https://api.openai.com".to_string(),
        Model::Mistral => "http://localhost:4891".to_string(),
        Model::MiniOrca => "http://localhost:4891".to_string(),
    }
}

fn get_api_model_from_model(model: &Model) -> String {
    match model {
        Model::ChatGpt => "gpt-4-1106-preview".to_string(),
        Model::Mistral => "mistral".to_string(),
        Model::MiniOrca => "miniorca".to_string(),
    }
}

pub fn make_ai_request(prompt: &Vec<Message>, model: &Model) -> Result<ApiResponse> {
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

// Compiler fixing

#[derive(Serialize, Deserialize, Debug)]
pub struct FixCodeJob {
    pub model: Model,
    pub output_json: MappedJsonError,
    pub file_contents: String,
}

impl Job for FixCodeJob {
    fn run(&self) -> Result<Value> {
        self.fix_code()
            .map(|output| serde_json::to_value(output).expect("Output type is deserializable"))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixCodeResult {
    pub code: String,
    pub explanation: String,
}

impl FixCodeJob {
    pub fn fix_code(&self) -> Result<FixCodeResult> {
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

        let (code, explain) = extract_response_code(&content);

        Ok(FixCodeResult {
            code: code.to_string(),
            explanation: explain.to_string(),
        })
    }
}

pub fn extract_response_code(response: &str) -> (String, String) {
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
