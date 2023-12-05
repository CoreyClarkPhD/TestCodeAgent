use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{compiler::ClangOutputJson, system::job_core::Job};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappedJsonError {
    column: i32,
    line: i32,
    pub filepath: String,
    message: String,
    snippet: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputJob {
    errors: Vec<ClangOutputJson>,
}

impl Job for OutputJob {
    fn run(&self) -> Result<serde_json::Value> {
        self.map_output().map(|output| {
            serde_json::to_value(output).expect("Struct with deserialize to deseralize")
        })
    }
}

impl OutputJob {
    pub fn map_output(&self) -> Result<Vec<MappedJsonError>> {
        let mut mapped_errors = vec![];

        for error in &self.errors {
            let location = error.locations.get(0).ok_or(anyhow!("No location"))?;

            let mapped_error = MappedJsonError {
                column: location.caret.column,
                line: location.caret.line,
                filepath: location.caret.file.clone(),
                message: error.message.clone(),
                snippet: get_file_snippet(&location.caret.file, location.caret.line)?,
            };

            mapped_errors.push(mapped_error);
        }

        Ok(mapped_errors)
    }
}

fn get_file_snippet(filepath: &str, line: i32) -> Result<String> {
    let file_contents = std::fs::read_to_string(filepath)?;
    let lines: Vec<_> = file_contents.lines().collect();

    let mut snippet = String::new();

    let start_line = (line - 3).max(0) as usize;
    let end_line = (line + 3).min(lines.len() as i32) as usize;

    for (_, line) in lines
        .iter()
        .enumerate()
        .skip(start_line)
        .take(end_line - start_line)
    {
        snippet.push_str(&format!("{}\n", line));
    }

    Ok(snippet)
}
