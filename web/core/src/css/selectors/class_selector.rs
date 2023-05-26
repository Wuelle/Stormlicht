use std::borrow::Cow;

use super::{CSSValidateSelector, Selector, Specificity};
use crate::{
    css::{syntax::Token, CSSParse, ParseError, Parser},
    dom::{dom_objects::Element, DOMPtr},
};

/// <https://drafts.csswg.org/selectors-4/#typedef-class-selector>
#[derive(Clone, Debug, PartialEq)]
pub struct ClassSelector<'a> {
    pub ident: Cow<'a, str>,
}

impl<'a> CSSParse<'a> for ClassSelector<'a> {
    // <https://drafts.csswg.org/selectors-4/#typedef-class-selector>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(Token::Delim('.')) = parser.next_token() {
            if let Some(Token::Ident(ident)) = parser.next_token() {
                return Ok(ClassSelector { ident });
            }
        }
        Err(ParseError)
    }
}

impl<'a> CSSValidateSelector for ClassSelector<'a> {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<'a> Selector for ClassSelector<'a> {
    fn matches(&self, _element: &DOMPtr<Element>) -> bool {
        log::warn!("FIXME: Class selector matching");
        false
    }

    fn specificity(&self) -> Specificity {
        Specificity::new(0, 1, 0)
    }
}
