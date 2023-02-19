//!
//! Provides [Context] to access the tracker.
//!

use crate::tracker::{DynTracker, TrackerData, Tracking};
use crate::{Code, ParseErrorExt};
use nom::{AsBytes, InputIter, InputLength, InputTake};
use nom_locate::LocatedSpan;
use std::fmt::Debug;

/// Provides access to the tracker functions for various input types.
pub struct Context;

impl Context {
    /// Creates an Ok() Result from the parameters and tracks the result.
    #[inline]
    pub fn ok<C, I, O, E>(&self, rest: I, input: I, value: O) -> Result<(I, O), nom::Err<E>>
    where
        C: Code,
        I: Copy + Debug,
        I: Tracking<C>,
        I: InputTake + InputLength + InputIter,
        E: ParseErrorExt<C, I> + Debug,
    {
        rest.track_ok(input);
        rest.track_exit();
        Ok((rest, value))
    }

    /// Tracks the error and creates a Result.
    #[inline]
    pub fn err<C, I, O, E>(&self, err: E) -> Result<(I, O), nom::Err<E::WrappedError>>
    where
        C: Code,
        I: Copy + Debug,
        I: Tracking<C>,
        I: InputTake + InputLength + InputIter,
        E: ParseErrorExt<C, I> + Debug,
    {
        match err.parts() {
            None => Err(err.into_wrapped()),
            Some((code, span, e)) => {
                span.track_err(code, e);
                span.track_exit();
                Err(err.into_wrapped())
            }
        }
    }

    /// When multiple Context.enter() calls are used within one function
    /// (to denote some separation), this can be used to exit such a compartment
    /// with an ok track.
    #[inline]
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
    #[inline]
    pub fn err_section<C, I, E>(&self, err: &E)
    where
        C: Code,
        I: Copy + Debug,
        I: Tracking<C>,
        I: InputTake + InputLength + InputIter,
        E: ParseErrorExt<C, I> + Debug,
    {
        match err.parts() {
            None => {}
            Some((code, span, e)) => {
                span.track_err(code, e);
            }
        }
    }

    /// Enter a parser function.
    #[inline]
    pub fn enter<C, I>(&self, func: C, span: I)
    where
        C: Code,
        I: Tracking<C>,
    {
        span.track_enter(func);
    }

    /// Track some debug info.
    #[inline]
    pub fn debug<C, I>(&self, span: I, debug: String)
    where
        C: Code,
        I: Tracking<C>,
    {
        span.track_debug(debug);
    }

    /// Track some other info.
    #[inline]
    pub fn info<C, I>(&self, span: I, info: &'static str)
    where
        C: Code,
        I: Tracking<C>,
    {
        span.track_info(info);
    }

    /// Track some warning.
    #[inline]
    pub fn warn<C, I>(&self, span: I, warn: &'static str)
    where
        C: Code,
        I: Tracking<C>,
    {
        span.track_warn(warn);
    }
}

type DynSpan<'s, C, T> = LocatedSpan<T, DynTracker<'s, C, T>>;

impl<'s, C, T> Tracking<C> for DynSpan<'s, C, T>
where
    C: Code,
    T: Copy + Debug + AsBytes + InputTake + InputLength,
{
    fn track_enter(&self, func: C) {
        self.extra
            .0
            .track(TrackerData::Enter(func, clear_span(self)));
    }

    fn track_debug(&self, debug: String) {
        self.extra
            .0
            .track(TrackerData::Debug(clear_span(self), debug));
    }

    fn track_info(&self, info: &'static str) {
        self.extra
            .0
            .track(TrackerData::Info(clear_span(self), info));
    }

    fn track_warn(&self, warn: &'static str) {
        self.extra
            .0
            .track(TrackerData::Warn(clear_span(self), warn));
    }

    fn track_ok(&self, parsed: DynSpan<'s, C, T>) {
        self.extra
            .0
            .track(TrackerData::Ok(clear_span(self), clear_span(&parsed)));
    }

    fn track_err<E: Debug>(&self, code: C, err: &E) {
        self.extra.0.track(TrackerData::Err(
            clear_span(self),
            code,
            format!("{:?}", err),
        ));
    }

    fn track_exit(&self) {
        self.extra.0.track(TrackerData::Exit());
    }
}

fn clear_span<C, T>(span: &DynSpan<'_, C, T>) -> LocatedSpan<T, ()>
where
    C: Code,
    T: AsBytes + Copy,
{
    unsafe {
        LocatedSpan::new_from_raw_offset(
            span.location_offset(),
            span.location_line(),
            *span.fragment(),
            (),
        )
    }
}

type PlainSpan<'s, T> = LocatedSpan<T, ()>;

impl<'s, C, T> Tracking<C> for PlainSpan<'s, T>
where
    T: Copy + Debug,
    T: InputTake + InputLength + AsBytes,
    C: Code,
{
    fn track_enter(&self, _func: C) {}

    fn track_debug(&self, _debug: String) {}

    fn track_info(&self, _info: &'static str) {}

    fn track_warn(&self, _warn: &'static str) {}

    fn track_ok(&self, _parsed: PlainSpan<'s, T>) {}

    fn track_err<E>(&self, _func: C, _err: &E) {}

    fn track_exit(&self) {}
}

impl<'s, C> Tracking<C> for &'s str
where
    C: Code,
{
    fn track_enter(&self, _func: C) {}

    fn track_debug(&self, _debug: String) {}

    fn track_info(&self, _info: &'static str) {}

    fn track_warn(&self, _warn: &'static str) {}

    fn track_ok(&self, _input: Self) {}

    fn track_err<E>(&self, _func: C, _err: &E) {}

    fn track_exit(&self) {}
}

impl<'s, C> Tracking<C> for &'s [u8]
where
    C: Code,
{
    fn track_enter(&self, _func: C) {}

    fn track_debug(&self, _debug: String) {}

    fn track_info(&self, _info: &'static str) {}

    fn track_warn(&self, _warn: &'static str) {}

    fn track_ok(&self, _input: Self) {}

    fn track_err<E>(&self, _func: C, _err: &E) {}

    fn track_exit(&self) {}
}
