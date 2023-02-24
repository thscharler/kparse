//!
//! Everything related to tracking in a parser.
//!

use crate::{Code, ErrWrapped, KParseError};
use nom::{AsBytes, InputIter, InputLength, InputTake};
use nom_locate::LocatedSpan;
use std::fmt::{Debug, Formatter};

mod std_tracker;

pub use std_tracker::*;

/// Standard input type for tracking.
///
/// This uses the LocatedSpan.extra field to pass the tracking context
/// through the parser.
pub type TrackSpan<'s, C, T> = LocatedSpan<T, DynTracker<'s, C, T>>;

/// Data packet for the Tracker.
pub enum TrackerData<C, T>
where
    C: Code,
{
    /// Enter function
    Enter(C, LocatedSpan<T, ()>),
    /// Exit function
    Exit(),
    /// Ok result
    Ok(LocatedSpan<T, ()>, LocatedSpan<T, ()>),
    /// Err result
    Err(LocatedSpan<T, ()>, C, String),
    /// Warning
    Warn(LocatedSpan<T, ()>, &'static str),
    /// General info
    Info(LocatedSpan<T, ()>, &'static str),
    /// Debug info
    Debug(LocatedSpan<T, ()>, String),
}

/// This trait defines the tracker functions.
/// Create an [StdTracker] and use it's span() function to get the input for your
/// parser.
///
/// This trait is only used to implement the tracker, use [crate::Context] to add tracking
/// to your parser.
pub trait Tracker<C, T>
where
    C: Code,
{
    /// Collects the tracking data.
    fn track(&self, data: TrackerData<C, T>);
}

/// An instance of this struct ist kept in the extra field of LocatedSpan.
/// This way it's propagated all the way through the parser.
///
/// Access the tracking functions via [crate::Context].
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DynTracker<'c, C, T>(pub(crate) &'c dyn Tracker<C, T>)
where
    C: Code;

impl<'c, C, T> Debug for DynTracker<'c, C, T>
where
    C: Code,
{
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

/// This trait is implemented for an input type. It takes a tracking event and
/// its raw data, converts if necessary and sends it to the actual tracker.
pub trait Tracking<C>
where
    C: Code,
    Self: Sized,
{
    /// Enter a parser function.
    fn track_enter(&self, func: C);

    /// Track some debug info.
    fn track_debug(&self, debug: String);

    /// Track some other info.
    fn track_info(&self, info: &'static str);

    /// Track some warning.
    fn track_warn(&self, warn: &'static str);

    /// Calls exit_ok() on the ParseContext. You might want to use ok() instead.
    fn track_ok(&self, parsed: Self);

    /// Calls exit_err() on the ParseContext. You might want to use err() instead.
    fn track_err<E: Debug>(&self, code: C, err: &E);

    /// Calls exit() on the ParseContext. You might want to use err() or ok() instead.
    fn track_exit(&self);
}

/// This is an extension trait for nom-Results.
///
/// This is for inline tracking of parser results.
///
/// ```rust ignore
/// let (rest, h0) = nom_header(input).track_as(APCHeader)?;
/// let (rest, _) = nom_tag_plan(rest).track_as(APCPlan)?;
/// let (rest, plan) = token_name(rest).track()?;
/// let (rest, h1) = nom_header(rest).track_as(APCHeader)?;
/// ```
pub trait ResultTracking<C, I>
where
    C: Code,
    I: Clone + Debug,
    I: Tracking<C>,
    I: InputTake + InputLength + InputIter + AsBytes,
{
    /// Track an Err() result.
    fn track(self) -> Self;

    /// Track an Err() result and modify the error code in one go.
    fn track_as(self, code: C) -> Self;

    /// Track both the Ok() and the Err() branch.
    fn track_ok(self, parsed: I) -> Self;
}

/// Provides access to the tracker functions for various input types.
pub struct Context;

impl Context {
    /// Creates an Ok() Result from the parameters and tracks the result.
    #[inline(always)]
    pub fn ok<C, I, O, E>(&self, rest: I, input: I, value: O) -> Result<(I, O), nom::Err<E>>
    where
        C: Code,
        I: Clone + Debug,
        I: Tracking<C>,
        I: InputTake + InputLength + InputIter,
        E: KParseError<C, I> + Debug,
    {
        rest.track_ok(input);
        rest.track_exit();
        Ok((rest, value))
    }

    /// Tracks the error and creates a Result.
    #[inline(always)]
    pub fn err<C, I, O, E>(
        &self,
        err: E,
    ) -> Result<(I, O), nom::Err<<E as ErrWrapped>::WrappedError>>
    where
        C: Code,
        I: Clone + Debug,
        I: Tracking<C>,
        I: InputTake + InputLength + InputIter,
        E: KParseError<C, I> + ErrWrapped + Debug,
    {
        match err.parts() {
            None => Err(err.wrap()),
            Some((code, span, e)) => {
                span.track_err(code, e);
                span.track_exit();
                Err(err.wrap())
            }
        }
    }

    /// When multiple Context.enter() calls are used within one function
    /// (to denote some separation), this can be used to exit such a compartment
    /// with an ok track.
    #[inline(always)]
    pub fn ok_section<C, I>(&self, rest: I, input: I)
    where
        C: Code,
        I: Tracking<C>,
    {
        rest.track_ok(input);
    }

    /// When multiple Context.enter() calls are used within one function
    /// (to denote some separation), this can be used to exit such a compartment
    /// with an ok track.
    #[inline(always)]
    pub fn err_section<C, I, E>(&self, err: &E)
    where
        C: Code,
        I: Clone + Debug,
        I: Tracking<C>,
        I: InputTake + InputLength + InputIter,
        E: KParseError<C, I> + Debug,
    {
        match err.parts() {
            None => {}
            Some((code, span, e)) => {
                span.track_err(code, e);
            }
        }
    }

    /// Enter a parser function.
    #[inline(always)]
    pub fn enter<C, I>(&self, func: C, span: I)
    where
        C: Code,
        I: Tracking<C>,
    {
        span.track_enter(func);
    }

    /// Track some debug info.
    #[inline(always)]
    pub fn debug<C, I>(&self, span: I, debug: String)
    where
        C: Code,
        I: Tracking<C>,
    {
        span.track_debug(debug);
    }

    /// Track some other info.
    #[inline(always)]
    pub fn info<C, I>(&self, span: I, info: &'static str)
    where
        C: Code,
        I: Tracking<C>,
    {
        span.track_info(info);
    }

    /// Track some warning.
    #[inline(always)]
    pub fn warn<C, I>(&self, span: I, warn: &'static str)
    where
        C: Code,
        I: Tracking<C>,
    {
        span.track_warn(warn);
    }
}

impl<C, I, O, E> ResultTracking<C, I> for Result<(I, O), nom::Err<E>>
where
    C: Code,
    I: Clone + Debug,
    I: Tracking<C>,
    I: InputTake + InputLength + InputIter + AsBytes,
    E: Debug,
    nom::Err<E>: KParseError<C, I>,
{
    /// Keep a track if self is an error.
    #[inline(always)]
    fn track(self) -> Self {
        match self {
            Ok((rest, token)) => Ok((rest, token)),
            Err(e) => match e.parts() {
                None => Err(e),
                Some((code, span, err)) => {
                    span.track_err(code, err);
                    span.track_exit();
                    Err(e)
                }
            },
        }
    }

    /// Keep track if self is an error, and set an error code too.
    #[inline(always)]
    fn track_as(self, code: C) -> Self {
        match self {
            Ok((rest, token)) => Ok((rest, token)),
            Err(e) => {
                let e = e.with_code(code);
                match e.parts() {
                    None => Err(e),
                    Some((code, span, err)) => {
                        span.track_err(code, err);
                        span.track_exit();
                        Err(e)
                    }
                }
            }
        }
    }

    /// Keep track of self, either as error or as ok result.
    #[inline(always)]
    fn track_ok(self, parsed: I) -> Self {
        match self {
            Ok((rest, token)) => {
                rest.track_ok(parsed);
                rest.track_exit();
                Ok((rest, token))
            }
            Err(e) => match e.parts() {
                None => Err(e),
                Some((code, span, err)) => {
                    span.track_err(code, err);
                    span.track_exit();
                    Err(e)
                }
            },
        }
    }
}

type DynSpan<'s, C, T> = LocatedSpan<T, DynTracker<'s, C, T>>;

impl<'s, C, T> Tracking<C> for DynSpan<'s, C, T>
where
    C: Code,
    T: Clone + Debug + AsBytes + InputTake + InputLength,
{
    #[inline(always)]
    fn track_enter(&self, func: C) {
        self.extra
            .0
            .track(TrackerData::Enter(func, clear_span(self)));
    }

    #[inline(always)]
    fn track_debug(&self, debug: String) {
        self.extra
            .0
            .track(TrackerData::Debug(clear_span(self), debug));
    }

    #[inline(always)]
    fn track_info(&self, info: &'static str) {
        self.extra
            .0
            .track(TrackerData::Info(clear_span(self), info));
    }

    #[inline(always)]
    fn track_warn(&self, warn: &'static str) {
        self.extra
            .0
            .track(TrackerData::Warn(clear_span(self), warn));
    }

    #[inline(always)]
    fn track_ok(&self, parsed: DynSpan<'s, C, T>) {
        self.extra
            .0
            .track(TrackerData::Ok(clear_span(self), clear_span(&parsed)));
    }

    #[inline(always)]
    fn track_err<E: Debug>(&self, code: C, err: &E) {
        self.extra.0.track(TrackerData::Err(
            clear_span(self),
            code,
            format!("{:?}", err),
        ));
    }

    #[inline(always)]
    fn track_exit(&self) {
        self.extra.0.track(TrackerData::Exit());
    }
}

fn clear_span<C, T>(span: &DynSpan<'_, C, T>) -> LocatedSpan<T, ()>
where
    C: Code,
    T: AsBytes + Clone,
{
    unsafe {
        LocatedSpan::new_from_raw_offset(
            span.location_offset(),
            span.location_line(),
            span.fragment().clone(),
            (),
        )
    }
}

type PlainSpan<'s, T> = LocatedSpan<T, ()>;

impl<'s, C, T> Tracking<C> for PlainSpan<'s, T>
where
    T: Clone + Debug,
    T: InputTake + InputLength + AsBytes,
    C: Code,
{
    #[inline(always)]
    fn track_enter(&self, _func: C) {}

    #[inline(always)]
    fn track_debug(&self, _debug: String) {}

    #[inline(always)]
    fn track_info(&self, _info: &'static str) {}

    #[inline(always)]
    fn track_warn(&self, _warn: &'static str) {}

    #[inline(always)]
    fn track_ok(&self, _parsed: PlainSpan<'s, T>) {}

    #[inline(always)]
    fn track_err<E>(&self, _func: C, _err: &E) {}

    #[inline(always)]
    fn track_exit(&self) {}
}

impl<'s, C> Tracking<C> for &'s str
where
    C: Code,
{
    #[inline(always)]
    fn track_enter(&self, _func: C) {}

    #[inline(always)]
    fn track_debug(&self, _debug: String) {}

    #[inline(always)]
    fn track_info(&self, _info: &'static str) {}

    #[inline(always)]
    fn track_warn(&self, _warn: &'static str) {}

    #[inline(always)]
    fn track_ok(&self, _input: Self) {}

    #[inline(always)]
    fn track_err<E>(&self, _func: C, _err: &E) {}

    #[inline(always)]
    fn track_exit(&self) {}
}

impl<'s, C> Tracking<C> for &'s [u8]
where
    C: Code,
{
    #[inline(always)]
    fn track_enter(&self, _func: C) {}

    #[inline(always)]
    fn track_debug(&self, _debug: String) {}

    #[inline(always)]
    fn track_info(&self, _info: &'static str) {}

    #[inline(always)]
    fn track_warn(&self, _warn: &'static str) {}

    #[inline(always)]
    fn track_ok(&self, _input: Self) {}

    #[inline(always)]
    fn track_err<E>(&self, _func: C, _err: &E) {}

    #[inline(always)]
    fn track_exit(&self) {}
}
