pub type ParseResult<In, Out> = Result<(In, Out), usize>;

pub trait Parser {
    type In: ?Sized;
    type Out;

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out>;
}

pub trait ParserCombinator: Parser + Sized {
    fn then<P: Parser<In = Self::In>>(self, other: P) -> ChainedParser<Self, P> {
        ChainedParser {
            first: self,
            second: other,
        }
    }

    fn map<O, F: Fn(Self::Out) -> O>(self, map_fn: F) -> MappingParser<O, Self, F> {
        MappingParser {
            parser: self,
            map_fn: map_fn,
        }
    }

    fn or<P: Parser<In = Self::In>>(self, other: P) -> OneOf<Self, P> {
        OneOf {
            first: self,
            second: other,
        }
    }
}

impl<T: Parser + Sized> ParserCombinator for T {}

/// Runs a parser and applies a transformation to the result
#[derive(Clone, Copy)]
pub struct MappingParser<O, P: Parser, F: Fn(P::Out) -> O> {
    parser: P,
    map_fn: F,
}

impl<O, P: Parser, F: Fn(P::Out) -> O> Parser for MappingParser<O, P, F> {
    type In = P::In;
    type Out = O;

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        match self.parser.parse(data) {
            Ok((remaining, resulting)) => Ok((remaining, (self.map_fn)(resulting))),
            Err(e) => Err(e),
        }
    }
}

/// Applies two parsers, returning both results
#[derive(Clone, Copy)]
pub struct ChainedParser<A, B> {
    first: A,
    second: B,
}

impl<T: ?Sized, A: Parser<In = T>, B: Parser<In = T>> Parser for ChainedParser<A, B> {
    type In = T;
    type Out = (A::Out, B::Out);

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        match self.first.parse(&data) {
            Ok((remaining_input, out_first)) => match self.second.parse(remaining_input) {
                Ok((remaining_input, out_second)) => Ok((remaining_input, (out_first, out_second))),
                Err(parsed_until) => Err(parsed_until),
            },
            Err(parsed_until) => Err(parsed_until),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Literal<T: 'static> {
    want: &'static [T],
}

impl<T: Eq> Parser for Literal<T> {
    type In = [T];
    type Out = ();

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        if data.len() < self.want.len() {
            return Err(0);
        }
        if self.want == &data[0..self.want.len()] {
            return Ok((&data[self.want.len()..], ()));
        } else {
            return Err(0);
        }
    }
}

pub const fn literal<T: 'static>(want: &'static [T]) -> Literal<T> {
    Literal { want }
}

/// Applies one parser multiple times (as often as possible, including not at all)
#[derive(Clone, Copy)]
pub struct Many<P: Parser> {
    parser: P,
}

impl<P: Parser> Parser for Many<P> {
    type In = P::In;
    type Out = Vec<P::Out>;

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        let mut remaining_data = data;
        let mut parsed_elements = Vec::new();
        while let Ok((remaining, resulting)) = self.parser.parse(remaining_data) {
            remaining_data = remaining;
            parsed_elements.push(resulting);
        }
        Ok((remaining_data, parsed_elements))
    }
}

pub fn many<P: Parser>(parser: P) -> Many<P> {
    Many { parser }
}

/// Applies one parser multiple times (as often as possible, but at least once)
pub struct Some<P: Parser> {
    parser: P,
}

impl<P: Parser> Parser for Some<P> {
    type In = P::In;
    type Out = Vec<P::Out>;

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        let mut remaining_data = data;
        let mut parsed_elements = Vec::new();

        while let Ok((remaining, resulting)) = self.parser.parse(remaining_data) {
            remaining_data = remaining;
            parsed_elements.push(resulting);
        }

        // At least one element must be parsed
        if parsed_elements.len() == 0 {
            return Err(0);
        }
        Ok((remaining_data, parsed_elements))
    }
}

pub fn some<P: Parser>(parser: P) -> Some<P> {
    Some { parser }
}

#[derive(Clone, Copy)]
pub struct Optional<P> {
    inner: P,
}

impl<P: Parser> Parser for Optional<P> {
    type In = P::In;
    type Out = Option<P::Out>;

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        match self.inner.parse(data) {
            Ok((remaining, resulting)) => Ok((remaining, Some(resulting))),
            Err(_e) => Ok((data, None)),
        }
    }
}

pub fn optional<P: Parser>(inner: P) -> Optional<P> {
    Optional { inner }
}

/// Tries to apply one parser. If that parser fails, tries to apply another one.
pub struct OneOf<P1, P2> {
    first: P1,
    second: P2,
}

#[derive(Debug, PartialEq)]
pub enum Either<A, B> {
    First(A),
    Second(B),
}

impl<I: ?Sized, P1: Parser<In = I>, P2: Parser<In = I>> Parser for OneOf<P1, P2> {
    type In = I;
    type Out = Either<P1::Out, P2::Out>;

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        match self.first.parse(data) {
            Ok((remaining, resulting)) => Ok((remaining, Either::First(resulting))),
            Err(_) => match self.second.parse(data) {
                Ok((remaining, resulting)) => Ok((remaining, Either::Second(resulting))),
                Err(e) => Err(e),
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct PredicateParser<I: ?Sized, O, F: for<'a> Fn(&'a I) -> ParseResult<&'a I, O>> {
    predicate: F,
    // rust complains about unused parameters even though they are not unused - TODO: investigate
    _m1: std::marker::PhantomData<I>,
    _m2: std::marker::PhantomData<O>,
}
 
impl<I: ?Sized, O, F: for<'a> Fn(&'a I) -> ParseResult<&'a I, O>> Parser
    for PredicateParser<I, O, F>
{
    type In = I;
    type Out = O;

    fn parse<'a>(&self, data: &'a Self::In) -> ParseResult<&'a Self::In, Self::Out> {
        (self.predicate)(data)
    }
}

pub fn predicate<I: ?Sized, O, F: for<'a> Fn(&'a I) -> ParseResult<&'a I, O>>(
    predicate: F,
) -> PredicateParser<I, O, F> {
    PredicateParser {
        predicate: predicate,
        _m1: std::marker::PhantomData,
        _m2: std::marker::PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let p = literal(b"abc");

        assert_eq!(p.parse(b"abc"), Ok((b"".as_slice(), ())));
        assert_eq!(p.parse(b"def"), Err(0));
    }

    #[test]
    fn test_chained() {
        let p = literal(b"abc").then(literal(b"def"));

        assert_eq!(p.parse(b"def"), Err(0));
        assert_eq!(p.parse(b"abc"), Err(0));
        assert_eq!(p.parse(b"abcdef"), Ok((b"".as_slice(), ((), ()))));
    }

    #[test]
    fn test_optional() {
        let p = optional(literal(b"abc"));

        assert_eq!(p.parse(b"abc"), Ok((b"".as_slice(), Some(()))));
        assert_eq!(p.parse(b"def"), Ok((b"def".as_slice(), None)));
    }

    #[test]
    fn test_map() {
        let p = literal(b"abc").map(|_| 1);
        assert_eq!(p.parse(b"abc"), Ok((b"".as_slice(), 1)));
        assert_eq!(p.parse(b"def"), Err(0));
    }

    #[test]
    fn test_many() {
        let p = many(literal(b"abc"));

        assert_eq!(p.parse(b"abc"), Ok((b"".as_slice(), vec![()])));
        assert_eq!(
            p.parse(b"abcabcabcd"),
            Ok((b"d".as_slice(), vec![(), (), ()]))
        );
        assert_eq!(p.parse(b"def"), Ok((b"def".as_slice(), vec![])));
    }

    #[test]
    fn test_some() {
        let p = some(literal(b"abc"));

        assert_eq!(p.parse(b"abc"), Ok((b"".as_slice(), vec![()])));
        assert_eq!(
            p.parse(b"abcabcabcd"),
            Ok((b"d".as_slice(), vec![(), (), ()]))
        );
        assert_eq!(p.parse(b"def"), Err(0));
    }

    #[test]
    fn test_or() {
        let p = literal(b"abc").or(literal(b"def"));

        assert_eq!(p.parse(b"abc"), Ok((b"".as_slice(), Either::First(()))));
        assert_eq!(p.parse(b"def"), Ok((b"".as_slice(), Either::Second(()))));
    }

    #[test]
    fn test_predicate() {
        let p = predicate(|x: &[u8]| if x == b"abc" { Ok((b"", 1)) } else { Err(0) });

        assert_eq!(p.parse(b"abc"), Ok((b"".as_slice(), 1)));
        assert_eq!(p.parse(b"def"), Err(0));
    }
}
