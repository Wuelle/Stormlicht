pub mod flow;
mod pixels;

use math::{Rectangle, Vec2D};
pub use pixels::CSSPixels;

use std::ops;

#[derive(Clone, Copy, Debug)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> Sides<T> {
    pub fn surround(&self, area: Rectangle<T>) -> Rectangle<T>
    where
        T: Copy,
        Vec2D<T>: ops::Add<Vec2D<T>, Output = Vec2D<T>> + ops::Sub<Vec2D<T>, Output = Vec2D<T>>,
    {
        Rectangle {
            top_left: area.top_left
                - Vec2D {
                    x: self.left,
                    y: self.top,
                },
            bottom_right: area.bottom_right
                + Vec2D {
                    x: self.right,
                    y: self.bottom,
                },
        }
    }
}

impl<T: Copy> Sides<T> {
    pub const fn all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

#[derive(Clone, Copy, Debug)]
pub struct ContainingBlock {
    width: CSSPixels,
    /// The height of the containing block
    ///
    /// `Some` if the height is defined (for example, using the CSS "height" property)
    /// or `None` if the height depends on the content.
    height: Option<CSSPixels>,
}

impl ContainingBlock {
    #[inline]
    #[must_use]
    pub const fn new(width: CSSPixels) -> Self {
        Self {
            width,
            height: None,
        }
    }

    pub const fn with_height(mut self, height: CSSPixels) -> Self {
        self.height = Some(height);
        self
    }

    #[inline]
    #[must_use]
    pub const fn width(&self) -> CSSPixels {
        self.width
    }

    #[inline]
    #[must_use]
    pub const fn height(&self) -> Option<CSSPixels> {
        self.height
    }
}
