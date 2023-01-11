use crate::{str_union, Code, HoldContext, ParseContext, Span};
use nom_locate::LocatedSpan;
use std::cell::RefCell;
use std::error::Error;
use std::marker::PhantomData;

pub struct TrackingContext<'s, C: Code, const TRACK: bool = false> {
    span: &'s str,
    data: RefCell<TrackingData<'s, C, TRACK>>,
}

struct TrackingData<'s, C: Code, const TRACK: bool = false> {
    func: Vec<C>,
    track: Vec<Track<'s, C>>,
}

impl<'s, C: Code, const TRACK: bool> Default for TrackingData<'s, C, TRACK> {
    fn default() -> Self {
        Self {
            func: Default::default(),
            track: Default::default(),
        }
    }
}

impl<'s, C: Code, const TRACK: bool> TrackingContext<'s, C, TRACK> {
    /// Creates a context for a given span.
    pub fn new(span: &'s str) -> Self {
        Self {
            span,
            data: Default::default(),
        }
    }

    /// Create a new Span from this context.
    pub fn new_span(&'s self) -> Span<'s, C> {
        Span::new_extra(self.span, HoldContext { 0: self })
    }
}

impl<'s, C: Code, const TRACK: bool> ParseContext<'s, C> for TrackingContext<'s, C, TRACK> {
    // we don't really need _span for this, but it's useful in Context.
    fn original(&'s self, _span: &Span<'s, C>) -> Span<'s, C> {
        self.new_span()
    }

    unsafe fn span_union(&self, first: &Span<'s, C>, second: &Span<'s, C>) -> Span<'s, C> {
        let u_str = str_union(&*self.span, &*first, &*second);

        // starting point is the first span, so we use it's extra.
        // and it naturally gives all the other values too.
        LocatedSpan::new_from_raw_offset(
            first.location_offset(),
            first.location_line(),
            u_str,
            first.extra.clone(),
        )
    }

    fn enter(&self, func: C, span: &Span<'s, C>) {
        self.push_func(func);
        self.track_enter(span);
    }

    fn debug(&self, span: &Span<'s, C>, debug: String) {
        self.track_debug(span, debug);
    }

    fn info(&self, span: &Span<'s, C>, info: &'static str) {
        self.track_info(span, info);
    }

    fn warn(&self, span: &Span<'s, C>, warn: &'static str) {
        self.track_warn(span, warn);
    }

    fn exit_ok(&self, span: &Span<'s, C>, parsed: &Span<'s, C>) {
        self.track_exit_ok(span, parsed);
        self.pop_func();
    }

    fn exit_err(&self, span: &Span<'s, C>, code: C, err: &dyn Error) {
        self.track_exit_err(span, code, err);
        self.pop_func()
    }
}

impl<'s, C: Code, const TRACK: bool> TrackingContext<'s, C, TRACK> {
    /// Dissolve to the tracking results.
    pub fn into_result(self) -> Vec<Track<'s, C>> {
        self.data.replace(TrackingData::default()).track
    }
}

impl<'s, C: Code, const TRACK: bool> TrackingContext<'s, C, TRACK> {
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

impl<'s, C: Code, const TRACK: bool> TrackingContext<'s, C, TRACK> {
    fn track_enter(&self, span: &Span<'s, C>) {
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

    fn track_debug(&self, span: &Span<'s, C>, debug: String) {
        if TRACK {
            let parent = self.parent_vec();
            let func = self.func();
            self.data.borrow_mut().track.push(Track::Debug(DebugTrack {
                func,
                span: *span,
                debug,
                parents: parent,
                _phantom: Default::default(),
            }));
        }
    }

    fn track_info(&self, span: &Span<'s, C>, info: &'static str) {
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

    fn track_warn(&self, span: &Span<'s, C>, warn: &'static str) {
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

    fn track_exit_ok(&self, span: &Span<'s, C>, parsed: &Span<'s, C>) {
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

    fn track_exit_err(&self, span: &Span<'s, C>, code: C, err: &dyn Error) {
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
pub enum Track<'s, C: Code> {
    Enter(EnterTrack<'s, C>),
    Debug(DebugTrack<'s, C>),
    Info(InfoTrack<'s, C>),
    Warn(WarnTrack<'s, C>),
    Ok(OkTrack<'s, C>),
    Err(ErrTrack<'s, C>),
    Exit(ExitTrack<'s, C>),
}

/// Track for entering a parser function.
pub struct EnterTrack<'s, C: Code> {
    /// Function
    pub func: C,
    /// Span
    pub span: Span<'s, C>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for debug information.
pub struct DebugTrack<'s, C: Code> {
    /// Function.
    pub func: C,
    /// Span
    pub span: Span<'s, C>,
    /// Debug info.
    pub debug: String,
    /// Parser call stack.
    pub parents: Vec<C>,
    /// For the lifetime ...
    pub _phantom: PhantomData<Span<'s, C>>,
}

/// Track for plain information.
pub struct InfoTrack<'s, C: Code> {
    /// Function
    pub func: C,
    /// Step info.
    pub info: &'static str,
    /// Span
    pub span: Span<'s, C>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for plain information.
pub struct WarnTrack<'s, C: Code> {
    /// Function
    pub func: C,
    /// Step info.
    pub warn: &'static str,
    /// Span
    pub span: Span<'s, C>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for ok results.
pub struct OkTrack<'s, C: Code> {
    /// Function.
    pub func: C,
    /// Span.
    pub span: Span<'s, C>,
    /// Remaining span.
    pub parsed: Span<'s, C>,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for err results.
pub struct ErrTrack<'s, C: Code> {
    /// Function.
    pub func: C,
    /// Code
    pub code: C,
    /// Span.
    pub span: Span<'s, C>,
    /// Error message.
    pub err: String,
    /// Parser call stack.
    pub parents: Vec<C>,
}

/// Track for exiting a parser function.
pub struct ExitTrack<'s, C: Code> {
    /// Function
    pub func: C,
    /// Parser call stack.
    pub parents: Vec<C>,
    /// For the lifetime ...
    pub _phantom: PhantomData<Span<'s, C>>,
}

impl<'s, C: Code> Track<'s, C> {
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
