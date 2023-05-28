//! <https://drafts.csswg.org/css-color>

use crate::css::{syntax::Token, CSSParse, ParseError, Parser};

use super::Number;

/// <https://drafts.csswg.org/css-color/#color-syntax>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Color {
    /// <https://drafts.csswg.org/css-color/#valdef-color-aliceblue>
    pub const ALICE_BLUE: Self = Self::rgb(240, 248, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-antiquewhite>
    pub const ANTIQUE_WHITE: Self = Self::rgb(250, 235, 215);

    /// <https://drafts.csswg.org/css-color/#valdef-color-aqua>
    pub const AQUA: Self = Self::rgb(0, 255, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-aquamarine>
    pub const AQUAMARINE: Self = Self::rgb(127, 255, 212);

    /// <https://drafts.csswg.org/css-color/#valdef-color-azure>
    pub const AZURE: Self = Self::rgb(240, 255, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-beige>
    pub const BEIGE: Self = Self::rgb(245, 245, 220);

    /// <https://drafts.csswg.org/css-color/#valdef-color-bisque>
    pub const BISQUE: Self = Self::rgb(255, 228, 196);

    /// <https://drafts.csswg.org/css-color/#valdef-color-black>
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-blanchedalmond>
    pub const BLANCHED_ALMOND: Self = Self::rgb(255, 235, 205);

    /// <https://drafts.csswg.org/css-color/#valdef-color-blue>
    pub const BLUE: Self = Self::rgb(0, 0, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-blueviolet>
    pub const BLUE_VIOLET: Self = Self::rgb(138, 43, 226);

    /// <https://drafts.csswg.org/css-color/#valdef-color-brown>
    pub const BROWN: Self = Self::rgb(165, 42, 42);

    /// <https://drafts.csswg.org/css-color/#valdef-color-burlywood>
    pub const BURLY_WOOD: Self = Self::rgb(222, 184, 135);

    /// <https://drafts.csswg.org/css-color/#valdef-color-cadetblue>
    pub const CADET_BLUE: Self = Self::rgb(95, 158, 160);

    /// <https://drafts.csswg.org/css-color/#valdef-color-chartreuse>
    pub const CHARTREUSE: Self = Self::rgb(127, 255, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-chocolate>
    pub const CHOCOLATE: Self = Self::rgb(210, 105, 30);

    /// <https://drafts.csswg.org/css-color/#valdef-color-coral>
    pub const CORAL: Self = Self::rgb(255, 127, 80);

    /// <https://drafts.csswg.org/css-color/#valdef-color-cornflowerblue>
    pub const CORNFLOWER_BLUE: Self = Self::rgb(100, 149, 237);

    /// <https://drafts.csswg.org/css-color/#valdef-color-cornsilk>
    pub const CORN_SILK: Self = Self::rgb(255, 248, 220);

    /// <https://drafts.csswg.org/css-color/#valdef-color-crimson>
    pub const CRIMSON: Self = Self::rgb(220, 20, 60);

    /// <https://drafts.csswg.org/css-color/#valdef-color-cyan>
    pub const CYAN: Self = Self::rgb(0, 255, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkblue>
    pub const DARK_BLUE: Self = Self::rgb(0, 0, 139);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkcyan>
    pub const DARK_CYAN: Self = Self::rgb(0, 139, 139);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkgoldenrod>
    pub const DARK_GOLDEN_ROD: Self = Self::rgb(184, 134, 11);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkgray>
    pub const DARK_GRAY: Self = Self::rgb(169, 169, 169);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkgreen>
    pub const DARK_GREEN: Self = Self::rgb(0, 100, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkgrey>
    pub const DARK_GREY: Self = Self::DARK_GRAY;

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkkhaki>
    pub const DARK_KHAKI: Self = Self::rgb(189, 183, 107);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkmagenta>
    pub const DARK_MAGENTA: Self = Self::rgb(139, 0, 139);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkolivegreen>
    pub const DARK_OLIVE_GREEN: Self = Self::rgb(85, 107, 47);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkorange>
    pub const DARK_ORANGE: Self = Self::rgb(255, 140, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkorchid>
    pub const DARK_ORCHID: Self = Self::rgb(153, 50, 204);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkred>
    pub const DARK_RED: Self = Self::rgb(139, 0, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darksalmon>
    pub const DARK_SALMON: Self = Self::rgb(233, 150, 122);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkseagreen>
    pub const DARK_SEA_GREEN: Self = Self::rgb(143, 188, 143);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkslateblue>
    pub const DARK_SLATE_BLUE: Self = Self::rgb(72, 61, 139);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkslategray>
    pub const DARK_SLATE_GRAY: Self = Self::rgb(47, 79, 79);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkslategrey>
    pub const DARK_SLATE_GREY: Self = Self::DARK_SLATE_GRAY;

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkturquoise>
    pub const DARK_TURQUOISE: Self = Self::rgb(0, 206, 209);

    /// <https://drafts.csswg.org/css-color/#valdef-color-darkviolet>
    pub const DARK_VIOLET: Self = Self::rgb(148, 0, 211);

    /// <https://drafts.csswg.org/css-color/#valdef-color-deeppink>
    pub const DEEP_PINK: Self = Self::rgb(255, 20, 147);

    /// <https://drafts.csswg.org/css-color/#valdef-color-deepskyblue>
    pub const DEEP_SKY_BLUE: Self = Self::rgb(0, 191, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-dimgray>
    pub const DIM_GRAY: Self = Self::rgb(105, 105, 105);

    /// <https://drafts.csswg.org/css-color/#valdef-color-dimgrey>
    pub const DIM_GREY: Self = Self::DIM_GRAY;

    /// <https://drafts.csswg.org/css-color/#valdef-color-dodgerblue>
    pub const DODGER_BLUE: Self = Self::rgb(30, 144, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-firebrick>
    pub const FIRE_BRICK: Self = Self::rgb(178, 34, 34);

    /// <https://drafts.csswg.org/css-color/#valdef-color-floralwhite>
    pub const FLORAL_WHITE: Self = Self::rgb(255, 250, 240);

    /// <https://drafts.csswg.org/css-color/#valdef-color-forestgreen>
    pub const FOREST_GREEN: Self = Self::rgb(34, 139, 34);

    /// <https://drafts.csswg.org/css-color/#valdef-color-fuchsia>
    pub const FUCHSIA: Self = Self::rgb(255, 0, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-gainsboro>
    pub const GAINSBORO: Self = Self::rgb(220, 220, 220);

    /// <https://drafts.csswg.org/css-color/#valdef-color-ghostwhite>
    pub const GHOST_WHITE: Self = Self::rgb(248, 248, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-gold>
    pub const GOLD: Self = Self::rgb(255, 215, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-goldenrod>
    pub const GOLDEN_ROD: Self = Self::rgb(218, 165, 32);

    /// <https://drafts.csswg.org/css-color/#valdef-color-gray>
    pub const GRAY: Self = Self::rgb(128, 128, 128);

    /// <https://drafts.csswg.org/css-color/#valdef-color-green>
    pub const GREEN: Self = Self::rgb(0, 128, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-greenyellow>
    pub const GREEN_YELLOW: Self = Self::rgb(173, 255, 47);

    /// <https://drafts.csswg.org/css-color/#valdef-color-grey>
    pub const GREY: Self = Self::GRAY;

    /// <https://drafts.csswg.org/css-color/#valdef-color-honeydew>
    pub const HONEYDEW: Self = Self::rgb(240, 255, 240);

    /// <https://drafts.csswg.org/css-color/#valdef-color-hotpink>
    pub const HOT_PINK: Self = Self::rgb(255, 105, 180);

    /// <https://drafts.csswg.org/css-color/#valdef-color-indianred>
    pub const INDIAN_RED: Self = Self::rgb(205, 92, 92);

    /// <https://drafts.csswg.org/css-color/#valdef-color-indigo>
    pub const INDIGO: Self = Self::rgb(75, 0, 130);

    /// <https://drafts.csswg.org/css-color/#valdef-color-ivory>
    pub const IVORY: Self = Self::rgb(255, 255, 240);

    /// <https://drafts.csswg.org/css-color/#valdef-color-khaki>
    pub const KHAKI: Self = Self::rgb(240, 230, 140);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lavender>
    pub const LAVENDER: Self = Self::rgb(230, 230, 250);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lavenderblush>
    pub const LAVENDER_BLUSH: Self = Self::rgb(255, 240, 245);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lawngreen>
    pub const LAWN_GREEN: Self = Self::rgb(124, 252, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lemonchiffon>
    pub const LEMON_CHIFFON: Self = Self::rgb(255, 250, 205);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightblue>
    pub const LIGHT_BLUE: Self = Self::rgb(173, 216, 230);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightcoral>
    pub const LIGHT_CORAL: Self = Self::rgb(240, 128, 128);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightcyan>
    pub const LIGHT_CYAN: Self = Self::rgb(224, 255, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightgoldenrodyellow>
    pub const LIGHT_GOLDEN_ROD_YELLOW: Self = Self::rgb(250, 250, 210);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightgray>
    pub const LIGHT_GRAY: Self = Self::rgb(211, 211, 211);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightgreen>
    pub const LIGHT_GREEN: Self = Self::rgb(144, 238, 144);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightgrey>
    pub const LIGHT_GREY: Self = Self::LIGHT_GRAY;

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightpink>
    pub const LIGHT_PINK: Self = Self::rgb(255, 182, 193);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightsalmon>
    pub const LIGHT_SALMON: Self = Self::rgb(255, 160, 122);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightseagreen>
    pub const LIGHT_SEA_GREEN: Self = Self::rgb(32, 178, 170);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightskyblue>
    pub const LIGHT_SKY_BLUE: Self = Self::rgb(135, 206, 250);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightslategray>
    pub const LIGHT_SLATE_GRAY: Self = Self::rgb(119, 136, 153);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightslategrey>
    pub const LIGHT_SLATE_GREY: Self = Self::LIGHT_GRAY;

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightsteelblue>
    pub const LIGHT_STEEL_BLUE: Self = Self::rgb(176, 196, 222);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lightyellow>
    pub const LIGHT_YELLOW: Self = Self::rgb(255, 255, 224);

    /// <https://drafts.csswg.org/css-color/#valdef-color-lime>
    pub const LIME: Self = Self::rgb(0, 255, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-limegreen>
    pub const LIME_GREEN: Self = Self::rgb(50, 205, 50);

    /// <https://drafts.csswg.org/css-color/#valdef-color-linen>
    pub const LINEN: Self = Self::rgb(250, 240, 230);

    /// <https://drafts.csswg.org/css-color/#valdef-color-magenta>
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-maroon>
    pub const MAROON: Self = Self::rgb(128, 0, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mediumaquamarine>
    pub const MEDIUM_AQUAMARINE: Self = Self::rgb(102, 205, 170);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mediumblue>
    pub const MEDIUM_BLUE: Self = Self::rgb(0, 0, 205);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mediumorchid>
    pub const MEDIUM_ORCHID: Self = Self::rgb(186, 85, 211);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mediumpurple>
    pub const MEDIUM_PURPLE: Self = Self::rgb(147, 112, 219);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mediumseagreen>
    pub const MEDIUM_SEA_GREEN: Self = Self::rgb(60, 179, 113);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mediumslateblue>
    pub const MEDIUM_SLATE_BLUE: Self = Self::rgb(123, 104, 238);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mediumspringgreen>
    pub const MEDIUM_SPRING_GREEN: Self = Self::rgb(0, 250, 154);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mediumturquoise>
    pub const MEDIUM_TURQUOISE: Self = Self::rgb(72, 209, 204);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mediumvioletred>
    pub const MEDIUM_VIOLET_RED: Self = Self::rgb(199, 21, 133);

    /// <https://drafts.csswg.org/css-color/#valdef-color-midnightblue>
    pub const MIDNIGHT_BLUE: Self = Self::rgb(25, 25, 112);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mintcream>
    pub const MINT_CREAM: Self = Self::rgb(245, 255, 250);

    /// <https://drafts.csswg.org/css-color/#valdef-color-mistyrose>
    pub const MISTY_ROSE: Self = Self::rgb(255, 228, 225);

    /// <https://drafts.csswg.org/css-color/#valdef-color-moccasin>
    pub const MOCCASIN: Self = Self::rgb(255, 228, 181);

    /// <https://drafts.csswg.org/css-color/#valdef-color-navajowhite>
    pub const NAVAJO_WHITE: Self = Self::rgb(255, 222, 173);

    /// <https://drafts.csswg.org/css-color/#valdef-color-navy>
    pub const NAVY: Self = Self::rgb(0, 0, 128);

    /// <https://drafts.csswg.org/css-color/#valdef-color-oldlace>
    pub const OLD_LACE: Self = Self::rgb(253, 245, 230);

    /// <https://drafts.csswg.org/css-color/#valdef-color-olive>
    pub const OLIVE: Self = Self::rgb(128, 128, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-olivedrab>
    pub const OLIVE_DRAB: Self = Self::rgb(107, 142, 35);

    /// <https://drafts.csswg.org/css-color/#valdef-color-orange>
    pub const ORANGE: Self = Self::rgb(255, 165, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-orangered>
    pub const ORANGE_RED: Self = Self::rgb(255, 69, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-orchid>
    pub const ORCHID: Self = Self::rgb(218, 112, 214);

    /// <https://drafts.csswg.org/css-color/#valdef-color-palegoldenrod>
    pub const PALE_GOLDEN_ROD: Self = Self::rgb(238, 232, 170);

    /// <https://drafts.csswg.org/css-color/#valdef-color-palegreen>
    pub const PALE_GREEN: Self = Self::rgb(152, 251, 152);

    /// <https://drafts.csswg.org/css-color/#valdef-color-paleturquoise>
    pub const PALE_TURQUOISE: Self = Self::rgb(175, 238, 238);

    /// <https://drafts.csswg.org/css-color/#valdef-color-palevioletred>
    pub const PALE_VIOLET_RED: Self = Self::rgb(219, 112, 147);

    /// <https://drafts.csswg.org/css-color/#valdef-color-papayawhip>
    pub const PAPAYA_WHIP: Self = Self::rgb(255, 239, 213);

    /// <https://drafts.csswg.org/css-color/#valdef-color-peachpuff>
    pub const PEACH_PUFF: Self = Self::rgb(255, 218, 185);

    /// <https://drafts.csswg.org/css-color/#valdef-color-peru>
    pub const PERU: Self = Self::rgb(205, 133, 63);

    /// <https://drafts.csswg.org/css-color/#valdef-color-pink>
    pub const PINK: Self = Self::rgb(255, 192, 203);

    /// <https://drafts.csswg.org/css-color/#valdef-color-plum>
    pub const PLUM: Self = Self::rgb(221, 160, 221);

    /// <https://drafts.csswg.org/css-color/#valdef-color-powderblue>
    pub const POWDER_BLUE: Self = Self::rgb(176, 224, 230);

    /// <https://drafts.csswg.org/css-color/#valdef-color-purple>
    pub const PURPLE: Self = Self::rgb(128, 0, 128);

    /// <https://drafts.csswg.org/css-color/#valdef-color-rebeccapurple>
    pub const REBECCA_PURPLE: Self = Self::rgb(102, 51, 153);

    /// <https://drafts.csswg.org/css-color/#valdef-color-red>
    pub const RED: Self = Self::rgb(255, 0, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-rosybrown>
    pub const ROSY_BROWN: Self = Self::rgb(188, 143, 143);

    /// <https://drafts.csswg.org/css-color/#valdef-color-royalblue>
    pub const ROYAL_BLUE: Self = Self::rgb(65, 105, 225);

    /// <https://drafts.csswg.org/css-color/#valdef-color-saddlebrown>
    pub const SADDLE_BROWN: Self = Self::rgb(139, 69, 19);

    /// <https://drafts.csswg.org/css-color/#valdef-color-salmon>
    pub const SALMON: Self = Self::rgb(250, 128, 114);

    /// <https://drafts.csswg.org/css-color/#valdef-color-sandybrown>
    pub const SANDY_BROWN: Self = Self::rgb(244, 164, 96);

    /// <https://drafts.csswg.org/css-color/#valdef-color-seagreen>
    pub const SEA_GREEN: Self = Self::rgb(46, 139, 87);

    /// <https://drafts.csswg.org/css-color/#valdef-color-seashell>
    pub const SEASHELL: Self = Self::rgb(255, 245, 238);

    /// <https://drafts.csswg.org/css-color/#valdef-color-sienna>
    pub const SIENNA: Self = Self::rgb(160, 82, 45);

    /// <https://drafts.csswg.org/css-color/#valdef-color-silver>
    pub const SILVER: Self = Self::rgb(192, 192, 192);

    /// <https://drafts.csswg.org/css-color/#valdef-color-skyblue>
    pub const SKY_BLUE: Self = Self::rgb(135, 206, 235);

    /// <https://drafts.csswg.org/css-color/#valdef-color-slateblue>
    pub const SLATE_BLUE: Self = Self::rgb(106, 90, 205);

    /// <https://drafts.csswg.org/css-color/#valdef-color-slategray>
    pub const SLATE_GRAY: Self = Self::rgb(112, 128, 144);

    /// <https://drafts.csswg.org/css-color/#valdef-color-slategrey>
    pub const SLATE_GREY: Self = Self::SLATE_GRAY;

    /// <https://drafts.csswg.org/css-color/#valdef-color-snow>
    pub const SNOW: Self = Self::rgb(255, 250, 250);

    /// <https://drafts.csswg.org/css-color/#valdef-color-springgreen>
    pub const SPRING_GREEN: Self = Self::rgb(0, 255, 127);

    /// <https://drafts.csswg.org/css-color/#valdef-color-steelblue>
    pub const STEEL_BLUE: Self = Self::rgb(70, 130, 180);

    /// <https://drafts.csswg.org/css-color/#valdef-color-tan>
    pub const TAN: Self = Self::rgb(210, 180, 140);

    /// <https://drafts.csswg.org/css-color/#valdef-color-teal>
    pub const TEAL: Self = Self::rgb(0, 128, 128);

    /// <https://drafts.csswg.org/css-color/#valdef-color-thistle>
    pub const THISTLE: Self = Self::rgb(216, 191, 216);

    /// <https://drafts.csswg.org/css-color/#valdef-color-tomato>
    pub const TOMATO: Self = Self::rgb(255, 99, 71);

    /// <https://drafts.csswg.org/css-color/#valdef-color-turquoise>
    pub const TURQUOISE: Self = Self::rgb(64, 224, 208);

    /// <https://drafts.csswg.org/css-color/#valdef-color-violet>
    pub const VIOLET: Self = Self::rgb(238, 130, 238);

    /// <https://drafts.csswg.org/css-color/#valdef-color-wheat>
    pub const WHEAT: Self = Self::rgb(245, 222, 179);

    /// <https://drafts.csswg.org/css-color/#valdef-color-white>
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    /// <https://drafts.csswg.org/css-color/#valdef-color-whitesmoke>
    pub const WHITE_SMOKE: Self = Self::rgb(245, 245, 245);

    /// <https://drafts.csswg.org/css-color/#valdef-color-yellow>
    pub const YELLOW: Self = Self::rgb(255, 255, 0);

    /// <https://drafts.csswg.org/css-color/#valdef-color-yellowgreen>
    pub const YELLOW_GREEN: Self = Self::rgb(154, 205, 50);

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, u8::MAX)
    }

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn parse_from_name(parser: &mut Parser) -> Result<Self, ParseError> {
        if let Some(Token::Ident(name)) = parser.next_token() {
            let color = match name.as_ref() {
                "aliceblue" => Self::ALICE_BLUE,
                "antiquewhite" => Self::ANTIQUE_WHITE,
                "aqua" => Self::AQUA,
                "aquamarine" => Self::AQUAMARINE,
                "azure" => Self::AZURE,
                "beige" => Self::BEIGE,
                "bisque" => Self::BISQUE,
                "black" => Self::BLACK,
                "blanchedalmond" => Self::BLANCHED_ALMOND,
                "blue" => Self::BLUE,
                "blueviolet" => Self::BLUE_VIOLET,
                "brown" => Self::BROWN,
                "burlywood" => Self::BURLY_WOOD,
                "cadetblue" => Self::CADET_BLUE,
                "chartreuse" => Self::CHARTREUSE,
                "chocolate" => Self::CHOCOLATE,
                "coral" => Self::CORAL,
                "cornflowerblue" => Self::CORNFLOWER_BLUE,
                "cornsilk" => Self::CORN_SILK,
                "crimson" => Self::CRIMSON,
                "cyan" => Self::CYAN,
                "darkblue" => Self::DARK_BLUE,
                "darkcyan" => Self::DARK_CYAN,
                "darkgoldenrod" => Self::DARK_GOLDEN_ROD,
                "darkgray" => Self::DARK_GRAY,
                "darkgreen" => Self::DARK_GREEN,
                "darkgrey" => Self::DARK_GREY,
                "darkkhaki" => Self::DARK_KHAKI,
                "darkmagenta" => Self::DARK_MAGENTA,
                "darkolivegreen" => Self::DARK_OLIVE_GREEN,
                "darkorange" => Self::DARK_ORANGE,
                "darkorchid" => Self::DARK_ORCHID,
                "darkred" => Self::DARK_RED,
                "darksalmon" => Self::DARK_SALMON,
                "darkseagreen" => Self::DARK_SEA_GREEN,
                "darkslateblue" => Self::DARK_SLATE_BLUE,
                "darkslategray" => Self::DARK_SLATE_GRAY,
                "darkslategrey" => Self::DARK_SLATE_GREY,
                "darkturquoise" => Self::DARK_TURQUOISE,
                "darkviolet" => Self::DARK_VIOLET,
                "deeppink" => Self::DEEP_PINK,
                "deepskyblue" => Self::DEEP_SKY_BLUE,
                "dimgray" => Self::DIM_GRAY,
                "dimgrey" => Self::DIM_GREY,
                "dodgerblue" => Self::DODGER_BLUE,
                "firebrick" => Self::FIRE_BRICK,
                "floralwhite" => Self::FLORAL_WHITE,
                "forestgreen" => Self::FOREST_GREEN,
                "fuchsia" => Self::FUCHSIA,
                "gainsboro" => Self::GAINSBORO,
                "ghostwhite" => Self::GHOST_WHITE,
                "gold" => Self::GOLD,
                "goldenrod" => Self::GOLDEN_ROD,
                "gray" => Self::GRAY,
                "green" => Self::GREEN,
                "greenyellow" => Self::GREEN_YELLOW,
                "grey" => Self::GREY,
                "honeydew" => Self::HONEYDEW,
                "hotpink" => Self::HOT_PINK,
                "indianred" => Self::INDIAN_RED,
                "indigo" => Self::INDIGO,
                "ivory" => Self::IVORY,
                "khaki" => Self::KHAKI,
                "lavender" => Self::LAVENDER,
                "lavenderblush" => Self::LAVENDER_BLUSH,
                "lawngreen" => Self::LAWN_GREEN,
                "lemonchiffon" => Self::LEMON_CHIFFON,
                "lightblue" => Self::LIGHT_BLUE,
                "lightcoral" => Self::LIGHT_CORAL,
                "lightcyan" => Self::LIGHT_CYAN,
                "lightgoldenrodyellow" => Self::LIGHT_GOLDEN_ROD_YELLOW,
                "lightgray" => Self::LIGHT_GRAY,
                "lightgreen" => Self::LIGHT_GREEN,
                "lightgrey" => Self::LIGHT_GREY,
                "lightpink" => Self::LIGHT_PINK,
                "lightsalmon" => Self::LIGHT_SALMON,
                "lightseagreen" => Self::LIGHT_SEA_GREEN,
                "lightskyblue" => Self::LIGHT_SKY_BLUE,
                "lightslategray" => Self::LIGHT_SLATE_GRAY,
                "lightslategrey" => Self::LIGHT_SLATE_GREY,
                "lightsteelblue" => Self::LIGHT_STEEL_BLUE,
                "lightyellow" => Self::LIGHT_YELLOW,
                "lime" => Self::LIME,
                "limegreen" => Self::LIME_GREEN,
                "linen" => Self::LINEN,
                "magenta" => Self::MAGENTA,
                "maroon" => Self::MAROON,
                "mediumaquamarine" => Self::MEDIUM_AQUAMARINE,
                "mediumblue" => Self::MEDIUM_BLUE,
                "mediumorchid" => Self::MEDIUM_ORCHID,
                "mediumpurple" => Self::MEDIUM_PURPLE,
                "mediumseagreeen" => Self::MEDIUM_SEA_GREEN,
                "mediumslateblue" => Self::MEDIUM_SLATE_BLUE,
                "mediumspringgreen" => Self::MEDIUM_SPRING_GREEN,
                "mediumturquoise" => Self::MEDIUM_TURQUOISE,
                "mediumvioletred" => Self::MEDIUM_VIOLET_RED,
                "midnightblue" => Self::MIDNIGHT_BLUE,
                "mintcream" => Self::MINT_CREAM,
                "mistyrose" => Self::MISTY_ROSE,
                "moccasin" => Self::MOCCASIN,
                "navajowhite" => Self::NAVAJO_WHITE,
                "navy" => Self::NAVY,
                "oldlace" => Self::OLD_LACE,
                "olive" => Self::OLIVE,
                "olivedrab" => Self::OLIVE_DRAB,
                "orange" => Self::ORANGE,
                "orangered" => Self::ORANGE_RED,
                "orchid" => Self::ORCHID,
                "palegoldenrod" => Self::PALE_GOLDEN_ROD,
                "palegreen" => Self::PALE_GREEN,
                "paleturquoise" => Self::PALE_TURQUOISE,
                "palevioletred" => Self::PALE_VIOLET_RED,
                "papayawhip" => Self::PAPAYA_WHIP,
                "peachpuff" => Self::PEACH_PUFF,
                "peru" => Self::PERU,
                "pink" => Self::PINK,
                "plum" => Self::PLUM,
                "powderblue" => Self::POWDER_BLUE,
                "purple" => Self::PURPLE,
                "rebeccapurple" => Self::REBECCA_PURPLE,
                "red" => Self::RED,
                "rosybrown" => Self::ROSY_BROWN,
                "royalblue" => Self::ROYAL_BLUE,
                "saddlebrown" => Self::SADDLE_BROWN,
                "salmon" => Self::SALMON,
                "sandybrown" => Self::SANDY_BROWN,
                "seagreen" => Self::SEA_GREEN,
                "seashell" => Self::SEASHELL,
                "sienna" => Self::SIENNA,
                "silver" => Self::SILVER,
                "skyblue" => Self::SKY_BLUE,
                "slateblue" => Self::SLATE_BLUE,
                "slategray" => Self::SLATE_GRAY,
                "slategrey" => Self::SLATE_GREY,
                "snow" => Self::SNOW,
                "springgreen" => Self::SPRING_GREEN,
                "steelblue" => Self::STEEL_BLUE,
                "tan" => Self::TAN,
                "teal" => Self::TEAL,
                "thistle" => Self::THISTLE,
                "tomato" => Self::TOMATO,
                "turquoise" => Self::TURQUOISE,
                "violet" => Self::VIOLET,
                "wheat" => Self::WHEAT,
                "white" => Self::WHITE,
                "whitesmoke" => Self::WHITE_SMOKE,
                "yellow" => Self::YELLOW,
                "yellowgreen" => Self::YELLOW_GREEN,
                _ => return Err(ParseError),
            };
            Ok(color)
        } else {
            Err(ParseError)
        }
    }

    fn parse_as_hex_color(parser: &mut Parser) -> Result<Self, ParseError> {
        // TODO: should we care about the hash flag here?
        if let Some(Token::Hash(ident, _)) = parser.next_token() {
            if ident.len() == 6 {
                // 6-digit hex number
                Ok(Self {
                    red: u8::from_str_radix(&ident[0..2], 16).map_err(|_| ParseError)?,
                    green: u8::from_str_radix(&ident[2..4], 16).map_err(|_| ParseError)?,
                    blue: u8::from_str_radix(&ident[4..6], 16).map_err(|_| ParseError)?,
                    alpha: u8::MAX,
                })
            } else if ident.len() == 8 {
                // 8-digit hex with alpha
                Ok(Self {
                    red: u8::from_str_radix(&ident[0..2], 16).map_err(|_| ParseError)?,
                    green: u8::from_str_radix(&ident[2..4], 16).map_err(|_| ParseError)?,
                    blue: u8::from_str_radix(&ident[4..6], 16).map_err(|_| ParseError)?,
                    alpha: u8::from_str_radix(&ident[6..8], 16).map_err(|_| ParseError)?,
                })
            } else if ident.len() == 3 {
                // Shorter version of 6-digit hex, each digit is "duplicated"
                Ok(Self {
                    red: u8::from_str_radix(&ident[0..1], 16).map_err(|_| ParseError)? * 0x11,
                    green: u8::from_str_radix(&ident[1..2], 16).map_err(|_| ParseError)? * 0x11,
                    blue: u8::from_str_radix(&ident[2..3], 16).map_err(|_| ParseError)? * 0x11,
                    alpha: u8::MAX,
                })
            } else if ident.len() == 4 {
                Ok(Self {
                    red: u8::from_str_radix(&ident[0..1], 16).map_err(|_| ParseError)? * 0x11,
                    green: u8::from_str_radix(&ident[1..2], 16).map_err(|_| ParseError)? * 0x11,
                    blue: u8::from_str_radix(&ident[2..3], 16).map_err(|_| ParseError)? * 0x11,
                    alpha: u8::from_str_radix(&ident[3..4], 16).map_err(|_| ParseError)? * 0x11,
                })
            } else {
                // Invalid length
                Err(ParseError)
            }
        } else {
            Err(ParseError)
        }
    }

    fn parse_legacy_rgb(parser: &mut Parser) -> Result<Self, ParseError> {
        // NOTE: The spec defines legacy-rgb and legacy-rgba
        //       But they are identical, so we do not differentiate between them
        let red = resolve_percentage(parser.expect_percentage()?);

        parser.skip_whitespace();
        parser.expect_token(Token::Comma)?;
        parser.skip_whitespace();

        let green = resolve_percentage(parser.expect_percentage()?);

        parser.skip_whitespace();
        parser.expect_token(Token::Comma)?;
        parser.skip_whitespace();

        let blue = resolve_percentage(parser.expect_percentage()?);

        parser.skip_whitespace();

        let alpha = parser
            .parse_optional_value(|p| {
                p.expect_token(Token::Comma)?;
                p.skip_whitespace();
                parse_alpha_value(p)
            })
            .unwrap_or(u8::MAX);
        parser.skip_whitespace();

        Ok(Self {
            red,
            green,
            blue,
            alpha,
        })
    }

    fn parse_modern_rgb(_parser: &mut Parser) -> Result<Self, ParseError> {
        // NOTE: The spec defines modern-rgb and modern-rgba
        //       But they are identical, so we do not differentiate between them
        todo!()
    }

    fn parse_rgb_function(parser: &mut Parser) -> Result<Self, ParseError> {
        if let Some(Token::Function(function_identifier)) = parser.next_token() {
            if function_identifier != "rgb" && function_identifier != "rgba" {
                return Err(ParseError);
            }

            if let Some(color) = parser.parse_optional_value(Self::parse_legacy_rgb) {
                parser.expect_token(Token::ParenthesisClose)?;
                return Ok(color);
            }

            let color = Self::parse_modern_rgb(parser)?;
            parser.expect_token(Token::ParenthesisClose)?;
            Ok(color)
        } else {
            Err(ParseError)
        }
    }
}

impl<'a> CSSParse<'a> for Color {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, ParseError> {
        if let Some(color) = parser.parse_optional_value(Self::parse_from_name) {
            return Ok(color);
        }

        if let Some(color) = parser.parse_optional_value(Self::parse_as_hex_color) {
            return Ok(color);
        }

        if let Some(color) = parser.parse_optional_value(Self::parse_rgb_function) {
            return Ok(color);
        }
        Err(ParseError)
    }
}

fn parse_alpha_value(parser: &mut Parser) -> Result<u8, ParseError> {
    let alpha = match parser.next_token() {
        Some(Token::Number(n)) => n.round_to_int().clamp(0, 255) as u8,
        Some(Token::Percentage(p)) => resolve_percentage(p),
        _ => return Err(ParseError),
    };
    parser.skip_whitespace();
    Ok(alpha)
}

fn resolve_percentage(percentage: Number) -> u8 {
    let clamped_percent = match percentage {
        Number::Number(f) => f.clamp(0., 100.),
        Number::Integer(i) => i.clamp(0, 100) as f32,
    };
    (clamped_percent * 2.55).round() as u8
}

#[cfg(test)]
mod tests {
    use super::Color;
    use crate::css::CSSParse;

    #[test]
    fn parse_color_name() {
        assert_eq!(
            Color::parse_from_str("mistyrose"),
            Ok(Color::rgb(255, 228, 225))
        );
    }

    #[test]
    fn parse_hex_color_code() {
        // 6 digit hex color
        assert_eq!(
            Color::parse_from_str("#F00f10"),
            Ok(Color::rgb(0xF0, 0x0F, 0x10))
        );

        // 8 digit hex color
        assert_eq!(
            Color::parse_from_str("#F00f10AB"),
            Ok(Color::rgba(0xF0, 0x0F, 0x10, 0xAB))
        );

        // 3 digit hex color
        assert_eq!(
            Color::parse_from_str("#abc"),
            Ok(Color::rgb(0xAA, 0xBB, 0xCC))
        );

        // 4 digit hex color
        assert_eq!(
            Color::parse_from_str("#abcd"),
            Ok(Color::rgba(0xAA, 0xBB, 0xCC, 0xDD))
        );
    }

    #[test]
    fn parse_legacy_rgb() {
        // legacy syntax without alpha value
        assert_eq!(
            Color::parse_from_str("rgb(100%, 50.0%, 10%)"),
            Ok(Color::rgb(255, 128, 26))
        );

        // legacy syntax with alpha value
        assert_eq!(
            Color::parse_from_str("rgb(100%, 50.0%, 10%, 1)"),
            Ok(Color::rgba(255, 128, 26, 1))
        );

        // legacy syntax with alpha %
        assert_eq!(
            Color::parse_from_str("rgb(100%, 50.0%, 10%, 1%)"),
            Ok(Color::rgba(255, 128, 26, 3))
        );
    }
}
