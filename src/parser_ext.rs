use crate::{Code, KParseError, ParserError};
use nom::{IResult, InputIter, InputLength, Offset, Parser, Slice};
use std::borrow::Borrow;
use std::error::Error;
use std::marker::PhantomData;
use std::ops::RangeTo;
use std::str::FromStr;

/// Convert the error.
pub struct IntoErr<PA, O, E1, E2> {
    pub(crate) parser: PA,
    pub(crate) _phantom: PhantomData<(O, E1, E2)>,
}

impl<PA, I, O, E1, E2> Parser<I, O, E2> for IntoErr<PA, O, E1, E2>
where
    PA: Parser<I, O, E1>,
    E1: Into<E2>,
{
    fn parse(&mut self, input: I) -> IResult<I, O, E2> {
        match self.parser.parse(input) {
            Ok((r, o)) => Ok((r, o)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.into())),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(e.into())),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}

/// Change the error code.
pub struct WithCode<PA, C> {
    pub(crate) parser: PA,
    pub(crate) code: C,
}

impl<PA, C, I, O, E> Parser<I, O, E> for WithCode<PA, C>
where
    PA: Parser<I, O, E>,
    C: Code,
    E: KParseError<C, I>,
{
    fn parse(&mut self, input: I) -> IResult<I, O, E> {
        match self.parser.parse(input) {
            Ok((r, v)) => Ok((r, v)),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(e.with_code(self.code))),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Error(e.with_code(self.code))),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
        }
    }
}

/// Map the output.
pub struct MapRes<PA, O1, TR, O2> {
    pub(crate) parser: PA,
    pub(crate) map: TR,
    pub(crate) _phantom: PhantomData<(O1, O2)>,
}

impl<PA, TR, I, O1, O2, E> Parser<I, O2, E> for MapRes<PA, O1, TR, O2>
where
    PA: Parser<I, O1, E>,
    TR: Fn(O1) -> Result<O2, nom::Err<E>>,
{
    fn parse(&mut self, input: I) -> IResult<I, O2, E> {
        self.parser
            .parse(input)
            .and_then(|(rest, tok)| Ok((rest, (self.map)(tok)?)))
    }
}

/// Add some context.
pub struct WithContext<PA, C, E, Y> {
    pub(crate) parser: PA,
    pub(crate) context: Y,
    pub(crate) _phantom: PhantomData<(C, E)>,
}

impl<PA, C, I, O, E, Y> Parser<I, O, ParserError<C, I>> for WithContext<PA, C, E, Y>
where
    PA: Parser<I, O, E>,
    C: Code,
    I: Clone,
    E: Into<ParserError<C, I>>,
    Y: Clone + 'static,
{
    fn parse(&mut self, input: I) -> IResult<I, O, ParserError<C, I>> {
        match self.parser.parse(input) {
            Err(nom::Err::Error(e)) => {
                let err: ParserError<C, I> = e.into();
                let err = err.with_user_data(self.context.clone());
                Err(err.error())
            }
            Err(nom::Err::Failure(e)) => {
                let err: ParserError<C, I> = e.into();
                let err = err.with_user_data(self.context.clone());
                Err(err.failure())
            }
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Ok((r, v)) => Ok((r, v)),
        }
    }
}

/// Convert the output with the FromStr trait.
pub struct FromStrParser<PA, C, O1, O2> {
    pub(crate) parser: PA,
    pub(crate) code: C,
    pub(crate) _phantom: PhantomData<(O1, O2)>,
}

impl<PA, C, I, O1, O2, E> Parser<I, O2, E> for FromStrParser<PA, C, O1, O2>
where
    PA: Parser<I, O1, E>,
    O1: InputIter<Item = char>,
    O2: FromStr,
    <O2 as FromStr>::Err: Error,
    C: Code,
    E: KParseError<C, O1> + Error,
{
    fn parse(&mut self, input: I) -> IResult<I, O2, E> {
        match self.parser.parse(input) {
            Ok((rest, token)) => {
                let txt: String = token.iter_elements().collect();
                match O2::from_str(txt.as_ref()) {
                    Ok(value) => Ok((rest, value)),
                    Err(_) => Err(nom::Err::Error(E::from(self.code, token))),
                }
            }
            Err(e) => Err(e),
        }
    }
}

/// Replace the output with the value.
pub struct Value<PA, O1, O2> {
    pub(crate) parser: PA,
    pub(crate) value: O2,
    pub(crate) _phantom: PhantomData<O1>,
}

impl<PA, I, O1, O2, E> Parser<I, O2, E> for Value<PA, O1, O2>
where
    PA: Parser<I, O1, E>,
    O2: Clone,
{
    fn parse(&mut self, input: I) -> IResult<I, O2, E> {
        match self.parser.parse(input) {
            Ok((r, _)) => Ok((r, self.value.clone())),
            Err(e) => Err(e),
        }
    }
}

/// Fails if not everything has been processed.
pub struct AllConsuming<PA, C> {
    pub(crate) parser: PA,
    pub(crate) code: C,
}

impl<PA, C, I, O, E> Parser<I, O, E> for AllConsuming<PA, C>
where
    C: Code,
    PA: Parser<I, O, E>,
    I: InputLength,
    E: KParseError<C, I>,
{
    fn parse(&mut self, input: I) -> IResult<I, O, E> {
        match self.parser.parse(input) {
            Ok((rest, value)) => {
                if rest.input_len() > 0 {
                    Err(nom::Err::Error(E::from(self.code, rest)))
                } else {
                    Ok((rest, value))
                }
            }
            Err(e) => Err(e),
        }
    }
}

/// Converts nom::Err::Incomplete to a error code.
pub struct Complete<PA, C> {
    pub(crate) parser: PA,
    pub(crate) code: C,
}

impl<PA, C, I, O, E> Parser<I, O, E> for Complete<PA, C>
where
    PA: Parser<I, O, E>,
    C: Code,
    E: KParseError<C, I>,
    I: Clone,
{
    fn parse(&mut self, input: I) -> IResult<I, O, E> {
        match self.parser.parse(input.clone()) {
            Err(nom::Err::Incomplete(_)) => Err(nom::Err::Error(E::from(self.code, input))),
            Err(e) => Err(e),
            Ok((r, v)) => Ok((r, v)),
        }
    }
}

/// Convert from nom::Err::Error to nom::Err::Failure
pub struct Cut<PA> {
    pub(crate) parser: PA,
}

impl<PA, I, O, E> Parser<I, O, E> for Cut<PA>
where
    PA: Parser<I, O, E>,
{
    fn parse(&mut self, input: I) -> IResult<I, O, E> {
        match self.parser.parse(input) {
            Err(nom::Err::Error(e)) => Err(nom::Err::Failure(e)),
            Ok((r, v)) => Ok((r, v)),
            Err(e) => Err(e),
        }
    }
}

/// Optional parser.
pub struct Optional<PA> {
    pub(crate) parser: PA,
}

impl<PA, I, O, E> Parser<I, Option<O>, E> for Optional<PA>
where
    PA: Parser<I, O, E>,
    I: Clone,
{
    fn parse(&mut self, input: I) -> IResult<I, Option<O>, E> {
        match self.parser.parse(input.clone()) {
            Ok((r, v)) => Ok((r, Some(v))),
            Err(nom::Err::Error(_)) => Ok((input, None)),
            Err(e) => Err(e),
        }
    }
}

/// Run the parser and return the parsed input.
pub struct Recognize<PA, O> {
    pub(crate) parser: PA,
    pub(crate) _phantom: PhantomData<O>,
}

impl<PA, I, O, E> Parser<I, I, E> for Recognize<PA, O>
where
    PA: Parser<I, O, E>,
    I: Clone + Slice<RangeTo<usize>> + Offset,
{
    fn parse(&mut self, input: I) -> IResult<I, I, E> {
        let (tail, _) = self.parser.parse(input.clone())?;
        let index = input.offset(&tail);
        Ok((tail, input.slice(..index)))
    }
}

/// Run the parser and return the parser output and the parsed input.
pub struct Consumed<PA> {
    pub(crate) parser: PA,
}

impl<PA, I, O, E> Parser<I, (I, O), E> for Consumed<PA>
where
    PA: Parser<I, O, E>,
    I: Clone + Slice<RangeTo<usize>> + Offset,
{
    fn parse(&mut self, input: I) -> IResult<I, (I, O), E> {
        let (tail, output) = self.parser.parse(input.clone())?;
        let index = input.offset(&tail);
        Ok((tail, (input.slice(..index), output)))
    }
}

/// Runs the parser and the terminator and just returns the result of the parser.
pub struct Terminated<PA, PT, O2> {
    pub(crate) parser: PA,
    pub(crate) terminator: PT,
    pub(crate) _phantom: PhantomData<O2>,
}

impl<PA, PT, I, O1, O2, E> Parser<I, O1, E> for Terminated<PA, PT, O2>
where
    PA: Parser<I, O1, E>,
    PT: Parser<I, O2, E>,
{
    fn parse(&mut self, input: I) -> IResult<I, O1, E> {
        match self.parser.parse(input) {
            Ok((rest, val)) => match self.terminator.parse(rest) {
                Ok((rest, _)) => Ok((rest, val)),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

/// Runs the parser and the successor and only returns the result of the
/// successor.
pub struct Precedes<PA, PS, O1> {
    pub(crate) parser: PA,
    pub(crate) successor: PS,
    pub(crate) _phantom: PhantomData<O1>,
}

impl<PA, PS, I, O1, O2, E> Parser<I, O2, E> for Precedes<PA, PS, O1>
where
    PA: Parser<I, O1, E>,
    PS: Parser<I, O2, E>,
{
    fn parse(&mut self, input: I) -> IResult<I, O2, E> {
        match self.parser.parse(input) {
            Ok((rest, _)) => match self.successor.parse(rest) {
                Ok((rest, val)) => Ok((rest, val)),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

/// Runs the parser and the successor and returns the result of the successor.
/// The parser itself may fail too.
pub struct OptPrecedes<PA, PS, O1> {
    pub(crate) parser: PA,
    pub(crate) successor: PS,
    pub(crate) _phantom: PhantomData<O1>,
}

impl<PA, PS, I, O1, O2, E> Parser<I, O2, E> for OptPrecedes<PA, PS, O1>
where
    PA: Parser<I, O1, E>,
    PS: Parser<I, O2, E>,
    I: Clone,
{
    fn parse(&mut self, input: I) -> IResult<I, O2, E> {
        match self.parser.parse(input.clone()) {
            Ok((rest, _)) => match self.successor.parse(rest) {
                Ok((rest, val)) => Ok((rest, val)),
                Err(e) => Err(e),
            },
            Err(nom::Err::Error(_)) => match self.successor.parse(input) {
                Ok((rest, val)) => Ok((rest, val)),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

/// Runs the delimiter before and after the main parser, and returns just
/// the result of the main parser.
pub struct DelimitedBy<PA, PD, O2> {
    pub(crate) parser: PA,
    pub(crate) delimiter: PD,
    pub(crate) _phantom: PhantomData<O2>,
}

impl<PA, PD, I, O1, O2, E> Parser<I, O1, E> for DelimitedBy<PA, PD, O2>
where
    PA: Parser<I, O1, E>,
    PD: Parser<I, O2, E>,
{
    fn parse(&mut self, input: I) -> IResult<I, O1, E> {
        let (rest, _) = self.delimiter.parse(input)?;
        let (rest, val) = self.parser.parse(rest)?;
        let (rest, _) = self.delimiter.parse(rest)?;

        Ok((rest, val))
    }
}

/// Runs the parser but doesn't change the input.
pub struct Peek<PA> {
    pub(crate) parser: PA,
}

impl<PA, I, O, E> Parser<I, O, E> for Peek<PA>
where
    PA: Parser<I, O, E>,
    I: Clone,
{
    fn parse(&mut self, input: I) -> IResult<I, O, E> {
        match self.parser.parse(input.clone()) {
            Ok((_, val)) => Ok((input, val)),
            Err(e) => Err(e),
        }
    }
}

/// Fails if the parser succeeds and vice versa.
pub struct PNot<PA, C, O> {
    pub(crate) parser: PA,
    pub(crate) code: C,
    pub(crate) _phantom: PhantomData<O>,
}

impl<PA, C, I, O, E> Parser<I, (), E> for PNot<PA, C, O>
where
    PA: Parser<I, O, E>,
    C: Code,
    E: KParseError<C, I>,
    I: Clone,
{
    fn parse(&mut self, input: I) -> IResult<I, (), E> {
        match self.parser.parse(input.clone()) {
            Ok(_) => Err(nom::Err::Error(E::from(self.code, input))),
            Err(nom::Err::Error(_)) => Ok((input, ())),
            Err(e) => Err(e),
        }
    }
}

/// Runs a verify function on the parser result.
pub struct Verify<PA, V, C, O2: ?Sized> {
    pub(crate) parser: PA,
    pub(crate) verify: V,
    pub(crate) code: C,
    pub(crate) _phantom: PhantomData<O2>,
}

impl<PA, V, C, I, O1, O2, E> Parser<I, O1, E> for Verify<PA, V, C, O2>
where
    PA: Parser<I, O1, E>,
    C: Code,
    V: Fn(&O2) -> bool,
    O1: Borrow<O2>,
    O2: ?Sized,
    E: KParseError<C, I>,
{
    fn parse(&mut self, input: I) -> IResult<I, O1, E> {
        match self.parser.parse(input) {
            Ok((rest, val)) => {
                if (self.verify)(val.borrow()) {
                    Ok((rest, val))
                } else {
                    Err(nom::Err::Error(E::from(self.code, rest)))
                }
            }
            Err(e) => Err(e),
        }
    }
}
