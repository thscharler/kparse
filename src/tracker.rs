use crate::{Code, Context, ParserError, Span, TrackParserError};

impl<'s, 't, C: Code, X: Copy, O, E> TrackParserError<'s, 't, C, X>
    for Result<(Span<'s, C>, O), nom::Err<E>>
where
    E: Into<ParserError<'s, C, X>>,
{
    type Result = Result<(Span<'s, C>, O), nom::Err<ParserError<'s, C, X>>>;

    fn track(self) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
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
                let p_err: ParserError<'s, C, X> = e.into();
                let p_err = p_err.with_code(code);
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                let p_err = p_err.with_code(code);
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }

    fn track_ok(self, parsed: Span<'s, C>) -> Self::Result {
        match self {
            Ok((span, v)) => {
                Context.exit_ok(&parsed, &span);
                Ok((span, v))
            }
            Err(nom::Err::Incomplete(e)) => Err(nom::Err::Incomplete(e)),
            Err(nom::Err::Error(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
            Err(nom::Err::Failure(e)) => {
                let p_err: ParserError<'s, C, X> = e.into();
                Context.exit_err(&p_err.span, p_err.code, &p_err);
                Err(nom::Err::Error(p_err))
            }
        }
    }
}
