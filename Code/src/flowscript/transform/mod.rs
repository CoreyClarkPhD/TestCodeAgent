use std::collections::HashMap;

use serde_json::Value;

use super::{parser::{ConnectionDef, ConnectionType, Defs, NodeDef}, nodes::{NodeMap, self, TaskNode, IfNode, CountNode, MultiNode}};

#[derive(Debug)]
pub enum TransformError {
    NoConnection(String),
}

pub fn get_point_to(name: &str, conns: &Vec<ConnectionDef>) -> Option<String> {
    let conn = conns
        .iter()
        .find(|conn| conn.from == name && conn.c_type == ConnectionType::Default);

    conn.map(|conn| conn.to.clone())
}

pub fn safe_parse_to_value(text: &str) -> Value {
    // If length is zero
    if text.is_empty() {
        return Value::Null;
    }
    // attempt to parse as a number, bool, undefined, null
    match serde_json::from_str(text) {
        Ok(v) => return v,
        Err(_) => {}
    };

    // parse as a string
    Value::String(text.to_owned())
}

pub fn defs_to_graph(defs: Defs) -> Result<NodeMap, TransformError> {
    let mut node_map: nodes::NodeMap = HashMap::new();
    let conns = defs.connections;

    for var_def in defs.variables {
        let name = var_def.0;
        let var = var_def.1;

        match var {
            NodeDef::Input => {
                let Some(points_to) = get_point_to(&name, &conns) else {
                    return Err(TransformError::NoConnection(name));
                };
                let node = nodes::InputNode { points_to };
                node_map.insert(name, Box::new(node));
            }

            NodeDef::Task => {
                let points_to = get_point_to(&name, &conns);

                let task = TaskNode {
                    command: name.clone(),
                    points_to,
                };
                node_map.insert(name, Box::new(task));
            }

            NodeDef::IfStatement(condition) => {
                let Some(true_point) = conns
                    .iter()
                    .find(|c| c.c_type == ConnectionType::IfResult(true) && c.from == name)
                else {
                    panic!("Error: If node {} has no true connection", name);
                };

                let Some(false_point) = conns
                    .iter()
                    .find(|c| c.c_type == ConnectionType::IfResult(false) && c.from == name)
                else {
                    panic!("Error: If node {} has no false connection", name);
                };

                let node = IfNode {
                    true_branch: true_point.to.clone(),
                    false_branch: false_point.to.clone(),
                    condition,
                };

                node_map.insert(name, Box::new(node));
            }

            NodeDef::Count => {
                let Some(points_to) = get_point_to(&name, &conns) else {
                    return Err(TransformError::NoConnection(name));
                };
                let node = CountNode::new(points_to);
                node_map.insert(name, Box::new(node));
            }

            NodeDef::Multi => {
                let before: Vec<String> = conns
                    .iter()
                    .filter(|conn| conn.from == name || conn.c_type == ConnectionType::MultiOut)
                    .map(|c| c.to.clone())
                    .collect();

                let Some(points_to) = get_point_to(&name, &conns) else {
                    return Err(TransformError::NoConnection(name));
                };

                let node = MultiNode {
                    run_before: before,
                    points_to,
                };

                node_map.insert(name, Box::new(node));
            }

            NodeDef::Switch(field) => {
                let cases: Vec<(Value, String)> = conns
                    .iter()
                    .filter(|conn| {
                        if conn.from != name {
                            return false;
                        }

                        match conn.c_type {
                            ConnectionType::SwitchBranch(_) => true,
                            _ => false,
                        }
                    })
                    .map(|conn| match conn.c_type.clone() {
                        ConnectionType::SwitchBranch(value) => {
                            (safe_parse_to_value(value.as_str()), conn.to.clone())
                        }
                        _ => panic!("Error: Switch node {} has non-switch connection", name),
                    })
                    .collect();

                let default_to = conns
                    .iter()
                    .find(|conn| conn.c_type == ConnectionType::Default && conn.from == name);

                let node = nodes::SwitchNode {
                    field,
                    cases_to: cases,
                    default_to: default_to.map(|c| c.to.clone()),
                };

                node_map.insert(name, Box::new(node));
            }

            NodeDef::Match(field) => {
                let cases: Vec<(Value, String)> = conns
                    .iter()
                    .filter(|conn| {
                        if conn.from != name {
                            return false;
                        }

                        match conn.c_type {
                            ConnectionType::MatchBranch(_) => true,
                            _ => false,
                        }
                    })
                    .map(|conn| match conn.c_type.clone() {
                        ConnectionType::MatchBranch(value) => {
                            (safe_parse_to_value(value.as_str()), conn.to.clone())
                        }
                        _ => panic!("Error: Switch node {} has non-switch connection", name),
                    })
                    .collect();

                let default_to = conns
                    .iter()
                    .find(|conn| conn.from == name && conn.c_type == ConnectionType::Default);

                let node = nodes::MatchNode {
                    field,
                    cases_to: cases,
                    default_to: default_to.map(|c| c.to.clone()),
                };


                node_map.insert(name, Box::new(node));
            }

            NodeDef::Setter(label) => {
                let points_to = get_point_to(&name, &conns);

                let node = nodes::AddFieldNode { label, points_to };

                node_map.insert(name, Box::new(node));
            }
        }
    }

    Ok(node_map)
}
