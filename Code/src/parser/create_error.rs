use pest::error::{self, Error};
use pest::iterators::Pair;

use super::Rule;

pub fn build_pest_error(pair: Pair<Rule>, msg: &str) -> error::Error<Rule> {
    Error::new_from_span(
        error::ErrorVariant::CustomError { message: msg.to_string() },
        pair.as_span(),
    )
}
