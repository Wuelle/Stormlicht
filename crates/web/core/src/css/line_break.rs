use super::{layout::Pixels, FontMetrics};

/// Breaks Paragraphs into lines based on their width
pub struct LineBreakIterator<'a> {
    /// The maximum width available for individual line boxes.
    ///
    /// Note that this is just a guideline, line boxes may overflow
    /// if they cannot be broken up.
    available_width: Pixels,
    font_metrics: FontMetrics,
    text: &'a str,
    is_done: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct TextLine<'a> {
    pub text: &'a str,
    pub width: Pixels,
}

impl<'a> LineBreakIterator<'a> {
    #[inline]
    #[must_use]
    pub const fn new(text: &'a str, font_metrics: FontMetrics, available_width: Pixels) -> Self {
        Self {
            text,
            font_metrics,
            available_width,
            is_done: text.is_empty(),
        }
    }

    pub fn adjust_available_width(&mut self, available_width: Pixels) {
        self.available_width = available_width;
    }

    pub const fn is_done(&self) -> bool {
        self.is_done
    }
}

impl<'a> Iterator for LineBreakIterator<'a> {
    type Item = TextLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        let mut previous_potential_breakpoint = None;
        let potential_breaks = self
            .text
            .match_indices(char::is_whitespace)
            .map(|(index, _)| index);

        for break_point in potential_breaks {
            let (line, remainder) = self.text.split_at(break_point);

            if line.is_empty() {
                continue;
            }

            let width = Pixels(
                self.font_metrics
                    .font_face
                    .compute_rendered_width(line, self.font_metrics.size.into()),
            );

            if width <= self.available_width {
                // No need to break yet
                previous_potential_breakpoint = Some((line, remainder, width));
                continue;
            } else {
                // We've exceeded the available space
                match previous_potential_breakpoint {
                    Some((line, remainder, width)) => {
                        // There was a valid potential breakpoint, let's use that one instead
                        self.text = remainder;
                        return Some(TextLine { text: line, width });
                    },
                    None => {
                        // Our line is too wide, but there was no opportunity to split it.
                        // Let's just return it as a whole
                        self.text = remainder;
                        return Some(TextLine { text: line, width });
                    },
                }
            }
        }

        // There are no further opportunities to split this text
        let width = Pixels(
            self.font_metrics
                .font_face
                .compute_rendered_width(self.text, self.font_metrics.size.into()),
        );

        match (self.available_width < width, previous_potential_breakpoint) {
            (true, Some((line, remainder, width))) => {
                // We don't have enough space for the entire remainder *and*
                // there was a valid potential breakpoint before, let's use that one instead
                self.text = remainder;
                Some(TextLine { text: line, width })
            },
            (false, _) | (_, None) => {
                self.is_done = true;

                Some(TextLine {
                    text: self.text,
                    width,
                })
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LineBreakIterator;
    use crate::css::{layout::Pixels, FontMetrics};

    #[test]
    fn do_not_break_empty_text() {
        // When iterating over line breaks of empty text, we should produce no lines at all
        // (as opposed to one empty line)
        let font_metrics = FontMetrics {
            font_face: Box::default(),
            size: Pixels::ZERO,
        };

        let mut lines = LineBreakIterator::new("", font_metrics, Pixels::ZERO);
        assert!(lines.next().is_none());
    }
}
