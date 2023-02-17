use crate::debug::{restrict, DebugWidth};
use crate::{Code, ParserError};
use nom::{InputIter, InputLength, InputTake};
use std::fmt;
use std::fmt::Debug;

/// impl of debug for ParserError.
pub(crate) fn debug_parse_error<C, I, Y>(
    f: &mut fmt::Formatter<'_>,
    err: &ParserError<C, I, Y>,
) -> fmt::Result
where
    C: Code,
    Y: Copy + Debug,
    I: Copy + Debug,
    I: InputTake + InputLength + InputIter,
{
    match f.width() {
        None | Some(0) => debug_parse_error_short(f, err),
        Some(1) => debug_parse_error_medium(f, err),
        Some(2) => debug_parse_error_long(f, err),
        _ => Ok(()),
    }
}

fn debug_parse_error_short<C, I, Y>(
    f: &mut impl fmt::Write,
    err: &ParserError<C, I, Y>,
) -> fmt::Result
where
    C: Code,
    Y: Copy + Debug,
    I: Copy + Debug,
    I: InputTake + InputLength + InputIter,
{
    write!(
        f,
        "parse error [{:?}] for {:?}",
        err.code,
        restrict(DebugWidth::Short, err.span)
    )?;

    if let Some(nom) = err.nom() {
        write!(f, "nom={:0?}, ", nom)?;
    }
    for v in err.iter_expected() {
        write!(f, "expect={:0?}, ", v)?;
    }
    for v in err.iter_suggested() {
        write!(f, "suggest={:0?}, ", v)?;
    }
    if let Some(cause) = err.cause() {
        write!(f, "cause={:0?}, ", cause)?;
    }
    if let Some(user_data) = err.user_data() {
        write!(f, "user_data={:0?}, ", user_data)?;
    }

    Ok(())
}

fn debug_parse_error_medium<C, I, Y>(
    f: &mut impl fmt::Write,
    err: &ParserError<C, I, Y>,
) -> fmt::Result
where
    C: Code,
    Y: Copy + Debug,
    I: Copy + Debug,
    I: InputTake + InputLength + InputIter,
{
    writeln!(
        f,
        "ParserError [{}] for {:?}",
        err.code,
        restrict(DebugWidth::Medium, err.span)
    )?;

    if let Some(nom) = err.nom() {
        writeln!(f, "nom")?;
        indent(f, 1)?;
        writeln!(f, "{:1?}, ", nom)?;
    }
    if err.iter_expected().next().is_some() {
        writeln!(f, "expected")?;
    }
    for v in err.iter_expected() {
        indent(f, 1)?;
        writeln!(f, "{:1?}, ", v)?;
    }
    if err.iter_suggested().next().is_some() {
        writeln!(f, "suggested")?;
    }
    for v in err.iter_suggested() {
        indent(f, 1)?;
        writeln!(f, "{:1?}, ", v)?;
    }
    if let Some(cause) = err.cause() {
        writeln!(f, "cause")?;
        indent(f, 1)?;
        writeln!(f, "{:1?}, ", cause)?;
    }
    if let Some(user_data) = err.user_data() {
        writeln!(f, "user_data")?;
        indent(f, 1)?;
        writeln!(f, "{:1?}, ", user_data)?;
    }

    Ok(())
}

fn debug_parse_error_long<C, I, Y>(
    f: &mut impl fmt::Write,
    err: &ParserError<C, I, Y>,
) -> fmt::Result
where
    C: Code,
    Y: Copy + Debug,
    I: Copy + Debug,
    I: InputTake + InputLength + InputIter,
{
    writeln!(
        f,
        "ParserError [{}] for {:?}",
        err.code,
        restrict(DebugWidth::Long, err.span)
    )?;

    if let Some(nom) = err.nom() {
        writeln!(f, "nom")?;
        indent(f, 1)?;
        writeln!(f, "{:2?}, ", nom)?;
    }
    if err.iter_expected().next().is_some() {
        writeln!(f, "expected")?;
    }
    for v in err.iter_expected() {
        indent(f, 1)?;
        writeln!(f, "{:2?}, ", v)?;
    }
    if err.iter_suggested().next().is_some() {
        writeln!(f, "suggested")?;
    }
    for v in err.iter_suggested() {
        indent(f, 1)?;
        writeln!(f, "{:2?}, ", v)?;
    }
    if let Some(cause) = err.cause() {
        writeln!(f, "cause")?;
        indent(f, 1)?;
        writeln!(f, "{:2?}, ", cause)?;
    }
    if let Some(user_data) = err.user_data() {
        writeln!(f, "user_data")?;
        indent(f, 1)?;
        writeln!(f, "{:2?}, ", user_data)?;
    }

    Ok(())
}

fn indent(f: &mut impl fmt::Write, ind: usize) -> fmt::Result {
    write!(f, "{}", " ".repeat(ind * 4))?;
    Ok(())
}
