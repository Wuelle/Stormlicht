use std::fmt;

use crate::css::{
    selectors::CSSValidateSelector, syntax::Token, CSSParse, ParseError, Parser, Serialize,
    Serializer,
};

/// <https://drafts.csswg.org/selectors-4/#combinators>
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Combinator {
    /// ` `
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#descendant-combinators>
    #[default]
    Descendant,

    /// `>`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#child-combinators>
    Child,

    /// `+`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#adjacent-sibling-combinators>
    NextSibling,

    /// `~`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#general-sibling-combinators>
    SubsequentSibling,

    /// `||`
    ///
    /// # Specification
    /// <https://drafts.csswg.org/selectors-4/#the-column-combinator>
    Column,
}

impl Combinator {
    pub fn is_descendant(&self) -> bool {
        *self == Self::Descendant
    }
}

impl<'a> CSSParse<'a> for Combinator {
    // <https://drafts.csswg.org/selectors-4/#typedef-combinator>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        match parser.next_token() {
            Some(Token::Delim('>')) => Ok(Combinator::Child),
            Some(Token::Delim('+')) => Ok(Combinator::NextSibling),
            Some(Token::Delim('~')) => Ok(Combinator::SubsequentSibling),
            Some(Token::Delim('|')) => {
                if matches!(parser.next_token(), Some(Token::Delim('|'))) {
                    Ok(Combinator::Column)
                } else {
                    Err(ParseError)
                }
            },
            _ => Err(ParseError),
        }
    }
}

impl CSSValidateSelector for Combinator {
    fn is_valid(&self) -> bool {
        // We don't support *any* combinators
        // As per spec, we therefore treat them as invalid
        false
    }
}

impl Serialize for Combinator {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        match self {
            Self::Descendant => serializer.serialize(' '),
            Self::Child => serializer.serialize('>'),
            Self::NextSibling => serializer.serialize('+'),
            Self::SubsequentSibling => serializer.serialize('~'),
            Self::Column => serializer.serialize("||"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Combinator;
    use crate::css::CSSParse;

    #[test]
    fn parse_combinator() {
        assert_eq!(Combinator::parse_from_str(">"), Ok(Combinator::Child));
        assert_eq!(Combinator::parse_from_str("+"), Ok(Combinator::NextSibling));
        assert_eq!(
            Combinator::parse_from_str("~"),
            Ok(Combinator::SubsequentSibling)
        );
        assert_eq!(Combinator::parse_from_str("||"), Ok(Combinator::Column));
    }
}
