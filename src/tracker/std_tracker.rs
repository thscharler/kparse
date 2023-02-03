//!
//! Tracking context for a parser.
//!
//! ```rust ignore
//! use kparse::TrackingContext;
//!
//! let txt = "1234";
//!
//! let ctx = TrackingContext::new(true);
//! let span = ctx.span(txt);
//!
//! // ... run parser with span.
//! ```

use crate::debug::tracks::debug_tracks;
use crate::tracker::{DynTracker, TrackSpan, Tracker};
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
/// use kparse::TrackingContext;
///
/// let txt = "1234";
///
/// let ctx = TrackingContext::new(true);
/// let span = ctx.span(txt);
///
/// // ... run parser with span.
/// ```
pub struct StdTracker<C, T>
where
    T: AsBytes + Copy,
    C: Code,
{
    track: bool,
    data: RefCell<TrackingData<C, T>>,
}

struct TrackingData<C, T>
where
    T: AsBytes + Copy,
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
    T: AsBytes + Copy,
    C: Code;

impl<C, T> Default for TrackingData<C, T>
where
    T: AsBytes + Copy,
    C: Code,
{
    fn default() -> Self {
        Self {
            func: Default::default(),
            track: Default::default(),
        }
    }
}

impl<C, T> StdTracker<C, T>
where
    T: AsBytes + Copy,
    C: Code,
{
    /// Creates a context for a given span.
    pub fn new() -> Self {
        Self {
            track: true,
            data: Default::default(),
        }
    }

    /// Allows to switch off tracking.
    pub fn tracking(mut self, track: bool) -> Self {
        self.track = track;
        self
    }

    /// Allows to switch off tracking.
    pub fn set_tracking(&mut self, track: bool) {
        self.track = track;
    }

    /// Is tracking enabled.
    pub fn is_tracking(&self) -> bool {
        self.track
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
    T: AsBytes + Copy + Debug,
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
    T: AsBytes + Copy,
    C: Code,
{
    fn enter(&self, func: C, span: &LocatedSpan<T, ()>) {
        self.push_func(func);
        self.track_enter(span);
    }

    fn debug(&self, span: &LocatedSpan<T, ()>, debug: String) {
        self.track_debug(span, debug);
    }

    fn info(&self, span: &LocatedSpan<T, ()>, info: &'static str) {
        self.track_info(span, info);
    }

    fn warn(&self, span: &LocatedSpan<T, ()>, warn: &'static str) {
        self.track_warn(span, warn);
    }

    fn exit_ok(&self, span: &LocatedSpan<T, ()>, parsed: &LocatedSpan<T, ()>) {
        self.track_exit_ok(span, parsed);
        self.pop_func();
    }

    fn exit_err(&self, span: &LocatedSpan<T, ()>, code: C, err_str: String) {
        self.track_exit_err(span, code, err_str);
        self.pop_func()
    }
}

impl<C, T> StdTracker<C, T>
where
    T: AsBytes + Copy,
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
    T: AsBytes + Copy,
    C: Code,
{
    fn track_enter(&self, span: &LocatedSpan<T, ()>) {
        if self.track {
            let parent = self.parent_vec();
            let func = self.func();
            self.data.borrow_mut().track.push(Track::Enter(EnterTrack {
                func,
                span: *span,
                parents: parent,
            }));
        }
    }

    fn track_debug(&self, span: &LocatedSpan<T, ()>, debug: String) {
        if self.track {
            let parent = self.parent_vec();
            let func = self.func();
            self.data.borrow_mut().track.push(Track::Debug(DebugTrack {
                func,
                span: *span,
                debug,
                parents: parent,
            }));
        }
    }

    fn track_info(&self, span: &LocatedSpan<T, ()>, info: &'static str) {
        if self.track {
            let parent = self.parent_vec();
            let func = self.func();
            self.data.borrow_mut().track.push(Track::Info(InfoTrack {
                func,
                info,
                span: *span,
                parents: parent,
            }));
        }
    }

    fn track_warn(&self, span: &LocatedSpan<T, ()>, warn: &'static str) {
        if self.track {
            let parent = self.parent_vec();
            let func = self.func();
            self.data.borrow_mut().track.push(Track::Warn(WarnTrack {
                func,
                warn,
                span: *span,
                parents: parent,
            }));
        }
    }

    fn track_exit_ok(&self, span: &LocatedSpan<T, ()>, parsed: &LocatedSpan<T, ()>) {
        if self.track {
            let parent = self.parent_vec();
            let func = self.func();
            self.data.borrow_mut().track.push(Track::Ok(OkTrack {
                func,
                span: *span,
                parsed: *parsed,
                parents: parent.clone(),
            }));
            self.data.borrow_mut().track.push(Track::Exit(ExitTrack {
                func,
                parents: parent,
                _phantom: Default::default(),
            }));
        }
    }

    fn track_exit_err(&self, span: &LocatedSpan<T, ()>, code: C, err_str: String) {
        if self.track {
            let parent = self.parent_vec();
            let func = self.func();
            self.data.borrow_mut().track.push(Track::Err(ErrTrack {
                func,
                code,
                span: *span,
                err: err_str,
                parents: parent.clone(),
            }));
            self.data.borrow_mut().track.push(Track::Exit(ExitTrack {
                func,
                parents: parent,
                _phantom: Default::default(),
            }));
        }
    }
}

/// One track of the parsing trace.
#[allow(missing_docs)]
pub enum Track<C, T>
where
    T: Copy,
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
    T: Copy,
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
    T: Copy,
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
    T: Copy,
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
    T: Copy,
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
    T: Copy,
    C: Code,
{
    /// Function.
    pub func: C,
    /// Span.
    pub span: LocatedSpan<T, ()>,
    /// Remaining span.
    pub parsed: LocatedSpan<T, ()>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for err results.
pub struct ErrTrack<C, T>
where
    T: Copy,
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
    T: Copy,
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
    T: Copy,
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
