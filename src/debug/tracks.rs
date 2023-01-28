//!
//! Debug output for Track
//!

use crate::debug::{restrict, DebugWidth};
use crate::tracker::{
    DebugTrack, EnterTrack, ErrTrack, ExitTrack, InfoTrack, OkTrack, Track, WarnTrack,
};
use crate::Code;
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use std::fmt;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};

fn indent(f: &mut impl fmt::Write, ind: usize) -> fmt::Result {
    write!(f, "{}", " ".repeat(ind * 2))?;
    Ok(())
}

pub(crate) fn debug_tracks<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    tracks: &Vec<Track<C, T>>,
) -> fmt::Result
where
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
        match t {
            Track::Enter(_) => {
                ind += 1;
                indent(f, ind)?;
                debug_track(f, w, t)?;
                writeln!(f)?;
            }
            Track::Info(_) | Track::Warn(_) | Track::Debug(_) | Track::Ok(_) | Track::Err(_) => {
                indent(f, ind)?;
                debug_track(f, w, t)?;
                writeln!(f)?;
            }
            Track::Exit(_) => {
                ind -= 1;
            }
        }
    }
    Ok(())
}

fn debug_track<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &Track<C, T>,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match v {
        Track::Enter(v) => debug_enter(f, w, v),
        Track::Info(v) => debug_info(f, w, v),
        Track::Warn(v) => debug_warn(f, w, v),
        Track::Debug(v) => debug_debug(f, w, v),
        Track::Ok(v) => debug_ok(f, w, v),
        Track::Err(v) => debug_err(f, w, v),
        Track::Exit(v) => debug_exit(f, w, v),
    }
}

fn debug_enter<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &EnterTrack<C, T>,
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
                "{}: enter with {:?}",
                v.func,
                restrict(w, v.span).fragment()
            )
        }
        DebugWidth::Long => write!(
            f,
            "{}: enter with {:?} <<{:?}",
            v.func,
            restrict(w, v.span).fragment(),
            v.parents
        ),
    }
}

fn debug_info<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &InfoTrack<C, T>,
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
                "{}: info {} {:?}",
                v.func,
                v.info,
                restrict(w, v.span).fragment()
            )
        }
        DebugWidth::Long => {
            write!(
                f,
                "{}: info {} {:?} <<{:?}",
                v.func,
                v.info,
                restrict(w, v.span).fragment(),
                v.parents
            )
        }
    }
}

fn debug_warn<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &WarnTrack<C, T>,
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
                "{}: warn {} {:?}",
                v.func,
                v.warn,
                restrict(w, v.span).fragment()
            )
        }
        DebugWidth::Long => {
            write!(
                f,
                "{}: warn {} {:?} <<{:?}",
                v.func,
                v.warn,
                restrict(w, v.span).fragment(),
                v.parents
            )
        }
    }
}

fn debug_debug<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &DebugTrack<C, T>,
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
        DebugWidth::Short | DebugWidth::Medium => write!(f, "{}: debug {}", v.func, v.debug),
        DebugWidth::Long => write!(f, "{}: debug {} <<{:?}", v.func, v.debug, v.parents),
    }
}

fn debug_ok<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &OkTrack<C, T>,
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
            if !v.span.as_bytes().is_empty() {
                write!(
                    f,
                    "{}: ok -> [ {:?}, {:?} ]",
                    v.func,
                    restrict(w, v.parsed).fragment(),
                    restrict(w, v.span).fragment()
                )?;
            } else {
                write!(f, "{}: ok -> no match", v.func)?;
            }
        }
    }
    Ok(())
}

fn debug_err<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &ErrTrack<C, T>,
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
        DebugWidth::Short | DebugWidth::Medium => write!(f, "{}: err {} ", v.func, v.err),
        DebugWidth::Long => write!(f, "{}: err {} <<{:?}", v.func, v.err, v.parents),
    }
}

fn debug_exit<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &ExitTrack<C, T>,
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
