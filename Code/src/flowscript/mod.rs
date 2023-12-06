use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::{flowscript::{parser::extract_definitions, transform::TransformError}, system::job_core};

mod nodes;
mod parser;
mod transform;

pub fn execute_flowscript<'a, T: job_core::Job + Serialize + Deserialize<'a>>(
    script: &String,
    input: T,
) -> Result<Value> {
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

    let Some(input_node) = graph.get("input") else {
        println!("Error: No input node");
        return Err(anyhow::anyhow!("No input node"));
    };

    let input_json = serde_json::to_value(input)?;
    input_node.execute(input_json, &graph)
}
