mod auto;
mod background_color;
mod border;
mod color;
mod display;
mod float;
mod font_family;
pub mod font_size;
pub mod length;
mod number;
mod percentage;
mod position;

pub use auto::AutoOr;
pub use background_color::BackgroundColor;
pub use border::{Border, LineStyle, LineWidth};
pub use color::Color;
pub use display::Display;
pub use float::{Clear, Float};
pub use font_family::FontFamily;
pub use font_size::FontSize;
pub use length::Length;
pub use number::Number;
pub use percentage::{Percentage, PercentageOr};
pub use position::Position;

pub type Margin = AutoOr<PercentageOr<Length>>;
pub type Padding = PercentageOr<Length>;
