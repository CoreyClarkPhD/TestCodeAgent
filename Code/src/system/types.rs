use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum JobType {
    Compile,
    Output,
    FixCode,
}
