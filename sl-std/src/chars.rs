#[derive(Clone, Copy, Debug)]
pub enum State {
    BeforeStart(usize),
    Within,
    AfterEnd(usize),
}

#[derive(Clone, Copy, Debug)]
pub struct ReversibleCharIterator<T> {
    source: T,
    /// The current byte position of the iterator
    ///
    /// At all times, this is guaranteed to point to a character boundary
    /// (which is not the end of the string) in [source](Self::source).
    pos: usize,
    state: State,
}

impl<T> ReversibleCharIterator<T>
where
    T: AsRef<str>,
{
    #[inline]
    #[must_use]
    pub fn new(source: T) -> Self {
        Self {
            source,
            pos: 0,
            state: State::Within,
        }
    }

    #[inline]
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_ref()
    }

    #[inline]
    #[must_use]
    pub fn state(&self) -> State {
        self.state
    }

    #[inline]
    #[must_use]
    pub fn remaining(&self) -> &str {
        &self.source.as_ref()[self.pos..]
    }

    pub fn go_back(&mut self) {
        match self.state {
            State::BeforeStart(ref mut n) => {
                *n += 1;
            },
            State::Within => {
                if self.pos == 0 {
                    self.state = State::BeforeStart(0);
                } else {
                    debug_assert!(self.source().is_char_boundary(self.pos));

                    // Find the byte position of the previous character
                    self.pos = self.source().floor_char_boundary(self.pos - 1);
                }
            },
            State::AfterEnd(ref mut n) => {
                if *n == 0 {
                    self.state = State::Within;
                } else {
                    *n -= 1;
                }
            },
        }
    }

    #[inline]
    pub fn go_back_n(&mut self, n: usize) {
        for _ in 0..n {
            self.go_back();
        }
    }

    /// Set the iterator position manually
    ///
    /// # Panics
    /// This function panics if the specified byte position is not a
    /// character boundary.
    pub fn set_position(&mut self, pos: usize) {
        assert!(self.source.as_ref().is_char_boundary(pos));
        self.state = State::Within;
        self.pos = pos;
    }

    pub fn current(&self) -> Option<char> {
        if let State::Within = self.state {
            let c = self.source()[self.pos..]
                .chars()
                .nth(0)
                .expect("pos was a char boundary");
            Some(c)
        } else {
            None
        }
    }
}

impl<T> Iterator for ReversibleCharIterator<T>
where
    T: AsRef<str>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::BeforeStart(ref mut n) => {
                if *n == 0 {
                    self.state = State::Within;
                } else {
                    *n -= 1;
                }

                None
            },
            State::Within => {
                debug_assert!(self.source().is_char_boundary(self.pos));

                let c = self.source()[self.pos..]
                    .chars()
                    .nth(0)
                    .expect("pos was a char boundary");

                let length = c.len_utf8();

                // Ensure that self.pos never points past the end of source
                if self.pos + length != self.source().len() {
                    self.pos += c.len_utf8();
                } else {
                    self.state = State::AfterEnd(0)
                }

                Some(c)
            },
            State::AfterEnd(ref mut n) => {
                *n += 1;
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReversibleCharIterator;

    #[test]
    fn forward_backward() {
        let mut iter = ReversibleCharIterator::new("💚💙💜");

        // Forward pass, expect all characters in order
        assert_eq!(iter.next(), Some('💚'));
        assert_eq!(iter.next(), Some('💙'));
        assert_eq!(iter.next(), Some('💜'));

        // Consume one character past the end
        assert_eq!(iter.next(), None);

        // Return to the middle of the string
        iter.go_back_n(2);
        assert_eq!(iter.next(), Some('💜'));
        assert_eq!(iter.next(), None);

        // Go before the start of the string
        iter.go_back_n(5);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), Some('💚'));
    }
}
