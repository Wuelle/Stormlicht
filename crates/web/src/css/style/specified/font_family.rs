use std::fmt;

use crate::{
    css::{
        self,
        style::{computed, StyleContext, ToComputedStyle},
        syntax::Token,
        CSSParse,
    },
    static_interned, InternedString,
};

/// <https://drafts.csswg.org/css-fonts/#font-family-prop>
#[derive(Clone, Debug)]
pub struct FontFamily {
    fonts: Vec<FontName>,
}

#[derive(Clone, Debug)]
pub enum FontName {
    /// <https://drafts.csswg.org/css-fonts/#family-name-syntax>
    Family(InternedString),
    Generic(GenericFontFamily),
}

impl FontFamily {
    #[must_use]
    pub fn fonts(&self) -> &[FontName] {
        &self.fonts
    }
}

/// <https://drafts.csswg.org/css-fonts/#generic-family-value>
#[derive(Clone, Copy, Debug)]
pub enum GenericFontFamily {
    /// <https://drafts.csswg.org/css-fonts/#serif-def>
    Serif,

    /// <https://drafts.csswg.org/css-fonts/#sans-serif-def>
    SansSerif,

    /// <https://drafts.csswg.org/css-fonts/#cursive-def>
    Cursive,

    /// <https://drafts.csswg.org/css-fonts/#fantasy-def>
    Fantasy,

    /// <https://drafts.csswg.org/css-fonts/#monospace-def>
    Monospace,

    /// <https://drafts.csswg.org/css-fonts/#system-ui-def>
    SystemUi,

    /// <https://drafts.csswg.org/css-fonts/#emoji-def>
    Emoji,

    /// <https://drafts.csswg.org/css-fonts/#math-def>
    Math,

    /// <https://drafts.csswg.org/css-fonts/#generic(fangsong)-def>
    GenericFangSong,

    /// <https://drafts.csswg.org/css-fonts/#ui-serif-def>
    UiSerif,

    /// <https://drafts.csswg.org/css-fonts/#ui-sans-serif-def>
    UiSansSerif,

    /// <https://drafts.csswg.org/css-fonts/#ui-monospace-def>
    UiMonospace,

    /// <https://drafts.csswg.org/css-fonts/#ui-rounded-def>
    UiRounded,
}

impl Default for FontFamily {
    fn default() -> Self {
        // The initial value is UA dependent
        Self {
            fonts: vec![FontName::Generic(GenericFontFamily::SansSerif)],
        }
    }
}

impl<'a> CSSParse<'a> for FontFamily {
    fn parse(parser: &mut css::Parser<'a>) -> Result<Self, css::ParseError> {
        let mut desired_fonts = vec![];

        while let Some(desired_font) = parser.parse_optional_value(CSSParse::parse) {
            desired_fonts.push(desired_font);
        }

        if desired_fonts.is_empty() {
            return Err(css::ParseError);
        }

        Ok(Self {
            fonts: desired_fonts,
        })
    }
}

impl<'a> CSSParse<'a> for FontName {
    fn parse(parser: &mut css::Parser<'a>) -> Result<Self, css::ParseError> {
        if let Some(Token::String(name)) = parser.peek_token_ignoring_whitespace(0) {
            let name = *name;
            let _ = parser.next_token_ignoring_whitespace();
            Ok(Self::Family(name))
        } else {
            let generic_family = CSSParse::parse(parser)?;
            Ok(Self::Generic(generic_family))
        }
    }
}

impl<'a> CSSParse<'a> for GenericFontFamily {
    fn parse(parser: &mut css::Parser<'a>) -> Result<Self, css::ParseError> {
        let parsed_value = match parser.next_token() {
            Some(Token::Ident(static_interned!("serif"))) => Self::Serif,
            Some(Token::Ident(static_interned!("sans-serif"))) => Self::SansSerif,
            Some(Token::Ident(static_interned!("cursive"))) => Self::Cursive,
            Some(Token::Ident(static_interned!("fantasy"))) => Self::Fantasy,
            Some(Token::Ident(static_interned!("monospace"))) => Self::Monospace,
            Some(Token::Ident(static_interned!("system-ui"))) => Self::SystemUi,
            Some(Token::Ident(static_interned!("emoji"))) => Self::Emoji,
            Some(Token::Ident(static_interned!("math"))) => Self::Math,
            Some(Token::Ident(static_interned!("generic(fangsong)"))) => Self::GenericFangSong,
            Some(Token::Ident(static_interned!("ui-serif"))) => Self::UiSerif,
            Some(Token::Ident(static_interned!("ui-sans-serif"))) => Self::UiSansSerif,
            Some(Token::Ident(static_interned!("ui-monospace"))) => Self::UiMonospace,
            Some(Token::Ident(static_interned!("ui-rounded"))) => Self::UiRounded,
            _ => return Err(css::ParseError),
        };
        Ok(parsed_value)
    }
}

impl fmt::Display for GenericFontFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serif => "serif".fmt(f),
            Self::SansSerif => "sans-serif".fmt(f),
            Self::Cursive => "cursive".fmt(f),
            Self::Fantasy => "fantasy".fmt(f),
            Self::Monospace => "monospace".fmt(f),
            Self::SystemUi => "system-ui".fmt(f),
            Self::Emoji => "emoji".fmt(f),
            Self::Math => "math".fmt(f),
            Self::GenericFangSong => "generic(fangsong)".fmt(f),
            Self::UiSerif => "ui-serif".fmt(f),
            Self::UiSansSerif => "ui-sans-serif".fmt(f),
            Self::UiMonospace => "ui-monospace".fmt(f),
            Self::UiRounded => "ui-rounded".fmt(f),
        }
    }
}

impl ToComputedStyle for FontFamily {
    type Computed = computed::FontFamily;

    fn to_computed_style(&self, context: &StyleContext) -> Self::Computed {
        _ = context;
        self.clone()
    }
}
