//! <https://drafts.csswg.org/css-lists/#propdef-list-style-type>

use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    static_interned, InternedString,
};

/// <https://drafts.csswg.org/css-counter-styles-3/#typedef-counter-style-name>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterStyle {
    /// <https://drafts.csswg.org/css-counter-styles-3/#decimal>
    Decimal,

    /// <https://drafts.csswg.org/css-counter-styles-3/#disc>
    Disc,

    /// <https://drafts.csswg.org/css-counter-styles-3/#square>
    Square,

    /// <https://drafts.csswg.org/css-counter-styles-3/#disclosure-open>
    DisclosureOpen,

    /// <https://drafts.csswg.org/css-counter-styles-3/#disclosure-closed>
    DisclosureClosed,
}

/// <https://drafts.csswg.org/css-lists/#propdef-list-style-type>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListStyleType {
    CounterStyle(CounterStyle),
    String(InternedString),
    None,
}

impl CounterStyle {
    #[must_use]
    pub fn as_str(&self) -> String {
        // FIXME: I don't think the spacing between
        //        the ::marker element and the list-item
        //        is specified - valid to use NBSP (U+00A0) here?
        match self {
            Self::Decimal | // FIXME: implement decimal
            Self::Disc => String::from("•\u{00A0}"),
            Self::Square => String::from("▪\u{00A0}"),
            Self::DisclosureOpen => String::from("▾\u{00A0}"),
            // FIXME: This should respect the writing type
            Self::DisclosureClosed => String::from("▸\u{00A0}")
        }
    }
}

impl ListStyleType {
    #[must_use]
    pub fn as_str(&self) -> Option<String> {
        match self {
            Self::CounterStyle(counter) => Some(counter.as_str()),
            Self::String(s) => Some(s.to_string()),
            Self::None => None,
        }
    }
}

impl<'a> CSSParse<'a> for ListStyleType {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let list_style_type = match parser.next_token_ignoring_whitespace() {
            Some(Token::Ident(static_interned!("none"))) => Self::None,
            Some(Token::Ident(static_interned!("decimal"))) => {
                Self::CounterStyle(CounterStyle::Decimal)
            },
            Some(Token::Ident(static_interned!("disc"))) => Self::CounterStyle(CounterStyle::Disc),
            Some(Token::Ident(static_interned!("square"))) => {
                Self::CounterStyle(CounterStyle::Square)
            },
            Some(Token::Ident(static_interned!("disclosure-open"))) => {
                Self::CounterStyle(CounterStyle::DisclosureOpen)
            },
            Some(Token::Ident(static_interned!("disclosure-closed"))) => {
                Self::CounterStyle(CounterStyle::DisclosureClosed)
            },
            Some(Token::String(s)) => Self::String(s),
            _ => return Err(ParseError),
        };

        Ok(list_style_type)
    }
}