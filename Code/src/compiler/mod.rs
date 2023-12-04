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
pub struct ClangOutputJson {}

pub struct CompileJob {
    files: Vec<PathBuf>,
}

impl Job for CompileJob {
    fn run(&self) -> Result<serde_json::Value> {
        match self.compile() {
            Ok(output) => Ok(serde_json::to_value(output)?),
            Err(e) => Err(e),
        }
    }
}

impl CompileJob {
    pub fn compile(&self) -> Result<ClangOutputJson> {
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
            .filter(|line| line.starts_with('{'))
            .collect::<Vec<_>>()
            .join("");

        println!("{}", output);

        Ok(ClangOutputJson{})
    }
}
