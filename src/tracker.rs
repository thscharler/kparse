use crate::{Code, ParseContext, ParserError, ParserResult, Span, TrackParseErr};

impl<'s, 't, C: Code, X: Copy, O> TrackParseErr<'s, 't, C, X> for ParserResult<'s, C, X, O> {
    type Result = ParserResult<'s, C, X, O>;

    fn track(self, ctx: &'t mut impl ParseContext<'s, C>) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(e) => ctx.err(e),
        }
    }

    fn track_as(self, ctx: &'t mut impl ParseContext<'s, C>, code: C) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(e) => ctx.err(e.into_code(code)),
        }
    }
}

impl<'s, 't, C: Code, X: Copy, E> TrackParseErr<'s, 't, C, X>
    for Result<(Span<'s>, Span<'s>), nom::Err<E>>
where
    E: Into<ParserError<'s, C, X>>,
{
    type Result = Result<(Span<'s>, Span<'s>), ParserError<'s, C, X>>;

    fn track(self, ctx: &'t mut impl ParseContext<'s, C>) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(e) => ctx.err(e.into()),
        }
    }

    fn track_as(self, ctx: &'t mut impl ParseContext<'s, C>, code: C) -> Self::Result {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let e: ParserError<'s, C, X> = e.into();
                ctx.err(e.into_code(code))
            }
        }
    }
}
