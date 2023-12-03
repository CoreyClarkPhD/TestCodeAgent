use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum JobType {
    Test,
    Compile,
    CompileFix,
}
