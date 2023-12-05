use anyhow::Result;
use serde_json::{from_value, Value};

use crate::{ai::FixCodeJob, compiler::CompileJob};

use super::types::JobType;

pub trait Job {
    fn run(&self) -> Result<Value>;
}

pub fn run_job(job_type: JobType, input: Value) -> Value {
    println!("Running job {:?}", job_type);
    let result = match job_type {
        JobType::FixCode => {
            let intoed: FixCodeJob = from_value(input).expect("Valid json");
            intoed.run()
        }
        JobType::Compile => {
            let intoed: CompileJob = from_value(input).expect("Valid json");
            intoed.run()
        }
        _ => panic!("Job not found"),
    };

    match result {
        Ok(response) => response,
        Err(e) => {
            println!("Error running job: {}", e);
            Value::Null
        }
    }
}
