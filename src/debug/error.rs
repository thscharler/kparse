use crate::debug::{restrict, DebugWidth};
use crate::{Code, ParserError};
use nom::{InputIter, InputLength, InputTake, Offset, Slice};
use std::fmt;
use std::fmt::Debug;
/// impl of debug for ParserError.
pub(crate) fn debug_parse_error<C, I, Y>(
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

fn debug_parse_error_short<C, I, Y>(
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

    if let Some(nom) = err.nom() {
        write!(f, " errorkind=")?;
        write!(f, " {:?}", nom.kind,)?;
    }

    let mut expected: Vec<_> = err.iter_expected().collect();
    expected.reverse();
    if !expected.is_empty() {
        write!(f, " expected=")?;
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

fn debug_parse_error_medium<C, I, Y>(
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
        "ParserError {} for {:?}",
        err.code,
        restrict(DebugWidth::Medium, err.span)
    )?;

    if let Some(nom) = err.nom() {
        writeln!(f, "errorkind={:?}", nom.kind,)?;
    }

    let mut expected: Vec<_> = err.iter_expected().collect();
    expected.reverse();
    if !expected.is_empty() {
        writeln!(f, "expected=")?;
        for exp in expected {
            indent(f, 1)?;
            writeln!(
                f,
                "{}:{:?}",
                exp.code,
                restrict(DebugWidth::Medium, exp.span)
            )?;
        }
    }

    Ok(())
}

fn debug_parse_error_long<C, I, Y>(
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
        "ParserError {} for {:?}",
        err.code,
        restrict(DebugWidth::Long, err.span)
    )?;

    if let Some(nom) = err.nom() {
        writeln!(f, " errorkind={:?}", nom.kind,)?;
    }

    let mut expected: Vec<_> = err.iter_expected().collect();
    expected.reverse();
    if !expected.is_empty() {
        writeln!(f, "expected=")?;
        for exp in expected {
            indent(f, 1)?;
            writeln!(f, "{}:{:?}", exp.code, restrict(DebugWidth::Long, exp.span))?;
        }
    }

    Ok(())
}

fn indent(f: &mut impl fmt::Write, ind: usize) -> fmt::Result {
    write!(f, "{}", " ".repeat(ind * 4))?;
    Ok(())
}
