use anyhow::Result;
use serde_json::Value;

use crate::flowscript::{parser::extract_definitions, transform::TransformError};

mod nodes;
mod parser;
mod transform;

pub fn execute_flowscript(script: &String, input: Value) -> Result<Value> {
    let defs = match extract_definitions(script) {
        Ok(defs) => defs,
        Err(e) => {
            println!("{}", e);
            return Err(anyhow::anyhow!("Error parsing file"));
        }
    };

    let graph = match transform::defs_to_graph(defs) {
        Ok(graph) => graph,
        Err(e) => {
            match &e {
                TransformError::NoConnection(f) => {
                    println!("Error: No connection for {}", f)
                }
            };
            return Err(anyhow::anyhow!("Error rectifying connections"));
        }
    };

    let Some(input) = graph.get("input") else {
        println!("Error: No input node");
        return Err(anyhow::anyhow!("No input node"));
    };

    let result = input.execute(input, &graph)?;
    println!("{}", result);

    todo!()
}
