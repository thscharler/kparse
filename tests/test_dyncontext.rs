#![allow(dead_code)]

use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct LocatedSpan<T, X = ()> {
    /// The offset represents the position of the fragment relatively to
    /// the input of the parser. It starts at offset 0.
    offset: usize,

    /// The line number of the fragment relatively to the input of the
    /// parser. It starts at line 1.
    line: u32,

    /// The fragment that is spanned.
    /// The fragment represents a part of the input of the parser.
    fragment: T,

    /// Extra information that can be embedded by the user.
    /// Example: the parsed file name
    pub extra: X,
}

pub type Span<'s, C> = LocatedSpan<&'s str, DynContext<'s, C>>;

pub trait Code: Copy + Display + Debug + Eq {
    const NOM_ERROR: Self;
}

pub trait ParseContext<'s, C: Code> {
    /// Returns a span that encloses all of the current parser.
    fn original(&self, span: &Span<'s, C>) -> Span<'s, C>;

    /// Tracks entering a parser function.
    fn enter(&self, func: C, span: &Span<'s, C>);
}

pub struct StrContext<'s> {
    span: &'s str,
}

impl<'s, C: Code> ParseContext<'s, C> for StrContext<'s> {
    fn original(&self, span: &Span<'s, C>) -> Span<'s, C> {
        Span {
            offset: 0,
            line: 0,
            fragment: self.span,
            extra: span.extra,
        }
    }

    fn enter(&self, _: C, _: &Span<'s, C>) {}
}

pub struct TrackingContext<'s, C: Code, const TRACK: bool = false> {
    span: &'s str,
    data: RefCell<TrackingData<'s, C, TRACK>>,
}

// todo: should not be pub
pub struct TrackingData<'s, C: Code, const TRACK: bool = false> {
    func: Vec<C>,
    track: Vec<Span<'s, C>>,
}

impl<'s, C: Code, const TRACK: bool> ParseContext<'s, C> for TrackingContext<'s, C, TRACK> {
    fn original(&self, span: &Span<'s, C>) -> Span<'s, C> {
        Span {
            offset: 0,
            line: 0,
            fragment: self.span,
            extra: span.extra,
        }
    }

    fn enter(&self, func: C, span: &Span<'s, C>) {
        self.data.borrow_mut().func.push(func);
        self.data.borrow_mut().track.push(*span);
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DynContext<'s, C: Code>(Option<&'s dyn ParseContext<'s, C>>);

impl<'s, C: Code> Debug for DynContext<'s, C> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub struct Context;

impl Context {
    #[inline(never)]
    pub fn original<'s, C: Code>(&self, span: &Span<'s, C>) -> Span<'s, C> {
        match span.extra.0 {
            Some(ctx) => ctx.original(span),
            None => Span {
                offset: span.offset,
                line: span.line,
                fragment: span.fragment,
                extra: span.extra,
            },
        }
    }

    #[inline(never)]
    pub fn enter<'s, C: Code>(&self, func: C, span: &Span<'s, C>) {
        match span.extra.0 {
            Some(ctx) => ctx.enter(func, span),
            None => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZCode {
    One,
    Two,
    Three,
}

impl Display for ZCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Code for ZCode {
    const NOM_ERROR: Self = Self::One;
}

#[inline(never)]
fn test_1<'s>(ctx: &'s mut Option<StrContext<'s>>) -> Span<'s, ZCode> {
    ctx.replace(StrContext {
        span: "xxxxyyyyzzzz",
    });
    let dynctx = DynContext(Some(ctx.as_ref().unwrap()));
    let span = Span {
        offset: 0,
        line: 0,
        fragment: &ctx.as_ref().unwrap().span[4..8],
        extra: dynctx,
    };

    Context.enter(ZCode::One, &span);
    let orig = Context.original(&span);

    orig
}

#[inline(never)]
fn test_2<'s>(ctx: &'s mut Option<TrackingContext<'s, ZCode, true>>) -> Span<'s, ZCode> {
    ctx.replace(TrackingContext {
        span: "xxxxyyyyzzzz",
        data: RefCell::new(TrackingData {
            func: Vec::new(),
            track: Vec::new(),
        }),
    });
    let dynctx = DynContext(Some(ctx.as_ref().unwrap()));
    let span = Span {
        offset: 0,
        line: 0,
        fragment: &ctx.as_ref().unwrap().span[4..8],
        extra: dynctx,
    };

    Context.enter(ZCode::One, &span);
    let orig = Context.original(&span);

    orig
}

#[inline(never)]
fn test_3<'s>(ctx: &mut Option<()>) -> Span<'s, ZCode> {
    ctx.replace(());
    let dynctx = DynContext(None);
    let span = Span {
        offset: 0,
        line: 0,
        fragment: "abababababab",
        extra: dynctx,
    };

    Context.enter(ZCode::One, &span);
    let orig = Context.original(&span);

    orig
}

#[test]
pub fn main() {
    dbg!(test_3(&mut None));
}
