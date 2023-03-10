//!
//! Debug output for Track
//!

use crate::debug::{restrict_ref, DebugWidth};
use crate::provider::{TrackData, TrackedData};
use crate::Code;
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
use std::fmt;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};

fn indent(f: &mut impl fmt::Write, ind: usize) -> fmt::Result {
    write!(f, "{}", " ".repeat(ind * 2))?;
    Ok(())
}

pub(crate) fn debug_tracks<T, C>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    tracks: &Vec<TrackedData<C, T>>,
) -> fmt::Result
where
    C: Code,
    T: AsBytes + Clone + Debug,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    let mut ind = 0;

    writeln!(f, "trace")?;

    for t in tracks {
        match t.track {
            TrackData::Enter(_, _) => {
                ind += 1;
                indent(f, ind)?;
                debug_track(f, w, t)?;
                writeln!(f)?;
            }
            TrackData::Info(_, _)
            | TrackData::Warn(_, _)
            | TrackData::Debug(_, _)
            | TrackData::Ok(_, _)
            | TrackData::Err(_, _, _) => {
                indent(f, ind)?;
                debug_track(f, w, t)?;
                writeln!(f)?;
            }
            TrackData::Exit() => {
                ind -= 1;
            }
        }
    }
    Ok(())
}

fn debug_track<T: AsBytes + Clone + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &TrackedData<C, T>,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match &v.track {
        TrackData::Enter(code, span) => debug_enter(f, w, v, *code, span.clone()),
        TrackData::Info(span, msg) => debug_info(f, w, v, span.clone(), msg),
        TrackData::Warn(span, msg) => debug_warn(f, w, v, span.clone(), msg),
        TrackData::Debug(span, msg) => debug_debug(f, w, v, span.clone(), msg.clone()),
        TrackData::Ok(rest, parsed) => debug_ok(f, w, v, rest.clone(), parsed.clone()),
        TrackData::Err(span, code, err) => debug_err(f, w, v, span.clone(), *code, err.clone()),
        TrackData::Exit() => debug_exit(f, w, v),
    }
}

fn debug_enter<T: AsBytes + Clone + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &TrackedData<C, T>,
    _code: C,
    span: LocatedSpan<T, ()>,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match w {
        DebugWidth::Short | DebugWidth::Medium => {
            write!(
                f,
                "{}: enter with {}:{:?}",
                v.func,
                span.location_offset(),
                restrict_ref(w, span.fragment())
            )
        }
        DebugWidth::Long => write!(
            f,
            "{}: enter with {}:{:?} <<{:?}",
            v.func,
            span.location_offset(),
            restrict_ref(w, span.fragment()),
            v.callstack
        ),
    }
}

fn debug_info<T: AsBytes + Clone + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &TrackedData<C, T>,
    span: LocatedSpan<T, ()>,
    msg: &str,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match w {
        DebugWidth::Short | DebugWidth::Medium => {
            write!(
                f,
                "{}: info {} {}:{:?}",
                v.func,
                msg,
                span.location_offset(),
                restrict_ref(w, span.fragment())
            )
        }
        DebugWidth::Long => {
            write!(
                f,
                "{}: info {} {}:{:?} <<{:?}",
                v.func,
                msg,
                span.location_offset(),
                restrict_ref(w, span.fragment()),
                v.callstack
            )
        }
    }
}

fn debug_warn<T: AsBytes + Clone + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &TrackedData<C, T>,
    span: LocatedSpan<T, ()>,
    msg: &str,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match w {
        DebugWidth::Short | DebugWidth::Medium => {
            write!(
                f,
                "{}: warn {} {}:{:?}",
                v.func,
                msg,
                span.location_offset(),
                restrict_ref(w, span.fragment())
            )
        }
        DebugWidth::Long => {
            write!(
                f,
                "{}: warn {} {}:{:?} <<{:?}",
                v.func,
                msg,
                span.location_offset(),
                restrict_ref(w, span.fragment()),
                v.callstack
            )
        }
    }
}

fn debug_debug<T: AsBytes + Clone + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &TrackedData<C, T>,
    _span: LocatedSpan<T, ()>,
    msg: String,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match w {
        DebugWidth::Short | DebugWidth::Medium => write!(f, "{}: debug {}", v.func, msg),
        DebugWidth::Long => write!(f, "{}: debug {} <<{:?}", v.func, msg, v.callstack),
    }
}

fn debug_ok<T: AsBytes + Clone + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &TrackedData<C, T>,
    span: LocatedSpan<T, ()>,
    parsed: LocatedSpan<T, ()>,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match w {
        DebugWidth::Short | DebugWidth::Medium | DebugWidth::Long => {
            if parsed.location_offset() + parsed.input_len() <= span.location_offset() {
                if parsed.input_len() > 0 {
                    write!(
                        f,
                        "{}: ok -> [ {}:{:?}, {}:{:?} ]",
                        v.func,
                        parsed.location_offset(),
                        parsed.fragment(),
                        span.location_offset(),
                        restrict_ref(w, span.fragment())
                    )?;
                } else {
                    write!(f, "{}: ok -> no match", v.func)?;
                }
            } else {
                let parsed_len = span.location_offset() - parsed.location_offset();
                let parsed = parsed.take(parsed_len);

                write!(
                    f,
                    "{}: ok -> [ {}:{:?}, {}:{:?} ]",
                    v.func,
                    parsed.location_offset(),
                    parsed.fragment(),
                    span.location_offset(),
                    restrict_ref(w, span.fragment())
                )?;
            }
        }
    }
    Ok(())
}

fn debug_err<T: AsBytes + Clone + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &TrackedData<C, T>,
    _span: LocatedSpan<T, ()>,
    _code: C,
    err: String,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match w {
        DebugWidth::Short | DebugWidth::Medium => write!(f, "{}: err {} ", v.func, err),
        DebugWidth::Long => write!(f, "{}: err {} <<{:?}", v.func, err, v.callstack),
    }
}

fn debug_exit<T: AsBytes + Clone + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &TrackedData<C, T>,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match w {
        DebugWidth::Short | DebugWidth::Medium | DebugWidth::Long => {
            write!(f, "{}: exit", v.func)
        }
    }
}
