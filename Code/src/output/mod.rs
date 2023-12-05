use serde::{Serialize, Deserialize};

use crate::compiler::ClangOutputJson;

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
    errors: Vec<ClangOutputJson>
}
