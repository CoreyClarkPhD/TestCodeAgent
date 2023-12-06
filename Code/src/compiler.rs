use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::system::job_core::Job;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    pub file: PathBuf,
    pub line: i32,
    pub column: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationPair {
    pub caret: Location,
    finish: Option<Location>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ErrorKind {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClangOutputJson {
    kind: ErrorKind,
    pub message: String,
    pub locations: Vec<LocationPair>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompileJob {
    pub files: Vec<PathBuf>,
    pub fix_warnings: bool,
}

impl Job for CompileJob {
    fn run(&self) -> Result<serde_json::Value> {
        self.compile().map(|output| json!({"errors": output}))
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

        println!("Running command: {}", command);

        let command_output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        let output = String::from_utf8(command_output.stderr)?;

        // Check if there are any errors
        if output.is_empty() {
            return Ok(Vec::new());
        }

        let output = output
            .lines()
            .flat_map(|text| {
                let json: Vec<ClangOutputJson> = serde_json::from_str(text)
                    .expect("g++ does not output valid json, do you have a main method?");
                json
            })
            .filter(|error| {
                if self.fix_warnings {
                    true
                } else {
                    error.kind == ErrorKind::Error
                }
            })
            .collect();

        Ok(output)
    }
}
