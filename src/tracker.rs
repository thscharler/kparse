use crate::{Code, Context, ParserError, Span, TrackParserError};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};

impl<'s, 't, T: AsBytes + Copy + Debug + 's, C: Code, Y: Copy, O, E>
    TrackParserError<'s, 't, T, C, Y> for Result<(Span<'s, T, C>, O), nom::Err<E>>
where
    E: Into<ParserError<'s, T, C, Y>>,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    type Result = Result<(Span<'s, T, C>, O), nom::Err<ParserError<'s, T, C, Y>>>;

    fn track(self) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<'s, T, C, Y> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<'s, T, C, Y> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    fn track_as(self, code: C) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<'s, T, C, Y> = e.into();
                let p_err = p_err.with_code(code);
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<'s, T, C, Y> = e.into();
                let p_err = p_err.with_code(code);
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    fn track_ok(self, parsed: Span<'s, T, C>) -> Self::Result {
        match self {
            Ok((span, v)) => {
                Context.exit_ok(&parsed, &span);
                Ok((span, v))
            }
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<'s, T, C, Y> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<'s, T, C, Y> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }
}
