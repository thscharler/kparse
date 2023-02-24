//!
//! Tracking context for a parser.
//!

use crate::debug::tracks::debug_tracks;
use crate::tracker::{DynTracker, TrackSpan, Tracker, TrackerData};
use crate::Code;
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{RangeFrom, RangeTo};

/// Context that can track the execution of a parser.
///
/// ```rust ignore
/// use nom::character::complete::digit1;
/// use kparse::examples::{ExSpan, ExTokenizerResult};
/// use kparse::tracker::StdTracker;
///
/// fn main() {
///     let txt = "1234";
///
///     #[cfg(debug_assertions)]
///     let ctx = StdTracker::new();
///     #[cfg(debug_assertions)]
///     let txt = ctx.span(txt);
///
///     // ... run parser with span.
///     nom_digits(txt);
/// }
///
/// fn nom_digits(i: ExSpan<'_>) -> ExTokenizerResult<'_, ExSpan<'_>> {
///     digit1(i)
/// }
///
/// ```
pub struct StdTracker<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    data: RefCell<TrackingData<C, T>>,
}

struct TrackingData<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    func: Vec<C>,
    track: Vec<Track<C, T>>,
}

/// New-type around a Vec<Track>, holds the tracking data of the parser.
///
/// Has a simple debug implementation to dump the tracks.
/// Hint: You can use "{:0?}", "{:1?}" and "{:2?}" to cut back the parse text.
pub struct Tracks<C, T>(pub Vec<Track<C, T>>)
where
    T: AsBytes + Clone,
    C: Code;

impl<C, T> Default for TrackingData<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    fn default() -> Self {
        Self {
            func: Default::default(),
            track: Default::default(),
        }
    }
}

impl<C, T> Default for StdTracker<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C, T> StdTracker<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    /// Creates a context for a given span.
    pub fn new() -> Self {
        Self {
            data: Default::default(),
        }
    }

    /// Create a new Span from this context using the original str.
    pub fn span<'s>(&'s self, text: T) -> TrackSpan<'s, C, T>
    where
        T: 's,
    {
        TrackSpan::new_extra(text, DynTracker(self))
    }

    /// Extract the tracking results.
    ///
    /// Removes the result from the context.
    pub fn results(&self) -> Tracks<C, T> {
        Tracks(self.data.replace(TrackingData::default()).track)
    }
}

impl<C, T> Debug for Tracks<C, T>
where
    T: AsBytes + Clone + Debug,
    C: Code,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        debug_tracks(f, f.width().into(), &self.0)
    }
}

impl<C, T> Tracker<C, T> for StdTracker<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    fn track(&self, data: TrackerData<C, T>) {
        match data {
            TrackerData::Enter(func, span) => {
                self.push_func(func);
                self.track_enter(span);
            }
            TrackerData::Exit() => {
                self.track_exit();
                self.pop_func();
            }
            TrackerData::Ok(span, parsed) => {
                self.track_ok(span, parsed);
            }
            TrackerData::Err(span, code, err_str) => {
                self.track_err(span, code, err_str);
            }
            TrackerData::Warn(span, warn) => {
                self.track_warn(span, warn);
            }
            TrackerData::Info(span, info) => {
                self.track_info(span, info);
            }
            TrackerData::Debug(span, debug) => {
                self.track_debug(span, debug);
            }
        }
    }
}

impl<C, T> StdTracker<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    // enter function
    fn push_func(&self, func: C) {
        self.data.borrow_mut().func.push(func);
    }

    // leave current function
    fn pop_func(&self) {
        self.data.borrow_mut().func.pop();
    }

    // current function
    fn func(&self) -> C {
        *self
            .data
            .borrow()
            .func
            .last()
            .expect("Vec<FnCode> is empty. forgot to trace.enter()")
    }

    fn parent_vec(&self) -> Vec<C> {
        self.data.borrow().func.clone()
    }
}

impl<C, T> StdTracker<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    fn track_enter(&self, span: LocatedSpan<T, ()>) {
        let parent = self.parent_vec();
        let func = self.func();
        self.data.borrow_mut().track.push(Track::Enter(EnterTrack {
            func,
            span,
            parents: parent,
        }));
    }

    fn track_debug(&self, span: LocatedSpan<T, ()>, debug: String) {
        let parent = self.parent_vec();
        let func = self.func();
        self.data.borrow_mut().track.push(Track::Debug(DebugTrack {
            func,
            span,
            debug,
            parents: parent,
        }));
    }

    fn track_info(&self, span: LocatedSpan<T, ()>, info: &'static str) {
        let parent = self.parent_vec();
        let func = self.func();
        self.data.borrow_mut().track.push(Track::Info(InfoTrack {
            func,
            info,
            span,
            parents: parent,
        }));
    }

    fn track_warn(&self, span: LocatedSpan<T, ()>, warn: &'static str) {
        let parent = self.parent_vec();
        let func = self.func();
        self.data.borrow_mut().track.push(Track::Warn(WarnTrack {
            func,
            warn,
            span,
            parents: parent,
        }));
    }

    fn track_ok(&self, span: LocatedSpan<T, ()>, parsed: LocatedSpan<T, ()>) {
        let parent = self.parent_vec();
        let func = self.func();
        self.data.borrow_mut().track.push(Track::Ok(OkTrack {
            func,
            span,
            parsed,
            parents: parent,
        }));
    }

    fn track_err(&self, span: LocatedSpan<T, ()>, code: C, err_str: String) {
        let parent = self.parent_vec();
        let func = self.func();
        self.data.borrow_mut().track.push(Track::Err(ErrTrack {
            func,
            code,
            span,
            err: err_str,
            parents: parent,
        }));
    }

    fn track_exit(&self) {
        let parent = self.parent_vec();
        let func = self.func();
        self.data.borrow_mut().track.push(Track::Exit(ExitTrack {
            func,
            parents: parent,
            _phantom: Default::default(),
        }));
    }
}

/// One track of the parsing trace.
#[allow(missing_docs)]
pub enum Track<C, T>
where
    T: Clone,
    C: Code,
{
    Enter(EnterTrack<C, T>),
    Debug(DebugTrack<C, T>),
    Info(InfoTrack<C, T>),
    Warn(WarnTrack<C, T>),
    Ok(OkTrack<C, T>),
    Err(ErrTrack<C, T>),
    Exit(ExitTrack<C, T>),
}

/// Track for entering a parser function.
pub struct EnterTrack<C, T>
where
    T: Clone,
    C: Code,
{
    /// Function
    pub func: C,
    /// Span
    pub span: LocatedSpan<T, ()>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for debug information.
pub struct DebugTrack<C, T>
where
    T: Clone,
    C: Code,
{
    /// Function.
    pub func: C,
    /// Span
    pub span: LocatedSpan<T, ()>,
    /// Debug info.
    pub debug: String,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for plain information.
pub struct InfoTrack<C, T>
where
    T: Clone,
    C: Code,
{
    /// Function
    pub func: C,
    /// Step info.
    pub info: &'static str,
    /// Span
    pub span: LocatedSpan<T, ()>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for plain information.
pub struct WarnTrack<C, T>
where
    T: Clone,
    C: Code,
{
    /// Function
    pub func: C,
    /// Step info.
    pub warn: &'static str,
    /// Span
    pub span: LocatedSpan<T, ()>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for ok results.
pub struct OkTrack<C, T>
where
    T: Clone,
    C: Code,
{
    /// Function.
    pub func: C,
    /// Span.
    pub span: LocatedSpan<T, ()>,
    /// Parsed span or input span.
    pub parsed: LocatedSpan<T, ()>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for err results.
pub struct ErrTrack<C, T>
where
    T: Clone,
    C: Code,
{
    /// Function.
    pub func: C,
    /// Code
    pub code: C,
    /// Span.
    pub span: LocatedSpan<T, ()>,
    /// Error message.
    pub err: String,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for exiting a parser function.
pub struct ExitTrack<C, T>
where
    T: Clone,
    C: Code,
{
    /// Function
    pub func: C,
    /// Parser call stack.
    pub parents: Vec<C>,
    /// For the lifetime ...
    pub _phantom: PhantomData<LocatedSpan<T, ()>>,
}

impl<C, T> Track<C, T>
where
    T: Clone,
    C: Code,
{
    /// Returns the func value for each branch.
    pub fn func(&self) -> C {
        match self {
            Track::Enter(v) => v.func,
            Track::Info(v) => v.func,
            Track::Warn(v) => v.func,
            Track::Debug(v) => v.func,
            Track::Ok(v) => v.func,
            Track::Err(v) => v.func,
            Track::Exit(v) => v.func,
        }
    }
}
