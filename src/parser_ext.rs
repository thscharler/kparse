use crate::{Code, KParseErrorExt};
use nom::{IResult, Parser};
use std::marker::PhantomData;

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

pub struct WithCode<PA, C> {
    pub(crate) parser: PA,
    pub(crate) code: C,
}

impl<PA, C, I, O, E> Parser<I, O, E> for WithCode<PA, C>
where
    PA: Parser<I, O, E>,
    C: Code,
    E: KParseErrorExt<C, I>,
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

pub struct Transform<PA, O1, TR, O2> {
    pub(crate) parser: PA,
    pub(crate) transform: TR,
    pub(crate) _phantom: PhantomData<(O1, O2)>,
}

impl<PA, TR, I, O1, O2, E> Parser<I, O2, E> for Transform<PA, O1, TR, O2>
where
    PA: Parser<I, O1, E>,
    TR: Fn(O1) -> Result<O2, nom::Err<E>>,
{
    fn parse(&mut self, input: I) -> IResult<I, O2, E> {
        self.parser
            .parse(input)
            .and_then(|(rest, tok)| Ok((rest, (self.transform)(tok)?)))
    }
}
