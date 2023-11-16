use math::{AffineTransform, Angle, Bitmap, Color, Rectangle, Vec2D};

use crate::{FlattenedPathPoint, Mask, Path, Rasterizer};

#[derive(Clone, Copy, Debug)]
pub enum Source {
    /// One single color
    Solid(Color), // TODO: add more sources, like images and gradients
}

impl Default for Source {
    fn default() -> Self {
        Self::Solid(Color::default())
    }
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub outline: Path,
    pub source: Source,

    /// A common transformation applied to all elements in the layer
    transform: AffineTransform,

    /// Controls whether or not a [Layer]'s contents should be rendered to the screen
    pub is_enabled: bool,
    needs_flattening: bool,
    flattened_outline: Vec<FlattenedPathPoint>,
}

impl Layer {
    /// Show the layer
    #[inline]
    pub fn enable(&mut self) -> &mut Self {
        self.is_enabled = true;
        self
    }

    /// Hide the layer
    #[inline]
    pub fn disable(&mut self) -> &mut Self {
        self.is_enabled = false;
        self
    }

    /// Draw text to the layer. This replaces any existing paths within
    /// the layer
    #[inline]
    pub fn text(
        &mut self,
        text: &str,
        fontface: font::Font,
        font_size: f32,
        offset: Vec2D,
    ) -> &mut Self {
        self.outline = Path::new(Vec2D::new(0., 0.));
        fontface.render(text, &mut self.outline, font_size, offset);
        self
    }

    /// Set the color source of the elements within the [Layer]
    #[inline]
    pub fn with_source(&mut self, source: Source) -> &mut Self {
        self.source = source;
        self
    }

    /// Set the outline of the layer
    #[inline]
    pub fn with_outline(&mut self, path: Path) -> &mut Self {
        self.outline = path;
        self
    }

    /// Rotate the layer by a fixed angle
    ///
    /// This operation does not cause the Bézier curves to be re-flattened
    #[inline]
    pub fn rotate(&mut self, angle: Angle) -> &mut Self {
        self.transform = self.transform.chain(AffineTransform::rotate(angle));
        self
    }

    /// Move the layer by a fixed amount
    ///
    /// This operation does not cause the Bézier curves to be re-flattened
    #[inline]
    pub fn translate(&mut self, translate_by: Vec2D) -> &mut Self {
        self.transform = self
            .transform
            .chain(AffineTransform::translate(translate_by));
        self
    }

    /// Scale the layer by a fixed amount along both axis
    ///
    /// This operation causes the Bézier curves to be re-flattened
    #[inline]
    pub fn scale(&mut self, x_scale: f32, y_scale: f32) -> &mut Self {
        if x_scale == 1. && y_scale == 1. {
            return self;
        }

        self.transform = self
            .transform
            .chain(AffineTransform::scale(x_scale, y_scale));
        self.needs_flattening = true;
        self
    }

    fn flatten_if_necessary(&mut self) {
        const FLATTEN_TOLERANCE: f32 = 0.01;
        if self.needs_flattening {
            self.flattened_outline.clear();
            self.outline
                .flatten(FLATTEN_TOLERANCE, &mut self.flattened_outline)
        }
    }

    fn apply_transform(&mut self) -> Option<Rectangle> {
        // Transform the outline
        self.flattened_outline
            .iter_mut()
            .for_each(|p| p.coordinates = self.transform.apply_to(p.coordinates));

        // Compute extents of the transformed outline

        self.flattened_outline
            .iter()
            .map(|p| p.coordinates)
            .fold(None, |extent, point| {
                extent
                    .map(|extent| {
                        let top_left = extent.top_left();
                        let bottom_right = extent.bottom_right();

                        Rectangle::from_corners(
                            Vec2D::new(
                                f32::min(top_left.x, point.x),
                                f32::min(top_left.y, point.y),
                            ),
                            Vec2D::new(
                                f32::max(bottom_right.x, point.x),
                                f32::max(bottom_right.y, point.y),
                            ),
                        )
                    })
                    .or(Some(Rectangle::from_corners(point, point)))
            })
    }

    pub(crate) fn render_to(&mut self, bitmap: &mut Bitmap<u32>) {
        self.flatten_if_necessary();

        if let Some(outline_extent) = self.apply_transform() {
            // Compute a mask for the layer.
            // This mask determines which pixels in the bitmap should be
            // colored and which should not be.
            let outline_offset = outline_extent.top_left();
            let outline_extent = outline_extent.snap_to_grid();
            let mut rasterizer = Rasterizer::new(outline_extent, outline_offset);
            rasterizer.fill(&self.flattened_outline);
            let mask = rasterizer.into_mask();

            // Compose the mask onto the buffer
            compose(bitmap, mask, self.source, outline_extent.top_left());
        }
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            outline: Path::empty(),
            source: Source::default(),
            transform: AffineTransform::identity(),
            is_enabled: true,
            needs_flattening: true,
            flattened_outline: vec![],
        }
    }
}

fn compose(bitmap: &mut Bitmap<u32>, mask: Mask, source: Source, offset: Vec2D<usize>) {
    if offset.x < bitmap.width() && offset.y < bitmap.height() {
        // Don't draw out of bounds
        let available_space = Vec2D::new(bitmap.width() - offset.x, bitmap.height() - offset.y);
        match source {
            Source::Solid(color) => {
                for x in 0..mask.width().min(available_space.x) {
                    for y in 0..mask.height().min(available_space.y) {
                        let opacity = mask.opacity_at(x, y).abs().min(1.);
                        let previous_color = bitmap.get_pixel(x + offset.x, y + offset.y);
                        let computed_color = color.interpolate(Color(previous_color), opacity);
                        bitmap.set_pixel(x + offset.x, y + offset.y, computed_color.into());
                    }
                }
            },
        }
    }
}
