use crate::{Code, DynContext, ParseContext, Span};
use nom::AsBytes;
use std::cell::RefCell;
use std::error::Error;
use std::marker::PhantomData;

///
/// Context that can track the execution of a parser.
///
/// The parser impl must call the tracking functions via Context.
///
pub struct TrackingContext<'s, T: AsBytes + Copy, C: Code, const TRACK: bool = false> {
    span: T,
    data: RefCell<TrackingData<'s, T, C, TRACK>>,
}

struct TrackingData<'s, T: AsBytes + Copy, C: Code, const TRACK: bool = false> {
    func: Vec<C>,
    track: Vec<Track<'s, T, C>>,
}

impl<'s, T: AsBytes + Copy + 's, C: Code, const TRACK: bool> Default
    for TrackingData<'s, T, C, TRACK>
{
    fn default() -> Self {
        Self {
            func: Default::default(),
            track: Default::default(),
        }
    }
}

impl<'s, T: AsBytes + Copy + 's, C: Code, const TRACK: bool> TrackingContext<'s, T, C, TRACK> {
    /// Creates a context for a given span.
    pub fn new(span: T) -> Self {
        Self {
            span,
            data: Default::default(),
        }
    }

    /// Create a new Span from this context using the original str.
    pub fn span(&'s self) -> Span<'s, T, C> {
        Span::new_extra(self.span, DynContext(Some(self)))
    }
}

impl<'s, T: AsBytes + Copy + 's, C: Code, const TRACK: bool> ParseContext<'s, T, C>
    for TrackingContext<'s, T, C, TRACK>
{
    fn enter(&self, func: C, span: &Span<'s, T, C>) {
        self.push_func(func);
        self.track_enter(span);
    }

    fn debug(&self, span: &Span<'s, T, C>, debug: String) {
        self.track_debug(span, debug);
    }

    fn info(&self, span: &Span<'s, T, C>, info: &'static str) {
        self.track_info(span, info);
    }

    fn warn(&self, span: &Span<'s, T, C>, warn: &'static str) {
        self.track_warn(span, warn);
    }

    fn exit_ok(&self, span: &Span<'s, T, C>, parsed: &Span<'s, T, C>) {
        self.track_exit_ok(span, parsed);
        self.pop_func();
    }

    fn exit_err(&self, span: &Span<'s, T, C>, code: C, err: &dyn Error) {
        self.track_exit_err(span, code, err);
        self.pop_func()
    }
}

impl<'s, T: AsBytes + Copy + 's, C: Code, const TRACK: bool> TrackingContext<'s, T, C, TRACK> {
    /// Extract the tracking results.
    pub fn results(&self) -> Vec<Track<'s, T, C>> {
        self.data.replace(TrackingData::default()).track
    }
}

impl<'s, T: AsBytes + Copy + 's, C: Code, const TRACK: bool> TrackingContext<'s, T, C, TRACK> {
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

impl<'s, T: AsBytes + Copy + 's, C: Code, const TRACK: bool> TrackingContext<'s, T, C, TRACK> {
    fn track_enter(&self, span: &Span<'s, T, C>) {
        if TRACK {
            let parent = self.parent_vec();
            let func = self.func();
            self.data.borrow_mut().track.push(Track::Enter(EnterTrack {
                func,
                span: *span,
                parents: parent,
            }));
        }
    }

    fn track_debug(&self, span: &Span<'s, T, C>, debug: String) {
        if TRACK {
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

    fn track_info(&self, span: &Span<'s, T, C>, info: &'static str) {
        if TRACK {
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

    fn track_warn(&self, span: &Span<'s, T, C>, warn: &'static str) {
        if TRACK {
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

    fn track_exit_ok(&self, span: &Span<'s, T, C>, parsed: &Span<'s, T, C>) {
        if TRACK {
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

    fn track_exit_err(&self, span: &Span<'s, T, C>, code: C, err: &dyn Error) {
        if TRACK {
            let err_str = if let Some(cause) = err.source() {
                cause.to_string()
            } else {
                err.to_string()
            };

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
pub enum Track<'s, T, C: Code> {
    Enter(EnterTrack<'s, T, C>),
    Debug(DebugTrack<'s, T, C>),
    Info(InfoTrack<'s, T, C>),
    Warn(WarnTrack<'s, T, C>),
    Ok(OkTrack<'s, T, C>),
    Err(ErrTrack<'s, T, C>),
    Exit(ExitTrack<'s, T, C>),
}

/// Track for entering a parser function.
pub struct EnterTrack<'s, T, C: Code> {
    /// Function
    pub func: C,
    /// Span
    pub span: Span<'s, T, C>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for debug information.
pub struct DebugTrack<'s, T, C: Code> {
    /// Function.
    pub func: C,
    /// Span
    pub span: Span<'s, T, C>,
    /// Debug info.
    pub debug: String,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for plain information.
pub struct InfoTrack<'s, T, C: Code> {
    /// Function
    pub func: C,
    /// Step info.
    pub info: &'static str,
    /// Span
    pub span: Span<'s, T, C>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for plain information.
pub struct WarnTrack<'s, T, C: Code> {
    /// Function
    pub func: C,
    /// Step info.
    pub warn: &'static str,
    /// Span
    pub span: Span<'s, T, C>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for ok results.
pub struct OkTrack<'s, T, C: Code> {
    /// Function.
    pub func: C,
    /// Span.
    pub span: Span<'s, T, C>,
    /// Remaining span.
    pub parsed: Span<'s, T, C>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for err results.
pub struct ErrTrack<'s, T, C: Code> {
    /// Function.
    pub func: C,
    /// Code
    pub code: C,
    /// Span.
    pub span: Span<'s, T, C>,
    /// Error message.
    pub err: String,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for exiting a parser function.
pub struct ExitTrack<'s, T, C: Code> {
    /// Function
    pub func: C,
    /// Parser call stack.
    pub parents: Vec<C>,
    /// For the lifetime ...
    pub _phantom: PhantomData<Span<'s, T, C>>,
}

impl<'s, T, C: Code> Track<'s, T, C> {
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
