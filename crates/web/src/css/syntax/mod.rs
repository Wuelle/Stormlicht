//! Implements the [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/) draft.

pub mod parser;
mod rule_parser;
mod tokenizer;

pub(crate) use rule_parser::RuleParser;
pub use tokenizer::Token;
