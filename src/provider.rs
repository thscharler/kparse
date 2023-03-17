use crate::debug::tracks::debug_tracks;
use crate::{Code, DynTrackProvider};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::{RangeFrom, RangeTo};

/// Data packet for the Tracker.
#[derive(Debug)]
pub enum TrackData<C, T>
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

/// Provides the tracking functionality backend.
pub trait TrackProvider<C, T>
where
    C: Code,
{
    /// Create a span with this TrackingProvider attached.
    fn track_span<'s>(&'s self, text: T) -> LocatedSpan<T, DynTrackProvider<'s, C, T>>
    where
        T: 's;

    /// Extract the tracking results.
    /// Removes the result from the context.
    fn results(&self) -> TrackedDataVec<C, T>;

    /// Collects the tracking data. Use Track.xxx()
    fn track(&self, data: TrackData<C, T>);
}

impl<'c, C, T> Debug for DynTrackProvider<'c, C, T>
where
    C: Code,
{
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
pub struct TrackedData<C, I>
where
    C: Code,
{
    pub func: C,
    pub callstack: Vec<C>,
    pub track: TrackData<C, I>,
}

pub struct TrackedDataVec<C, I>(Vec<TrackedData<C, I>>)
where
    C: Code;

impl<C, I> Debug for TrackedDataVec<C, I>
where
    C: Code,
    I: AsBytes + Clone + Debug,
    I: Offset
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

#[derive(Debug)]
pub struct StdTracker<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    data: RefCell<StdTracks<C, T>>,
}

#[derive(Debug)]
struct StdTracks<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    func: Vec<C>,
    track: Vec<TrackedData<C, T>>,
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

    fn callstack(&self) -> Vec<C> {
        self.data.borrow().func.clone()
    }

    fn append_track(&self, track: TrackData<C, T>) {
        let callstack = self.callstack();
        let func = self.func();
        self.data.borrow_mut().track.push(TrackedData {
            func,
            callstack,
            track,
        });
    }
}

impl<C, T> TrackProvider<C, T> for StdTracker<C, T>
where
    T: AsBytes + Clone,
    C: Code,
{
    /// Create a new Span from this context using the original str.
    fn track_span<'s>(&'s self, text: T) -> LocatedSpan<T, DynTrackProvider<'s, C, T>>
    where
        T: 's,
    {
        LocatedSpan::new_extra(text, self)
    }

    /// Extract the tracking results.
    ///
    /// Removes the result from the context.
    fn results(&self) -> TrackedDataVec<C, T> {
        TrackedDataVec(self.data.replace(StdTracks::default()).track)
    }

    fn track(&self, data: TrackData<C, T>) {
        match &data {
            TrackData::Enter(func, _) => {
                self.push_func(*func);
                self.append_track(data);
            }
            TrackData::Exit() => {
                self.append_track(data);
                self.pop_func();
            }
            TrackData::Ok(_, _)
            | TrackData::Err(_, _, _)
            | TrackData::Warn(_, _)
            | TrackData::Info(_, _)
            | TrackData::Debug(_, _) => {
                self.append_track(data);
            }
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

impl<C, T> Default for StdTracks<C, T>
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
