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
