#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/computed_style.rs"));

use super::layout::Sides;

use super::style::computed::Length;

impl ComputedStyle {
    #[must_use]
    pub fn used_border_widths(&self) -> Sides<Length> {
        let left = if self.border_left_style().is_none() {
            Length::ZERO
        } else {
            *self.border_left_width()
        };

        let right = if self.border_right_style().is_none() {
            Length::ZERO
        } else {
            *self.border_right_width()
        };

        let top = if self.border_top_style().is_none() {
            Length::ZERO
        } else {
            *self.border_top_width()
        };

        let bottom = if self.border_bottom_style().is_none() {
            Length::ZERO
        } else {
            *self.border_bottom_width()
        };

        Sides {
            top,
            right,
            bottom,
            left,
        }
    }
}
