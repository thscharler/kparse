use crate::debug::{restrict, DebugWidth};
use crate::{Code, ParserError, SpanAndCode};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use std::fmt;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};

/// impl of debug for ParserError.
pub(crate) fn debug_parse_error<'s, T: AsBytes + Copy + Debug, C: Code, Y: Copy>(
    f: &mut fmt::Formatter<'_>,
    err: &ParserError<'s, T, C, Y>,
) -> fmt::Result
where
    T: Offset
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

fn debug_parse_error_short<'s, T: AsBytes + Copy + Debug, C: Code, Y: Copy>(
    f: &mut impl fmt::Write,
    err: &ParserError<'s, T, C, Y>,
) -> fmt::Result
where
    T: Offset
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
        restrict(DebugWidth::Short, err.span).fragment()
    )?;

    let nom = err.nom();
    if !nom.is_empty() {
        write!(f, " nom errs ")?;
        for n in &nom {
            write!(
                f,
                " {:?}:{:?}",
                n.kind,
                restrict(DebugWidth::Short, n.span).fragment()
            )?;
        }
    }

    let expected: Vec<_> = err.iter_expected().collect();
    if !expected.is_empty() {
        write!(f, " expected ")?;
        debug_expect2_short(f, &expected, 1)?;
    }

    Ok(())
}

fn debug_parse_error_medium<'s, T: AsBytes + Copy + Debug, C: Code, Y: Copy>(
    f: &mut impl fmt::Write,
    err: &ParserError<'s, T, C, Y>,
) -> fmt::Result
where
    T: Offset
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
        restrict(DebugWidth::Medium, err.span).fragment()
    )?;

    let nom = err.nom();
    if !nom.is_empty() {
        writeln!(f, "nom=")?;
        for n in &nom {
            indent(f, 1)?;
            writeln!(
                f,
                "{:?}:{:?}",
                n.kind,
                restrict(DebugWidth::Medium, n.span).fragment()
            )?;
        }
    }

    let expect = err.expected_grouped_by_offset();
    if !expect.is_empty() {
        for (g_off, subgrp) in expect {
            let first = subgrp.first().unwrap();
            writeln!(
                f,
                "expect {}:{:?} ",
                g_off,
                restrict(DebugWidth::Medium, first.span).fragment()
            )?;
            debug_expect2_medium(f, &subgrp, 1)?;
        }
    }

    Ok(())
}

fn debug_parse_error_long<'s, T: AsBytes + Copy + Debug, C: Code, Y: Copy>(
    f: &mut impl fmt::Write,
    err: &ParserError<'s, T, C, Y>,
) -> fmt::Result
where
    T: Offset
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
        restrict(DebugWidth::Long, err.span).fragment()
    )?;

    let nom = err.nom();
    if !nom.is_empty() {
        writeln!(f, "nom=")?;
        for n in &nom {
            indent(f, 1)?;
            writeln!(
                f,
                "{:?}:{:?}",
                n.kind,
                restrict(DebugWidth::Long, n.span).fragment()
            )?;
        }
    }

    let expect: Vec<_> = err.iter_expected().collect();
    if !expect.is_empty() {
        let mut sorted = expect.clone();
        sorted.sort_by(|a, b| b.span.location_offset().cmp(&a.span.location_offset()));

        writeln!(f, "expect=")?;
        debug_expect2_long(f, &sorted, 1)?;
    }

    Ok(())
}

fn indent(f: &mut impl fmt::Write, ind: usize) -> fmt::Result {
    write!(f, "{}", " ".repeat(ind * 4))?;
    Ok(())
}

// expect2

fn debug_expect2_long<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    exp_vec: &Vec<&SpanAndCode<'_, T, C>>,
    ind: usize,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    for exp in exp_vec {
        indent(f, ind)?;
        write!(
            f,
            "{}:{}:{:?}",
            exp.code,
            exp.span.location_offset(),
            restrict(DebugWidth::Long, exp.span).fragment()
        )?;
        writeln!(f)?;
    }

    Ok(())
}

fn debug_expect2_medium<T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    exp_vec: &Vec<&SpanAndCode<'_, T, C>>,
    ind: usize,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    for exp in exp_vec {
        indent(f, ind)?;
        write!(f, "{:20}", exp.code)?;

        writeln!(f)?;
    }

    Ok(())
}

fn debug_expect2_short<'s, T: AsBytes + Copy + Debug, C: Code>(
    f: &mut impl fmt::Write,
    it: &Vec<&SpanAndCode<'s, T, C>>,
    _ind: usize,
) -> fmt::Result
where
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
{
    for exp in it {
        write!(
            f,
            "{}:{:?} ",
            exp.code,
            restrict(DebugWidth::Short, exp.span).fragment()
        )?;
    }

    Ok(())
}
