use std::fmt;

use crate::{
    css::{
        selectors::{CSSValidateSelector, NSPrefix},
        syntax::Token,
        CSSParse, ParseError, Parser, Serialize, Serializer,
    },
    InternedString,
};

/// <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WQName {
    pub prefix: Option<NSPrefix>,
    pub ident: InternedString,
}

impl<'a> CSSParse<'a> for WQName {
    // <https://drafts.csswg.org/selectors-4/#typedef-wq-name>
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        let prefix = parser.parse_optional_value(NSPrefix::parse);

        parser.skip_whitespace();

        if let Some(Token::Ident(ident)) = parser.next_token() {
            Ok(WQName { prefix, ident })
        } else {
            Err(ParseError)
        }
    }
}

impl CSSValidateSelector for WQName {
    fn is_valid(&self) -> bool {
        true
    }
}

impl Serialize for WQName {
    fn serialize_to<T: Serializer>(&self, serializer: &mut T) -> fmt::Result {
        // FIXME: serialize name space prefix
        serializer.serialize_identifier(&self.ident.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::WQName;
    use crate::css::{selectors::NSPrefix, CSSParse};

    #[test]
    fn parse_wq_name() {
        assert_eq!(
            WQName::parse_from_str("foo | bar"),
            Ok(WQName {
                prefix: Some(NSPrefix::Ident("foo".into())),
                ident: "bar".into()
            })
        );

        assert_eq!(
            WQName::parse_from_str("bar"),
            Ok(WQName {
                prefix: None,
                ident: "bar".into()
            })
        );
    }
}
