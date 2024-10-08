use crate::{Number, Value};

use super::{
    tokenization::{SkipLineTerminators, Token, Tokenizer},
    SyntaxError,
};

/// <https://262.ecma-international.org/14.0/#prod-Literal>
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    NullLiteral,
    BooleanLiteral(bool),
    NumericLiteral(Number),
    StringLiteral(String),
}

impl Literal {
    pub fn parse(tokenizer: &mut Tokenizer<'_>) -> Result<Self, SyntaxError> {
        // FIXME: How should we propagate syntax errors here?
        let literal = match tokenizer.next(SkipLineTerminators::Yes)? {
            Some(Token::Identifier(identifier)) => match identifier.as_str() {
                "null" => Self::NullLiteral,
                "true" => Self::BooleanLiteral(true),
                "false" => Self::BooleanLiteral(false),
                _ => {
                    return Err(tokenizer
                        .syntax_error(format!("unknown literal identifier: {identifier:?}")))
                },
            },
            Some(Token::StringLiteral(string_literal)) => {
                Self::StringLiteral(string_literal.clone())
            },
            Some(Token::NumericLiteral(numeric_literal)) => {
                Self::NumericLiteral(Number::new(f64::from(numeric_literal)))
            },
            _ => return Err(tokenizer.syntax_error("expected literal token")),
        };

        Ok(literal)
    }
}

impl From<Literal> for Value {
    fn from(value: Literal) -> Self {
        match value {
            Literal::NullLiteral => Self::Null,
            Literal::BooleanLiteral(bool) => Self::Boolean(bool),
            Literal::NumericLiteral(number) => Self::Number(number),
            Literal::StringLiteral(s) => Self::String(s),
        }
    }
}
