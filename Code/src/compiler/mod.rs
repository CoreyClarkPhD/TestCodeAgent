use anyhow::Result;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;

use crate::system::job_core::Job;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompileJsonOutput {
    column: Number,
    line: Number,
    pub filepath: String,
    message: String,
    snippet: String,
}

pub enum ActualCompileError {}

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
    pub fn new(files: Vec<PathBuf>) -> Self {
        Self { files }
    }

    pub fn compile(&self) -> Result<CompileJsonOutput> {
        let mut output = CompileJsonOutput {
            column: 0.into(),
            line: 0.into(),
            filepath: "".to_string(),
            message: "".to_string(),
            snippet: "".to_string(),
        };

        Ok(output)
    }
}
