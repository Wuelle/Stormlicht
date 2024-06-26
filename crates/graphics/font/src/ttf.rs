//! [TrueType](https://developer.apple.com/fonts/TrueType-Reference-Manual) font parser
//!
//! ## Reference Material:
//! * <https://learn.microsoft.com/en-us/typography/opentype/spec/otff>
//! * <https://formats.kaitai.io/ttf/index.html>
//! * <https://handmade.network/forums/articles/t/7330-implementing_a_font_reader_and_rasterizer_from_scratch%252C_part_1__ttf_font_reader>

use std::{fmt, iter};

use crate::{
    hinting::Interpreter,
    path::{Operation, PathConsumer, PathReader},
    ttf_tables::{
        cmap::{self, GlyphID},
        glyf::{self, CompoundGlyph, Glyph, GlyphPointIterator, Metrics},
        head, hhea, hmtx, loca, maxp, name,
        offset::OffsetTable,
    },
};

const DEFAULT_FONT: &[u8; 168644] = include_bytes!(concat!(
    env!("DOWNLOAD_DIR"),
    "/fonts/roboto/Roboto-Medium.ttf"
));

const CMAP_TAG: u32 = u32::from_be_bytes(*b"cmap");
const HEAD_TAG: u32 = u32::from_be_bytes(*b"head");
const LOCA_TAG: u32 = u32::from_be_bytes(*b"loca");
const GLYF_TAG: u32 = u32::from_be_bytes(*b"glyf");
const HHEA_TAG: u32 = u32::from_be_bytes(*b"hhea");
const HMTX_TAG: u32 = u32::from_be_bytes(*b"hmtx");
const MAXP_TAG: u32 = u32::from_be_bytes(*b"maxp");
const NAME_TAG: u32 = u32::from_be_bytes(*b"name");
const _VHEA_TAG: u32 = u32::from_be_bytes(*b"vhea");
const PREP_TAG: u32 = u32::from_be_bytes(*b"prep");
const FPGM_TAG: u32 = u32::from_be_bytes(*b"fpgm");

#[derive(Clone, Copy, Debug)]
pub enum TTFParseError {
    UnexpectedEOF,
    UnsupportedFormat,
    MissingTable,
}

#[derive(Clone)]
pub struct Font {
    offset_table: OffsetTable,
    head_table: head::HeadTable,
    format4: cmap::Format4,
    glyph_table: glyf::GlyphOutlineTable,
    hmtx_table: hmtx::HMTXTable,
    maxp_table: maxp::MaxPTable,
    name_table: name::NameTable,

    /// A program that is run once the font is loaded and whenever its environment changes
    ///
    /// Stored inside the `prep` table
    control_value_program: Option<Vec<u8>>,
    interpreter: Interpreter,
    is_instructed: bool,
}

impl Font {
    pub fn new(data: &[u8]) -> Result<Self, TTFParseError> {
        let offset_table = OffsetTable::new(data);
        if offset_table.scaler_type() != 0x00010000 {
            return Err(TTFParseError::UnsupportedFormat);
        }

        let head_entry = offset_table
            .get_table(HEAD_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let head_table = head::HeadTable::new(data, head_entry.offset());

        let cmap_entry = offset_table
            .get_table(CMAP_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let cmap_table = cmap::CMAPTable::new(data, cmap_entry.offset());

        let unicode_table_offset = cmap_table
            .get_unicode_table()
            .ok_or(TTFParseError::MissingTable)?;
        let format4 = cmap::Format4::new(&data[cmap_entry.offset() + unicode_table_offset..]);

        let maxp_entry = offset_table
            .get_table(MAXP_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let maxp_table = maxp::MaxPTable::new(&data[maxp_entry.offset()..]);

        let loca_entry = offset_table
            .get_table(LOCA_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let loca_table = loca::LocaTable::new(
            &data[loca_entry.offset()..],
            head_table.loca_table_format(),
            maxp_table.num_glyphs as usize,
        );

        let glyf_entry = offset_table
            .get_table(GLYF_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let glyph_table = glyf::GlyphOutlineTable::new(
            data,
            glyf_entry.offset(),
            glyf_entry.length(),
            loca_table,
        );
        let hhea_entry = offset_table
            .get_table(HHEA_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let hhea_table = hhea::HHEATable::new(data, hhea_entry.offset());

        let hmtx_entry = offset_table
            .get_table(HMTX_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let hmtx_table = hmtx::HMTXTable::new(
            &data[hmtx_entry.offset()..],
            hhea_table.num_of_long_hor_metrics(),
        );

        let name_entry = offset_table
            .get_table(NAME_TAG)
            .ok_or(TTFParseError::MissingTable)?;
        let name_table = name::NameTable::new(&data[name_entry.offset()..]).unwrap();

        let mut interpreter = Interpreter::new(
            maxp_table.max_storage as usize,
            maxp_table.max_function_defs as usize,
        );

        // If there is a fpgm table, execute the font program inside it.
        // If no such table exists then the font is not instructed
        let is_instructed = offset_table
            .get_table(FPGM_TAG)
            .map(|fpgm_entry| &data[fpgm_entry.offset()..][..fpgm_entry.length()])
            .filter(|p| {
                let result = interpreter.run(p);
                if let Err(error) = result {
                    log::error!("Failed to run font program: {error:?}. Treating the font as not-instructed")
                }
                result.is_err()
            })
            .is_some();

        // If there is a prep table
        let control_value_program = if is_instructed {
            offset_table
            .get_table(PREP_TAG)
            .map(|prep_entry| data[prep_entry.offset()..][..prep_entry.length()].to_owned())
            .filter(|p| {
                let result = interpreter.run(p);
                if let Err(error) = result {
                    log::error!("Failed to run prep table program: {error:?}. Ignoring prep table in the future (this might break the font)")
                }
                result.is_err()
            })
        } else {
            None
        };

        Ok(Self {
            offset_table,
            head_table,
            format4,
            glyph_table,
            hmtx_table,
            maxp_table,
            name_table,
            control_value_program,
            interpreter,
            is_instructed,
        })
    }

    pub fn rerun_prep_program(&mut self) {
        if self.is_instructed {
            if let Some(program) = &self.control_value_program {
                if let Err(error) = self.interpreter.run(program) {
                    log::error!("Failed to run prep table program: {error:?}");
                }
            }
        }
    }

    /// Get the total number of glyphs defined in the font
    pub fn num_glyphs(&self) -> usize {
        self.maxp_table.num_glyphs as usize
    }

    // TODO: support more than one cmap format table (format 4 seems to be the most common but still)
    pub fn format_4(&self) -> &cmap::Format4 {
        &self.format4
    }

    /// Get the full name of the font, if specified.
    /// Fonts will usually specify their own name, though it is not required.
    pub fn name(&self) -> Option<&str> {
        self.name_table.get_font_name()
    }

    pub fn glyf(&self) -> &glyf::GlyphOutlineTable {
        &self.glyph_table
    }

    pub fn hmtx(&self) -> &hmtx::HMTXTable {
        &self.hmtx_table
    }

    pub fn head(&self) -> &head::HeadTable {
        &self.head_table
    }

    pub fn offset_table(&self) -> &OffsetTable {
        &self.offset_table
    }

    /// Get the Glyph index for a given codepoint
    pub fn get_glyph_id(&self, codepoint: u16) -> Option<GlyphID> {
        self.format4.get_glyph_id(codepoint)
    }

    pub fn get_glyph(&self, glyph_id: GlyphID) -> Result<Glyph<'_>, TTFParseError> {
        // Any character that does not exist is mapped to index zero, which is defined to be the
        // missing character glyph
        let glyph = self.glyph_table.get_glyph(glyph_id);
        Ok(glyph)
    }

    /// Return the number of coordinate points per font size unit.
    /// This value is used to scale fonts, ie. when you render a font with
    /// size `17px`, one `em` equals `17px`.
    ///
    /// Note that this value does not constrain the size of individual glyphs.
    /// A glyph may have a size larger than `1em`.
    #[inline]
    #[must_use]
    pub fn units_per_em(&self) -> f32 {
        self.head_table.units_per_em() as f32
    }

    // Returns a substring of text that has a specified width
    pub fn find_prefix_with_width<'text>(
        &self,
        text: &'text str,
        font_size: f32,
        available_width: f32,
    ) -> &'text str {
        let mut glyph_positions = GlyphPositionIterator::new(self, text);
        while glyph_positions.next().is_some() {
            if available_width < (glyph_positions.x as f32 * font_size) / self.units_per_em() {
                return &text[..text.len() - glyph_positions.remainder().len()];
            }
        }

        // If the total length of the string does not exceed the specified width then
        // we simply return the entire string
        text
    }

    pub fn compute_rendered_width(&self, text: &str, font_size: f32) -> f32 {
        let mut glyph_positions = GlyphPositionIterator::new(self, text);
        while glyph_positions.next().is_some() {}

        (glyph_positions.x as f32 * font_size) / self.units_per_em()
    }

    pub fn render<P: PathConsumer>(
        &self,
        text: &str,
        renderer: &mut P,
        font_size: f32,
        text_offset: math::Vec2D,
    ) {
        for glyph in RenderedGlyphIterator::new(self, text) {
            let scale_point = |glyph_point: math::Vec2D<i32>| math::Vec2D {
                x: (glyph_point.x as f32 * font_size) / self.units_per_em(),
                y: font_size - (glyph_point.y as f32 * font_size) / self.units_per_em(),
            };

            // Draw the outlines of the glyph on the rasterizer buffer
            // Note: all the coordinates in the path operations are relative to the glyph positiont;
            for path_op in glyph.path_operations {
                match path_op {
                    Operation::MoveTo(destination) => {
                        renderer.move_to(scale_point(destination + glyph.position) + text_offset);
                    },
                    Operation::LineTo(destination) => {
                        let scaled_destination =
                            scale_point(destination + glyph.position) + text_offset;
                        renderer.line_to(scaled_destination);
                    },
                    Operation::QuadBezTo(p1, p2) => {
                        let scaled_p1 = scale_point(p1 + glyph.position) + text_offset;
                        let scaled_p2 = scale_point(p2 + glyph.position) + text_offset;
                        renderer.quad_bez_to(scaled_p1, scaled_p2);
                    },
                }
            }
        }
    }

    pub fn render_as_svg(&self, text: &str, id_prefix: &str) -> String {
        let mut min_x = 0;
        let mut max_x = 0;
        let mut min_y = 0;
        let mut max_y = 0;

        let mut symbols = Vec::with_capacity(text.len());
        let mut symbol_positions = Vec::with_capacity(text.len());
        let path_objects: Vec<RenderedGlyph<'_>> = RenderedGlyphIterator::new(self, text).collect();

        // SVG uses a different coordinate space than our font renderer
        // We therefore have to create run two passes over the text:
        // First pass calculates the textbox dimensions
        // Second pass renders the actual glyphs
        for glyph in &path_objects {
            min_x = min_x.min(glyph.position.x + glyph.metrics.min_x as i32);
            min_y = min_y.min(glyph.position.y + glyph.metrics.min_y as i32);
            max_x = max_x.max(glyph.position.x + glyph.metrics.max_x as i32);
            max_y = max_y.max(glyph.position.y + glyph.metrics.max_y as i32);
        }

        for (index, glyph) in path_objects.into_iter().enumerate() {
            symbol_positions.push(glyph.position);

            let mut glyph_path = glyph
                .path_operations
                .map(|operation| match operation {
                    Operation::MoveTo(math::Vec2D { x, y }) => {
                        format!("M{x} {}", y)
                    },
                    Operation::LineTo(math::Vec2D { x, y }) => {
                        format!("L{x} {}", y)
                    },
                    Operation::QuadBezTo(
                        math::Vec2D { x: x1, y: y1 },
                        math::Vec2D { x: x2, y: y2 },
                    ) => {
                        format!("Q{} {} {} {}", x1, y1, x2, y2)
                    },
                })
                .collect::<Vec<String>>()
                .join(" ");
            glyph_path.push_str(" Z");
            symbols.push(format!(
                "<symbol id=\"{id_prefix}/{index}\" overflow=\"visible\"><path d=\"{glyph_path}\"></path></symbol>"
            ));
        }

        let symbol_uses = symbol_positions
            .iter()
            .enumerate()
            .map(|(index, math::Vec2D { x, y })| {
                format!("<use xlink:href=\"#{id_prefix}/{index}\" x=\"{x}\" y=\"{y}\"/>")
            })
            .collect::<Vec<String>>()
            .join("");

        let width = max_x - min_x;
        let height = max_y - min_y;
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <svg version=\"1.1\"
            xmlns=\"http://www.w3.org/2000/svg\"
            xmlns:xlink=\"http://www.w3.org/1999/xlink\"
            viewBox=\"{min_x} {min_y} {width} {height}\">
          {} {}
        </svg>",
            symbols.join(""),
            symbol_uses,
        )
    }

    /// A font that can be used for testing, or if no other font is available
    pub fn fallback() -> Self {
        Self::new(DEFAULT_FONT).expect("Fallback font file is valid")
    }
}

pub fn read_u16_at(data: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(data[offset..offset + 2].try_into().unwrap())
}

pub fn read_u32_at(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap())
}

pub fn read_i16_at(data: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes(data[offset..offset + 2].try_into().unwrap())
}

pub struct RenderedGlyphIterator<'a, 'b> {
    glyphs: GlyphPositionIterator<'a, 'b>,
    current_compound_glyphs: Vec<CompoundGlyph<'a>>,

    /// The x coordinate that any compound glyph components positions are relative to
    x: i32,

    /// The y coordinate that any compound glyph components positions are relative to
    y: i32,
}

struct GlyphPositionIterator<'font, 'text> {
    font: &'font Font,
    x: i32,
    y: i32,
    chars: std::str::Chars<'text>,
}

#[derive(Clone, Copy, Debug)]
struct PositionedGlyph {
    x: i32,
    y: i32,
    id: GlyphID,
}

pub struct RenderedGlyph<'a> {
    metrics: Metrics,
    position: math::Vec2D<i32>,
    path_operations: PathReader<GlyphPointIterator<'a>>,
}

impl<'font, 'text> Iterator for GlyphPositionIterator<'font, 'text> {
    type Item = PositionedGlyph;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.chars.next()?;

        let id = self
            .font
            .get_glyph_id(c as u16)
            .unwrap_or(GlyphID::REPLACEMENT);

        let horizontal_metrics = self.font.hmtx_table.get_metric_for(id);
        let x = self.x + horizontal_metrics.left_side_bearing() as i32;
        let y = self.y;

        self.x += horizontal_metrics.advance_width() as i32;

        Some(PositionedGlyph { x, y, id })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.chars.size_hint()
    }
}

impl iter::FusedIterator for GlyphPositionIterator<'_, '_> {}

impl<'a, 'b> Iterator for RenderedGlyphIterator<'a, 'b> {
    type Item = RenderedGlyph<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Determine which glyph we should render and where we should render it to.
        // If we are currently in the process of emitting the components of some compound glyph, continue doing that
        // else, read the next character and emit that
        let positioned_glyph = if let Some(current_glyph) = self.current_compound_glyphs.last_mut()
        {
            if let Some(component) = current_glyph.next() {
                PositionedGlyph {
                    id: component.glyph_id,
                    x: self.x + component.x_offset as i32,
                    y: self.y + component.y_offset as i32,
                }
            } else {
                // We are done emitting all parts of the current component glyph, pop it from the stack and start again
                self.current_compound_glyphs.pop();
                return self.next();
            }
        } else {
            self.glyphs.next()?
        };

        let glyph = self
            .glyphs
            .font
            .get_glyph(positioned_glyph.id)
            .expect("Font contains no glyph for glyph id");

        match glyph {
            Glyph::Empty => {
                // Nothing to do, return the next glyph
                self.next()
            },
            Glyph::Simple(simple_glyph) => {
                let path_operations = PathReader::new(simple_glyph.into_iter());
                Some(RenderedGlyph {
                    metrics: simple_glyph.metrics,
                    position: math::Vec2D::new(positioned_glyph.x, positioned_glyph.y),
                    path_operations,
                })
            },
            Glyph::Compound(compound_glyph) => {
                self.x = positioned_glyph.x;
                self.y = positioned_glyph.y;
                self.current_compound_glyphs.push(compound_glyph);
                self.next()
            },
        }
    }
}

impl iter::FusedIterator for RenderedGlyphIterator<'_, '_> {}

impl<'font, 'text> GlyphPositionIterator<'font, 'text> {
    #[inline]
    #[must_use]
    pub fn new(font: &'font Font, text: &'text str) -> Self {
        Self {
            font,
            x: 0,
            y: 0,
            chars: text.chars(),
        }
    }

    #[inline]
    #[must_use]
    pub fn remainder(&self) -> &'text str {
        self.chars.as_str()
    }
}

impl<'a, 'b> RenderedGlyphIterator<'a, 'b> {
    #[inline]
    #[must_use]
    pub fn new(font: &'a Font, text: &'b str) -> Self {
        Self {
            glyphs: GlyphPositionIterator::new(font, text),
            current_compound_glyphs: vec![],
            x: 0,
            y: 0,
        }
    }
}

impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{name}")
        } else {
            write!(f, "<Unnamed Font>")
        }
    }
}
