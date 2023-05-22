use std::borrow::Cow;

use super::CSSValidateSelector;
use crate::css::{
    parser::{CSSParse, ParseError, Parser},
    tokenizer::Token,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-ns-prefix>
#[derive(Clone, Debug, PartialEq)]
pub enum NSPrefix<'a> {
    Ident(Cow<'a, str>),
    Asterisk,
    Empty,
}

impl<'a> CSSParse<'a> for NSPrefix<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-ns-prefix>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let prefix = parser
            .parse_optional_value(|parser| match parser.next_token() {
                Some(Token::Ident(ident)) => Ok(NSPrefix::Ident(ident)),
                Some(Token::Delim('*')) => Ok(NSPrefix::Asterisk),
                _ => Err(ParseError),
            })
            .unwrap_or(NSPrefix::Empty);

        parser.skip_whitespace();

        if let Some(Token::Delim('|')) = parser.next_token() {
            Ok(prefix)
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSValidateSelector for NSPrefix<'a> {}

#[cfg(test)]
mod tests {
    use super::NSPrefix;
    use crate::css::parser::CSSParse;

    #[test]
    fn parse_ns_prefix() {
        assert_eq!(
            NSPrefix::parse_from_str("foo |"),
            Ok(NSPrefix::Ident("foo".into()))
        );
        assert_eq!(NSPrefix::parse_from_str("* |"), Ok(NSPrefix::Asterisk));
        assert_eq!(NSPrefix::parse_from_str("|"), Ok(NSPrefix::Empty),);
    }
}