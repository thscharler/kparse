use crate::debug::{restrict, DebugWidth};
use crate::{Code, ParserError};
use nom::{InputIter, InputLength, InputTake, Offset, Slice};
use std::fmt;
use std::fmt::Debug;
/// impl of debug for ParserError.
pub(crate) fn debug_parse_error<I, C, Y>(
    f: &mut fmt::Formatter<'_>,
    err: &ParserError<C, I, Y>,
) -> fmt::Result
where
    C: Code,
    Y: Copy,
    I: Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    match f.width() {
        None | Some(0) => debug_parse_error_short(f, err),
        Some(1) => debug_parse_error_medium(f, err),
        Some(2) => debug_parse_error_long(f, err),
        _ => Ok(()),
    }
}

use std::ops::{RangeFrom, RangeTo};

fn debug_parse_error_short<I, C, Y>(
    f: &mut impl fmt::Write,
    err: &ParserError<C, I, Y>,
) -> fmt::Result
where
    C: Code,
    Y: Copy,
    I: Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    write!(
        f,
        "ParserError {} for {:?}",
        err.code,
        restrict(DebugWidth::Short, err.span)
    )?;

    let nom = err.nom();
    if !nom.is_empty() {
        write!(f, " nom errs ")?;
        for n in &nom {
            write!(f, " {:?}:{:?}", n.kind, restrict(DebugWidth::Short, n.span))?;
        }
    }

    let mut expected: Vec<_> = err.iter_expected().collect();
    expected.reverse();
    if !expected.is_empty() {
        write!(f, " expected ")?;
        for exp in expected {
            write!(
                f,
                "{}:{:?} ",
                exp.code,
                restrict(DebugWidth::Short, exp.span)
            )?;
        }
    }

    Ok(())
}

fn debug_parse_error_medium<I, C, Y>(
    f: &mut impl fmt::Write,
    err: &ParserError<C, I, Y>,
) -> fmt::Result
where
    C: Code,
    Y: Copy,
    I: Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    writeln!(
        f,
        "ParserError {} {:?}",
        err.code,
        restrict(DebugWidth::Medium, err.span)
    )?;

    let nom = err.nom();
    if !nom.is_empty() {
        writeln!(f, "nom=")?;
        for n in &nom {
            indent(f, 1)?;
            writeln!(f, "{:?}:{:?}", n.kind, restrict(DebugWidth::Medium, n.span))?;
        }
    }

    let mut expected: Vec<_> = err.iter_expected().collect();
    expected.reverse();
    if !expected.is_empty() {
        writeln!(f, "expect=")?;
        for exp in expected {
            indent(f, 1)?;
            write!(
                f,
                "{}:{:?}",
                exp.code,
                restrict(DebugWidth::Medium, exp.span)
            )?;
            writeln!(f)?;
        }
    }

    Ok(())
}

fn debug_parse_error_long<I, C, Y>(
    f: &mut impl fmt::Write,
    err: &ParserError<C, I, Y>,
) -> fmt::Result
where
    C: Code,
    Y: Copy,
    I: Copy + Debug,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    writeln!(
        f,
        "ParserError {} {:?}",
        err.code,
        restrict(DebugWidth::Long, err.span)
    )?;

    let nom = err.nom();
    if !nom.is_empty() {
        writeln!(f, "nom=")?;
        for n in &nom {
            indent(f, 1)?;
            writeln!(f, "{:?}:{:?}", n.kind, restrict(DebugWidth::Long, n.span))?;
        }
    }

    let mut expected: Vec<_> = err.iter_expected().collect();
    expected.reverse();
    if !expected.is_empty() {
        writeln!(f, "expect=")?;
        for exp in expected {
            indent(f, 1)?;
            write!(f, "{}:{:?}", exp.code, restrict(DebugWidth::Long, exp.span))?;
            writeln!(f)?;
        }
    }

    Ok(())
}

fn indent(f: &mut impl fmt::Write, ind: usize) -> fmt::Result {
    write!(f, "{}", " ".repeat(ind * 4))?;
    Ok(())
}
