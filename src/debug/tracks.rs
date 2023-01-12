use crate::debug::{restrict, DebugWidth};
use crate::{
    Code, DebugTrack, EnterTrack, ErrTrack, ExitTrack, InfoTrack, OkTrack, Track, WarnTrack,
};
use std::fmt;
use std::fmt::{Debug, Formatter};

pub struct Tracks<'a, 's, C: Code>(pub &'a Vec<Track<'s, C>>);

impl<'a, 's, C: Code> Debug for Tracks<'a, 's, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        debug_tracks(f, DebugWidth::Medium, &self.0)
    }
}

fn indent(f: &mut impl fmt::Write, ind: usize) -> fmt::Result {
    write!(f, "{}", " ".repeat(ind * 2))?;
    Ok(())
}

pub fn debug_tracks<C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    tracks: &Vec<Track<'_, C>>,
) -> fmt::Result {
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

fn debug_track<C: Code>(f: &mut impl fmt::Write, w: DebugWidth, v: &Track<'_, C>) -> fmt::Result {
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

fn debug_enter<C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &EnterTrack<'_, C>,
) -> fmt::Result {
    match w {
        DebugWidth::Short | DebugWidth::Medium => {
            write!(f, "{}: enter with \"{}\"", v.func, restrict(w, v.span))
        }
        DebugWidth::Long => write!(
            f,
            "{}: enter with \"{}\" <<{:?}",
            v.func,
            restrict(w, v.span),
            v.parents
        ),
    }
}

fn debug_info<C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &InfoTrack<'_, C>,
) -> fmt::Result {
    match w {
        DebugWidth::Short | DebugWidth::Medium => {
            write!(f, "{}: info {} \"{}\"", v.func, v.info, restrict(w, v.span))
        }
        DebugWidth::Long => {
            write!(
                f,
                "{}: info {} \"{}\" <<{:?}",
                v.func,
                v.info,
                restrict(w, v.span),
                v.parents
            )
        }
    }
}

fn debug_warn<C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &WarnTrack<'_, C>,
) -> fmt::Result {
    match w {
        DebugWidth::Short | DebugWidth::Medium => {
            write!(f, "{}: warn {} \"{}\"", v.func, v.warn, restrict(w, v.span))
        }
        DebugWidth::Long => {
            write!(
                f,
                "{}: warn {} \"{}\" <<{:?}",
                v.func,
                v.warn,
                restrict(w, v.span),
                v.parents
            )
        }
    }
}

fn debug_debug<C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &DebugTrack<'_, C>,
) -> fmt::Result {
    match w {
        DebugWidth::Short | DebugWidth::Medium => write!(f, "{}: debug {}", v.func, v.debug),
        DebugWidth::Long => write!(f, "{}: debug {} <<{:?}", v.func, v.debug, v.parents),
    }
}

fn debug_ok<C: Code>(f: &mut impl fmt::Write, w: DebugWidth, v: &OkTrack<'_, C>) -> fmt::Result {
    match w {
        DebugWidth::Short | DebugWidth::Medium | DebugWidth::Long => {
            if !v.span.is_empty() {
                write!(
                    f,
                    "{}: ok -> [ {}, '{}' ]",
                    v.func,
                    restrict(w, v.parsed),
                    restrict(w, v.span)
                )?;
            } else {
                write!(f, "{}: ok -> no match", v.func)?;
            }
        }
    }
    Ok(())
}

fn debug_err<C: Code>(f: &mut impl fmt::Write, w: DebugWidth, v: &ErrTrack<'_, C>) -> fmt::Result {
    match w {
        DebugWidth::Short | DebugWidth::Medium => write!(f, "{}: err {} ", v.func, v.err),
        DebugWidth::Long => write!(f, "{}: err {} <<{:?}", v.func, v.err, v.parents),
    }
}

fn debug_exit<C: Code>(
    f: &mut impl fmt::Write,
    w: DebugWidth,
    v: &ExitTrack<'_, C>,
) -> fmt::Result {
    match w {
        DebugWidth::Short | DebugWidth::Medium | DebugWidth::Long => {
            write!(f, "{}: exit", v.func)
        }
    }
}
