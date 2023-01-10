use crate::debug::{restrict, DebugWidth};
use crate::{Code, Expect, ParserError};
use std::fmt;

pub fn debug_parse_of_error_short<'s, C: Code, X: Copy>(
    f: &mut impl fmt::Write,
    err: &ParserError<'s, C, X>,
) -> fmt::Result {
    write!(
        f,
        "ParserError [{}] for \"{}\"",
        err.code,
        restrict(DebugWidth::Short, err.span)
    )?;

    let nom = err.nom();
    if !nom.is_empty() {
        write!(f, " nom errs ")?;
        for n in &nom {
            write!(
                f,
                " {:?}:\"{}\"",
                n.kind,
                restrict(DebugWidth::Short, n.span)
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

pub fn debug_parse_of_error_medium<'s, C: Code, X: Copy>(
    f: &mut impl fmt::Write,
    err: &ParserError<'s, C, X>,
) -> fmt::Result {
    writeln!(
        f,
        "ParserError {} \"{}\"",
        err.code,
        restrict(DebugWidth::Medium, err.span)
    )?;

    let nom = err.nom();
    if !nom.is_empty() {
        writeln!(f, "nom=")?;
        for n in &nom {
            indent(f, 1)?;
            writeln!(
                f,
                "{:?}:\"{}\"",
                n.kind,
                restrict(DebugWidth::Medium, n.span)
            )?;
        }
    }

    let expect = err.expect_grouped_by_offset();
    if !expect.is_empty() {
        for (g_off, subgrp) in expect {
            let first = subgrp.first().unwrap();
            writeln!(
                f,
                "expect {}:\"{}\" ",
                g_off,
                restrict(DebugWidth::Medium, first.span)
            )?;
            debug_expect2_medium(f, &subgrp, 1)?;
        }
    }

    Ok(())
}

pub fn debug_parse_of_error_long<'s, C: Code, X: Copy>(
    f: &mut impl fmt::Write,
    err: &ParserError<'s, C, X>,
) -> fmt::Result {
    writeln!(
        f,
        "ParserError {} \"{}\"",
        err.code,
        restrict(DebugWidth::Long, err.span)
    )?;

    let nom = err.nom();
    if !nom.is_empty() {
        writeln!(f, "nom=")?;
        for n in &nom {
            indent(f, 1)?;
            writeln!(f, "{:?}:\"{}\"", n.kind, restrict(DebugWidth::Long, n.span))?;
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

fn debug_expect2_long<C: Code>(
    f: &mut impl fmt::Write,
    exp_vec: &Vec<&Expect<'_, C>>,
    ind: usize,
) -> fmt::Result {
    for exp in exp_vec {
        indent(f, ind)?;
        write!(
            f,
            "{}:{}:\"{}\"",
            exp.code,
            exp.span.location_offset(),
            restrict(DebugWidth::Long, exp.span)
        )?;
        writeln!(f)?;
    }

    Ok(())
}

fn debug_expect2_medium<C: Code>(
    f: &mut impl fmt::Write,
    exp_vec: &Vec<&Expect<'_, C>>,
    ind: usize,
) -> fmt::Result {
    for exp in exp_vec {
        indent(f, ind)?;
        write!(f, "{:20}", exp.code)?;

        writeln!(f)?;
    }

    Ok(())
}

fn debug_expect2_short<'s, C: Code>(
    f: &mut impl fmt::Write,
    it: &Vec<&Expect<'s, C>>,
    _ind: usize,
) -> fmt::Result {
    for exp in it {
        write!(
            f,
            "{}:\"{}\" ",
            exp.code,
            restrict(DebugWidth::Short, exp.span)
        )?;
    }

    Ok(())
}
