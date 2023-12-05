use anyhow::Result;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;

use crate::system::job_core::Job;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappedJsonError {
    column: Number,
    line: Number,
    pub filepath: String,
    message: String,
    snippet: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Location {
    file: String,
    line: i32,
    column: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LocationPair {
    caret: Location,
    finish: Option<Location>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorKind {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClangOutputJson {
    kind: ErrorKind,
    message: String,
    locations: Vec<LocationPair>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompileJob {
    pub files: Vec<PathBuf>,
}

impl Job for CompileJob {
    fn run(&self) -> Result<serde_json::Value> {
        self.compile().map(|output| {
            serde_json::to_value(output).expect("Struct with deserialize to deseralize")
        })
    }
}

impl CompileJob {
    pub fn compile(&self) -> Result<Vec<ClangOutputJson>> {
        let joined_files: Vec<_> = self.files.iter().map(|p| p.to_string_lossy()).collect();
        let joined_files = joined_files.join(" ");

        let command = format!(
            "g++-13 -std=c++17 {} -fdiagnostics-format=json",
            joined_files
        );

        let command_output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        let output = String::from_utf8(command_output.stderr)?;

        let output = output
            .lines()
            .into_iter()
            .map(|text| {
                let json: Vec<ClangOutputJson> = serde_json::from_str(text).unwrap();
                json
            })
            .flatten()
            .collect();

        Ok(output)
    }
}
