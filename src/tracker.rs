use crate::{Code, ParseContext, ParserError, ParserResult, Span, TrackParseErr};

impl<'s, 't, C: Code, X: Copy, O> TrackParseErr<'s, 't, C, X> for ParserResult<'s, C, X, O> {
    type Result = ParserResult<'s, C, X, O>;

    fn track(self) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(e) => ParseContext::exit_err(Err(e)),
        }
    }

    fn track_as(self, code: C) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let e = e.into_code(code);
                ParseContext::exit_err(Err(nom::Err::Error(e)))
            }
            Err(nom::Err::Failure(e)) => {
                let e = e.into_code(code);
                ParseContext::exit_err(Err(nom::Err::Error(e)))
            }
        }
    }

    fn track_ok(self) -> Self::Result {
        match self {
            Ok(v) => ParseContext::exit_ok(Ok(v)),
            Err(e) => ParseContext::exit_err(Err(e)),
        }
    }
}

impl<'s, 't, C: Code, X: Copy, E> TrackParseErr<'s, 't, C, X>
    for Result<(Span<'s, C>, Span<'s, C>), nom::Err<E>>
where
    E: Into<ParserError<'s, C, X>>,
{
    type Result = Result<(Span<'s, C>, Span<'s, C>), nom::Err<ParserError<'s, C, X>>>;

    fn track(self) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                ParseContext::exit_err(Err(nom::Err::Error(p_err)))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                ParseContext::exit_err(Err(nom::Err::Error(p_err)))
            }
        }
    }

    fn track_as(self, code: C) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                let p_err = p_err.into_code(code);
                ParseContext::exit_err(Err(nom::Err::Error(p_err)))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                let p_err = p_err.into_code(code);
                ParseContext::exit_err(Err(nom::Err::Error(p_err)))
            }
        }
    }

    fn track_ok(self) -> Self::Result {
        match self {
            Ok(v) => ParseContext::exit_ok(Ok(v)),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                ParseContext::exit_err(Err(nom::Err::Error(p_err)))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                ParseContext::exit_err(Err(nom::Err::Error(p_err)))
            }
        }
    }
}
