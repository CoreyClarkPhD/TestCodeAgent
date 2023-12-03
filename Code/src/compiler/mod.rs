use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;

#[derive(Serialize, Deserialize, Debug, Clone )]
pub struct CompileJsonOutput {
    column: Number,
    line: Number,
    pub filepath: String,
    message: String,
    snippet: String,
}

pub enum ActualCompileError {

}

pub fn compile_files(files: &Vec<PathBuf>) -> Result<Vec<CompileJsonOutput>, ActualCompileError> {




    Ok(vec![])
}
