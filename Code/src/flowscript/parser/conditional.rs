use pest::{
    error::{self, Error},
    iterators::Pair,
    Parser,
};
use pest_derive::Parser;
use serde_json::Value;

#[derive(Parser)]
#[grammar = "./flowscript/grammar/condition.pest"]
pub struct ConditionParser;

fn build_pest_error(pair: Pair<Rule>, msg: &str) -> error::Error<Rule> {
    Error::new_from_span(
        error::ErrorVariant::CustomError {
            message: msg.to_string(),
        },
        pair.as_span(),
    )
}

fn parse_side(pair: Pair<Rule>, result: &serde_json::Value) -> Result<Value, Error<Rule>> {
    let Some(inner) = pair.clone().into_inner().next() else {
        return Err(build_pest_error(pair, "Could not get inner"));
    };
    match inner.as_rule() {
        Rule::number_value => {
            let val = serde_json::from_str::<Value>(inner.as_str())
                .map_err(|_| build_pest_error(pair, "Could not convert number"))?;
            Ok(val)
        }

        Rule::string_value => {
            let input = format!("\"{}\"", inner.as_str());
            let val = serde_json::from_str::<Value>(input.as_str())
                .map_err(|_| build_pest_error(pair, "Could not convert string"))?;
            Ok(val)
        }

        Rule::json_value => {
            let Some(val) = result.get(inner.as_str()) else {
                return Err(build_pest_error(pair, "Could not get value from result"));
            };
            Ok(val.clone())
        }
        _ => Err(build_pest_error(pair, "Could not match pair type")),
    }
}

fn parse_value_pair(pair: Pair<Rule>, result: Value) -> Result<(Value, Value), Error<Rule>> {
    let mut left: Value = Value::Null;
    let mut right: Value = Value::Null;

    for pair in pair.into_inner().flatten() {
        match pair.as_rule() {
            Rule::first => {
                left = parse_side(pair, &result)?;
            }

            Rule::second => {
                right = parse_side(pair, &result)?;
            }
            _ => {}
        };
    }
    Ok((left, right))
}

pub fn evaluate_if_statement(
    condition: String,
    job_result: &serde_json::Value,
) -> Result<bool, Error<Rule>> {
    let mut parsed = ConditionParser::parse(Rule::expression, &condition)?;

    let Some(condition) = parsed
        .clone()
        .flatten()
        .find(|pair| pair.as_rule() == Rule::condition)
    else {
        return Err(build_pest_error(
            parsed.next().expect("Has inside"),
            "Could not match condition",
        ));
    };

    let whole_thing = parsed.next().expect("Has inside");

    let possible_num_error = build_pest_error(whole_thing.clone(), "Could not parse a number");

    match condition
        .clone()
        .into_inner()
        .next()
        .expect("Has inner")
        .as_rule()
    {
        Rule::equal => {
            let (left, right) = parse_value_pair(whole_thing, job_result.clone())?;
            return Ok(left == right);
        }

        Rule::not_equal => {
            let (left, right) = parse_value_pair(whole_thing, job_result.clone())?;
            return Ok(left != right);
        }

        Rule::less_than => {
            let (left, right) = parse_value_pair(whole_thing, job_result.clone())?;
            let left = left.as_f64().ok_or(possible_num_error.clone())?;
            let right = right.as_f64().ok_or(possible_num_error.clone())?;
            return Ok(left < right);
        }

        Rule::less_than_eq_to => {
            let (left, right) = parse_value_pair(whole_thing, job_result.clone())?;
            let left = left.as_f64().ok_or(possible_num_error.clone())?;
            let right = right.as_f64().ok_or(possible_num_error.clone())?;
            return Ok(left <= right);
        }
        Rule::greater_than => {
            let (left, right) = parse_value_pair(whole_thing, job_result.clone())?;
            let left = left.as_f64().ok_or(possible_num_error.clone())?;
            let right = right.as_f64().ok_or(possible_num_error.clone())?;
            return Ok(left > right);
        }

        Rule::greater_than_eq_to => {
            let (left, right) = parse_value_pair(whole_thing, job_result.clone())?;
            let left = left.as_f64().ok_or(possible_num_error.clone())?;
            let right = right.as_f64().ok_or(possible_num_error.clone())?;
            return Ok(left >= right);
        }

        _ => {}
    };

    Err(build_pest_error(
        parsed.next().expect("Has inside"),
        "Could not match condition",
    ))
}
