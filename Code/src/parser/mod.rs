pub mod conditional;
mod create_error;
use std::collections::HashMap;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./grammar/grammar.pest"]
pub struct FlowscriptParser;

#[derive(Debug)]
pub enum NodeDef {
    Input,               // Done
    Task,                // Done
    IfStatement(String), // Done
    Count,               // Done
    Multi,               // Done
    Switch(String),      // Done
    Match(String),
    Setter(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConnectionType {
    Default,
    IfResult(bool), // If the label says "true" or "false"
    SwitchBranch(String),
    MatchBranch(String),
    MultiOut,
}

#[derive(Debug, Clone)]
pub struct ConnectionDef {
    pub from: String,
    pub to: String,
    pub c_type: ConnectionType,
}

#[derive(Debug)]
pub struct Defs {
    pub variables: HashMap<String, NodeDef>,
    pub connections: Vec<ConnectionDef>,
}

fn process_attribute_pair(pair: Pair<'_, Rule>) -> (String, String) {
    if pair.as_rule() != Rule::attribute {
        panic!("Error: Expected attributes");
    };

    let mut key = String::new();
    let mut value = String::new();

    pair.into_inner().for_each(|pair| match pair.as_rule() {
        Rule::attribute_name => {
            key = pair.as_str().to_owned();
        }
        Rule::attribute_value => {
            value = pair.as_str().to_owned();
        }
        _ => {}
    });
    (key, value)
}

fn get_attributes(pair: Pair<'_, Rule>) -> HashMap<String, String> {
    // TODO: Fix typesafety?
    if pair.as_rule() != Rule::attributes {
        panic!("Error: Expected attributes");
    };

    let mut attributes_map = HashMap::new();

    pair.into_inner().for_each(|pair| match pair.as_rule() {
        Rule::attribute => {
            let attr = process_attribute_pair(pair);
            attributes_map.insert(attr.0, attr.1);
        }
        _ => {}
    });
    attributes_map
}

fn process_var_def(pair: Pair<Rule>) -> Result<(String, NodeDef), pest::error::Error<Rule>> {
    if pair.as_rule() != Rule::variable_def {
        panic!("Error: Expected variable_def"); // Should never happen
    };

    let mut name = String::new();

    let mut attributes: Option<HashMap<String, String>> = None;

    for single in pair.clone().into_inner() {
        match single.as_rule() {
            Rule::variable => {
                name = single.as_str().to_owned();
            }
            Rule::attributes => {
                attributes = Some(get_attributes(single));
            }
            _ => {}
        }
    }

    if name == "input" {
        return Ok((name, NodeDef::Input));
    }

    match attributes {
        Some(attrs) => {
            // Other node types must have a shape
            if !attrs.contains_key("shape") {
                return Err(create_error::build_pest_error(
                    pair,
                    "Non-task node requires a shape definition",
                ));
            }
            let shape = attrs.get("shape").unwrap();

            match shape.as_str() {
                "rectangle" => {
                    let Some(condition) = attrs.get("label") else {
                        return Err(create_error::build_pest_error(
                            pair,
                            "If statement requires a condition",
                        ));
                    };
                    return Ok((name, NodeDef::IfStatement(condition.to_owned())));
                }
                "component" => return Ok((name, NodeDef::Count)),

                "diamond" => {
                    let Some(condition) = attrs.get("label") else {
                        return Err(create_error::build_pest_error(
                            pair,
                            "Switch statement requires a field to switch on",
                        ));
                    };
                    return Ok((name, NodeDef::Switch(condition.to_owned())));
                }

                "Mdiamond" => {
                    let Some(label) = attrs.get("label") else {
                        return Err(create_error::build_pest_error(
                            pair,
                            "Match statement requires a field to match with",
                        ));
                    };
                    return Ok((name, NodeDef::Match(label.to_owned())));
                }

                "point" => return Ok((name, NodeDef::Multi)),

                "cds" => {
                    let Some(label) = attrs.get("label") else {
                        return Err(create_error::build_pest_error(
                            pair,
                            "Setter requires a field to set",
                        ));
                    };
                    return Ok((name, NodeDef::Setter(label.to_owned())));
                }
                _ => {}
            }
        }
        None => {
            return Ok((name, NodeDef::Task));
        }
    };
    Err(create_error::build_pest_error(pair, "Unknown node type"))
}

fn process_connection_def(pair: Pair<Rule>) -> Result<ConnectionDef, pest::error::Error<Rule>> {
    if pair.as_rule() != Rule::connection_def {
        panic!("Expected connection definition") // Should never happen
    }

    let variable_mentions: Vec<Pair<Rule>> = pair
        .clone()
        .into_inner()
        .flatten()
        .filter(|p| p.as_rule() == Rule::variable)
        .collect();

    if variable_mentions.len() != 2 {
        return Err(create_error::build_pest_error(
            pair,
            "Connection definition must have exactly two variables",
        ));
    }

    let from = variable_mentions[0].as_str().to_owned();
    let to = variable_mentions[1].as_str().to_owned();

    let has_attributes = pair
        .clone()
        .into_inner()
        .flatten()
        .find(|p| p.as_rule() == Rule::attributes);

    match has_attributes {
        Some(attrs) => {
            let attr_map = get_attributes(attrs);
            let maybe_label = attr_map.get("label");

            match maybe_label {
                Some(label) => {
                    if label == "true" {
                        // TODO: Make sure not dashed
                        return Ok(ConnectionDef {
                            from,
                            to,
                            c_type: ConnectionType::IfResult(true),
                        });
                    } else if label == "false" {
                        // TODO: Make sure not dashed
                        return Ok(ConnectionDef {
                            from,
                            to,
                            c_type: ConnectionType::IfResult(false),
                        });
                    }

                    let is_dashed = attr_map.get("style").unwrap_or(&"".to_owned()) == "dashed";
                    // If it is dashed, its a match branch
                    if is_dashed {
                        return Ok(ConnectionDef {
                            from,
                            to,
                            c_type: ConnectionType::MatchBranch(label.to_owned()),
                        });
                    } else {
                        return Ok(ConnectionDef {
                            from,
                            to,
                            c_type: ConnectionType::SwitchBranch(label.to_owned()),
                        });
                    }
                }

                // Must be a multi or a default
                None => {
                    let style = attr_map.get("style");
                    // If style is "dashed" return multi
                    match style {
                        Some(s) => {
                            if s == "dashed" {
                                return Ok(ConnectionDef {
                                    from,
                                    to,
                                    c_type: ConnectionType::MultiOut,
                                });
                            }
                        }
                        None => {
                            return Ok(ConnectionDef {
                                from,
                                to,
                                c_type: ConnectionType::Default,
                            });
                        }
                    }
                }
            }
        }
        None => {
            return Ok(ConnectionDef {
                from,
                to,
                c_type: ConnectionType::Default,
            })
        }
    };

    Err(create_error::build_pest_error(
        pair,
        "Unknown connection type",
    ))
}

pub fn extract_definitions(filepath: String) -> Result<Defs, pest::error::Error<Rule>> {
    let mut vars = HashMap::new();
    let mut conns = Vec::new();
    let Ok(contents) = std::fs::read_to_string(filepath) else {
        panic!("Error: Could not read file");
    };

    let parse = FlowscriptParser::parse(Rule::program, &contents)?;

    let defs: Vec<Pair<Rule>> = parse
        .flatten()
        .filter(|pair| {
            pair.as_rule() == Rule::variable_def || pair.as_rule() == Rule::connection_def
        })
        .collect();

    for def in defs {
        match def.as_rule() {
            Rule::variable_def => {
                let var_def = process_var_def(def)?;
                vars.insert(var_def.0, var_def.1);
            }
            Rule::connection_def => {
                let con_def = process_connection_def(def)?;
                conns.push(con_def);
            }
            _ => {}
        }
    }

    // If input isn't included in vars, add it
    if !vars.contains_key("input") {
        vars.insert("input".to_owned(), NodeDef::Input);
    }

    for conn in conns.iter() {
        let from = conn.from.clone();
        if !vars.contains_key(&from) {
            vars.insert(from.clone(), NodeDef::Task);
        }
        let to = conn.to.clone();
        if !vars.contains_key(&to) {
            vars.insert(to.clone(), NodeDef::Task);
        }
    }

    Ok(Defs {
        variables: vars,
        connections: conns,
    })
}
