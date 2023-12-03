use std::{cell::RefCell, collections::HashMap};

use anyhow::{anyhow, Result};
use serde_json::{json, Map, Value};

use crate::{parser};

pub trait Node {
    fn execute(&self, input: serde_json::Value, node_map: &NodeMap) -> Result<Value>;
}

pub type NodeMap = HashMap<String, Box<dyn Node>>;

// Input node ------------------

#[derive(Debug)]
pub struct InputNode {
    pub points_to: String,
}

impl Node for InputNode {
    fn execute(&self, input: Value, node_map: &NodeMap) -> Result<Value> {
        let node = node_map
            .get(&self.points_to)
            .ok_or(anyhow!("Could not find input in table"))?;
        node.execute(input, node_map)
    }
}

// Task Node -------------------

#[derive(Debug)]
pub struct TaskNode {
    pub command: String,
    pub points_to: Option<String>,
}

impl Node for TaskNode {
    fn execute(&self, _input: Value, _node_map: &NodeMap) -> Result<Value> {
        Ok(json!("TaskNode"))
    }
}

// If Node -----------------------

#[derive(Debug)]
pub struct IfNode {
    pub condition: String,
    pub true_branch: String,
    pub false_branch: String,
}

impl Node for IfNode {
    fn execute(&self, input: Value, node_map: &NodeMap) -> Result<Value> {
        // TODO: Show errors
        let bool_result =
            parser::conditional::evaluate_if_statement(self.condition.clone(), &input)
                .map_err(|e| anyhow!("Could not evaluate conditional {}", e))?;

        if bool_result {
            let node = node_map
                .get(self.true_branch.as_str())
                .ok_or(anyhow!("Could not get node in table"))?;
            node.execute(input, node_map)
        } else {
            let node = node_map
                .get(self.false_branch.as_str())
                .ok_or(anyhow!("Could not find node in table"))?;
            node.execute(input, node_map)
        }
    }
}

// Count Node --------------------

#[derive(Debug)]
pub struct CountNode {
    pub count: RefCell<usize>,
    pub points_to: String,
}

impl Node for CountNode {
    fn execute(&self, input: Value, node_map: &NodeMap) -> Result<Value> {
        let next = node_map.get(&self.points_to).unwrap();
        self.count.replace_with(|&mut x| x + 1);
        // Merge the count into the json
        let mut binding = input.clone();
        let new_input = match binding.as_object_mut() {
            Some(map) => map,
            None => return Err(anyhow!("Input is not an object")),
        };
        new_input.insert(
            "__count".to_owned(),
            serde_json::Value::from(*self.count.borrow()),
        );

        next.execute(new_input.clone().into(), node_map)
    }
}

impl CountNode {
    pub fn new(points_to: String) -> CountNode {
        CountNode {
            count: RefCell::new(0),
            points_to,
        }
    }
}

// Multi Node ---------------------

#[derive(Debug)]
pub struct MultiNode {
    pub run_before: Vec<String>,
    pub points_to: String,
}

impl Node for MultiNode {
    fn execute(&self, input: Value, node_map: &NodeMap) -> Result<Value> {
        let mut results = Vec::new();
        for node_name in &self.run_before {
            let node = node_map.get(node_name).unwrap();
            let result = node.execute(input.clone(), node_map)?;
            results.push(result);
        }

        // Merge all results into one json value
        let mut merged = match input.as_object() {
            Some(map) => map.clone(),
            None => serde_json::Map::new(),
        };

        for result in results {
            let result = match result.as_object() {
                Some(map) => map,
                None => return Err(anyhow!("Result is not an object")),
            };
            for (key, value) in result {
                merged.insert(key.clone(), value.clone());
            }
        }

        let node = node_map.get(&self.points_to).unwrap();
        node.execute(merged.into(), node_map)
    }
}

// Switch Node --------------------
#[derive(Debug)]
pub struct SwitchNode {
    pub field: String,
    pub cases_to: Vec<(Value, String)>,
    pub default_to: Option<String>,
}

impl Node for SwitchNode {
    fn execute(&self, input: Value, node_map: &NodeMap) -> Result<Value> {
        let to_compare = input
            .get(self.field.clone())
            .ok_or(anyhow!("Could not read field for match statement"))?;
        for case in self.cases_to.as_slice() {
            let (test, points_to) = case;
            if *to_compare == *test {
                let node = node_map.get(&points_to.clone()).unwrap();
                return node.execute(input, node_map);
            }
        }

        match self.default_to {
            Some(ref points_to) => {
                let node = node_map.get(points_to).unwrap();
                node.execute(input, node_map)
            }
            None => Ok(input),
        }
    }
}

// Switch Node --------------------
#[derive(Debug)]
pub struct MatchNode {
    pub field: String,
    pub cases_to: Vec<(Value, String)>,
    pub default_to: Option<String>,
}

impl Node for MatchNode {
    fn execute(&self, input: Value, node_map: &NodeMap) -> Result<Value> {
        let mut result = input.clone();
        let to_compare = result
            .get(self.field.clone())
            .ok_or(anyhow!("Could not read field for match statement"))?;
        for case in self.cases_to.as_slice() {
            let (test, points_to) = case;
            if *to_compare == *test {
                let node = node_map.get(&points_to.clone()).unwrap();
                result = node.execute(input.clone(), node_map)?;
                break;
            }
        }

        match self.default_to {
            Some(ref points_to) => {
                let node = node_map.get(points_to).unwrap();
                node.execute(result, node_map)
            }
            None => Ok(result),
        }
    }
}

pub struct AddFieldNode {
    pub label: String,
    pub points_to: Option<String>,
}

impl Node for AddFieldNode {
    fn execute(&self, input: Value, node_map: &NodeMap) -> Result<Value> {
        let mut new_input = input.clone();
        let mut default = Map::new();
        let new_input = match new_input.as_object_mut() {
            Some(map) => map,
            None => &mut default,
        };

        // Split label on ":" and set the values
        let mut split = self.label.split(':');
        let key = split
            .next()
            .ok_or_else(|| anyhow!("Could not get key from field node"))?;
        let value = split.next().unwrap_or("null");

        let value = crate::transform::safe_parse_to_value(value.trim());

        new_input.insert(key.to_owned().trim().to_owned(), value);

        if let Some(ref points_to) = self.points_to {
            let node = node_map.get(points_to).unwrap();
            node.execute(new_input.clone().into(), node_map)
        } else {
            Ok(new_input.clone().into())
        }
    }
}
