//!
//! Test framework for parsers.
//!
//! ```rust ignore
//! use kparse::test::{CheckDump, track_parse};
//!
//! // run the parser and expect Ok(). Otherwise dump & panic.
//! track_parse(&mut None, "sample", parser_fn).okok().q(CheckDump);
//!
//! ```
//! Runs the parser and works like a builder to evaluate the results.
//! The final function is q() which runs the given report.
//!
//! Note: The &mut None is because lifetimes.

use crate::debug::{restrict, DebugWidth};
use crate::spans::SpanFragment;
use crate::tracker::{StdTracker, TrackSpan};
use crate::{Code, ParserError};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use nom_locate::LocatedSpan;
pub use report::*;
pub use span::*;
use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{RangeFrom, RangeTo};
use std::time::{Duration, Instant};
use std::vec::Vec;

/// Value comparison.
pub type CompareFn<O, V> = for<'a> fn(parsed: &'a O, test: V) -> bool;

/// Collected data of the test run.
///
/// Call any of the test functions and finish with q().
pub struct Test<'s, P, I, O, E> {
    /// ParseContext
    pub context: &'s P,
    /// text
    pub span: I,
    /// Test Result
    pub result: Result<(I, O), nom::Err<E>>,
    /// Test duration
    pub duration: Duration,
    /// Any check failed
    pub failed: Cell<bool>,
}

/// Result reporting.
pub trait Report<T> {
    /// Report something.
    fn report(&self, test: &T);
}

/// Runs the parser and records the results.
/// Use ok(), err(), ... to check specifics.
///
/// Finish the test with q().
#[must_use]
pub fn track_parse<'s, C, T, O, E>(
    buf: &'s mut Option<StdTracker<C, T>>,
    text: T,
    fn_test: impl Fn(TrackSpan<'s, C, T>) -> Result<(TrackSpan<'s, C, T>, O), nom::Err<E>>,
) -> Test<'s, StdTracker<C, T>, TrackSpan<'s, C, T>, O, E>
where
    T: AsBytes + Copy,
    C: Code,
{
    track_parse_ext(buf, true, text, fn_test)
}

/// Runs the parser and records the results.
/// Use ok(), err(), ... to check specifics.
///
/// Finish the test with q().
#[must_use]
pub fn track_parse_ext<'s, C, T, O, E>(
    buf: &'s mut Option<StdTracker<C, T>>,
    track: bool,
    text: T,
    fn_test: impl Fn(TrackSpan<'s, C, T>) -> Result<(TrackSpan<'s, C, T>, O), nom::Err<E>>,
) -> Test<'s, StdTracker<C, T>, TrackSpan<'s, C, T>, O, E>
where
    T: AsBytes + Copy,
    C: Code,
{
    buf.replace(StdTracker::new().tracking(track));
    let context = buf.as_ref().expect("yes");

    let span = context.span(text);

    let now = Instant::now();
    let result = fn_test(span);
    let duration = now.elapsed();

    Test {
        span,
        context,
        result,
        duration,
        failed: Cell::new(false),
    }
}

/// Not an error code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoCode;

impl Display for NoCode {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Code for NoCode {
    const NOM_ERROR: Self = NoCode;
}

// -----------------------------------------------------------------------

/// Runs a parser for LocatedSpan and records the results.
/// Use ok(), err(), ... to check specifics.
/// Finish the test with q().
///
/// This method changes behaviour between debug and release build.
/// In debug build the StdTracker is active and expects a TrackSpan for the parser function.
/// In release mode no tracking is active and it expects a LocatedSpan for the parser function.
#[must_use]
#[cfg(debug_assertions)]
pub fn span_parse<'s, C, T, O, E>(
    buf: &'s mut Option<StdTracker<C, T>>,
    text: T,
    fn_test: impl Fn(TrackSpan<'s, C, T>) -> Result<(TrackSpan<'s, C, T>, O), nom::Err<E>>,
) -> Test<'s, StdTracker<C, T>, TrackSpan<'s, C, T>, O, E>
where
    T: AsBytes + Copy + 's,
    C: Code,
{
    buf.replace(StdTracker::new());
    let context = buf.as_ref().expect("yes");

    let span = context.span(text);

    let now = Instant::now();
    let result = fn_test(span);
    let duration = now.elapsed();

    Test {
        span,
        context,
        result,
        duration,
        failed: Cell::new(false),
    }
}

/// Runs a parser for LocatedSpan and records the results.
/// Use ok(), err(), ... to check specifics.
/// Finish the test with q().
///
/// This method changes behaviour between debug and release build.
/// In debug build the StdTracker is active and expects a TrackSpan for the parser function.
/// In release mode no tracking is active and it expects a LocatedSpan for the parser function.
#[must_use]
#[cfg(not(debug_assertions))]
pub fn span_parse<'s, T, O, E>(
    _buf: &'s mut Option<StdTracker<NoCode, &'s str>>,
    text: T,
    fn_test: impl Fn(LocatedSpan<T, ()>) -> Result<(LocatedSpan<T, ()>, O), nom::Err<E>>,
) -> Test<'s, (), LocatedSpan<T, ()>, O, E>
where
    T: AsBytes + Copy + 's,
{
    let span = LocatedSpan::new(text);

    let now = Instant::now();
    let result = fn_test(span);
    let duration = now.elapsed();

    Test {
        span,
        context: &(),
        result,
        duration,
        failed: Cell::new(false),
    }
}

/// Runs a parser for LocatedSpan and records the results.
/// Use ok(), err(), ... to check specifics.
/// Finish the test with q().
///
/// This method does the same in debug and release builds.
#[must_use]
pub fn span_parsex<'s, T, O, E>(
    _buf: &'s mut Option<StdTracker<NoCode, &'s str>>,
    text: T,
    fn_test: impl Fn(LocatedSpan<T, ()>) -> Result<(LocatedSpan<T, ()>, O), nom::Err<E>>,
) -> Test<'s, (), LocatedSpan<T, ()>, O, E>
where
    T: AsBytes + Copy + 's,
{
    let span = LocatedSpan::new(text);

    let now = Instant::now();
    let result = fn_test(span);
    let duration = now.elapsed();

    Test {
        span,
        context: &(),
        result,
        duration,
        failed: Cell::new(false),
    }
}

/// Runs a parser for &str and records the results.
/// Use ok(), err(), ... to check specifics.
/// Finish the test with q().
///
/// This method changes behaviour between debug and release build.
/// In debug build the StdTracker is active and expects a TrackSpan for the parser function.
/// In release mode no tracking is active and it expects a &str for the parser function.
#[must_use]
#[cfg(debug_assertions)]
pub fn str_parse<'s, C, O, E>(
    buf: &'s mut Option<StdTracker<C, &'s str>>,
    text: &'s str,
    fn_test: impl Fn(TrackSpan<'s, C, &'s str>) -> Result<(TrackSpan<'s, C, &'s str>, O), nom::Err<E>>,
) -> Test<'s, StdTracker<C, &'s str>, TrackSpan<'s, C, &'s str>, O, E>
where
    C: Code,
{
    buf.replace(StdTracker::new());
    let context = buf.as_ref().expect("yes");

    let span = context.span(text);

    let now = Instant::now();
    let result = fn_test(span);
    let duration = now.elapsed();

    Test {
        span,
        context,
        result,
        duration,
        failed: Cell::new(false),
    }
}

/// Runs a parser for &str and records the results.
/// Use ok(), err(), ... to check specifics.
/// Finish the test with q().
///
/// This method changes behaviour between debug and release build.
/// In debug build the StdTracker is active and expects a TrackSpan for the parser function.
/// In release mode no tracking is active and it expects a &str for the parser function.
#[must_use]
#[cfg(not(debug_assertions))]
pub fn str_parse<'s, O, E>(
    _buf: &'s mut Option<StdTracker<NoCode, &'s str>>,
    text: &'s str,
    fn_test: impl Fn(&'s str) -> Result<(&'s str, O), nom::Err<E>>,
) -> Test<'s, (), &'s str, O, E> {
    let now = Instant::now();
    let result = fn_test(text);
    let duration = now.elapsed();

    Test {
        span: text,
        context: &(),
        result,
        duration,
        failed: Cell::new(false),
    }
}

/// Runs a parser for &str and records the results.
/// Use ok(), err(), ... to check specifics.
/// Finish the test with q().
///
/// This method does the same in debug and release builds.
#[must_use]
pub fn str_parsex<'s, O, E>(
    _buf: &'s mut Option<StdTracker<NoCode, &'s str>>,
    text: &'s str,
    fn_test: impl Fn(&'s str) -> Result<(&'s str, O), nom::Err<E>>,
) -> Test<'s, (), &'s str, O, E> {
    let now = Instant::now();
    let result = fn_test(text);
    let duration = now.elapsed();

    Test {
        span: text,
        context: &(),
        result,
        duration,
        failed: Cell::new(false),
    }
}

/// Runs a parser for &[u8] and records the results.
/// Use ok(), err(), ... to check specifics.
/// Finish the test with q().
///
/// This method changes behaviour between debug and release build.
/// In debug build the StdTracker is active and expects a TrackSpan for the parser function.
/// In release mode no tracking is active and it expects a &[u8] for the parser function.
#[must_use]
#[cfg(debug_assertions)]
pub fn byte_parse<'s, C, O, E>(
    buf: &'s mut Option<StdTracker<C, &'s [u8]>>,
    text: &'s [u8],
    fn_test: impl Fn(TrackSpan<'s, C, &'s [u8]>) -> Result<(TrackSpan<'s, C, &'s [u8]>, O), nom::Err<E>>,
) -> Test<'s, StdTracker<C, &'s [u8]>, TrackSpan<'s, C, &'s [u8]>, O, E>
where
    C: Code,
{
    buf.replace(StdTracker::new());
    let context = buf.as_ref().expect("yes");

    let span = context.span(text);

    let now = Instant::now();
    let result = fn_test(span);
    let duration = now.elapsed();

    Test {
        span,
        context,
        result,
        duration,
        failed: Cell::new(false),
    }
}

/// Runs a parser for &[u8] and records the results.
/// Use ok(), err(), ... to check specifics.
/// Finish the test with q().
///
/// This method changes behaviour between debug and release build.
/// In debug build the StdTracker is active and expects a TrackSpan for the parser function.
/// In release mode no tracking is active and it expects a &[u8] for the parser function.
#[must_use]
#[cfg(not(debug_assertions))]
pub fn byte_parse<'s, O, E>(
    _buf: &'s mut Option<StdTracker<NoCode, &'s [u8]>>,
    text: &'s [u8],
    fn_test: impl Fn(&'s [u8]) -> Result<(&'s [u8], O), nom::Err<E>>,
) -> Test<'s, (), &'s [u8], O, E> {
    let now = Instant::now();
    let result = fn_test(text);
    let duration = now.elapsed();

    Test {
        span: text,
        context: &(),
        result,
        duration,
        failed: Cell::new(false),
    }
}

/// Runs a parser for &[u8] and records the results.
/// Use ok(), err(), ... to check specifics.
/// Finish the test with q().
///
/// This method does the same in debug and release builds.
#[must_use]
pub fn byte_parsex<'s, O, E>(
    _buf: &'s mut Option<StdTracker<NoCode, &'s [u8]>>,
    text: &'s [u8],
    fn_test: impl Fn(&'s [u8]) -> Result<(&'s [u8], O), nom::Err<E>>,
) -> Test<'s, (), &'s [u8], O, E> {
    let now = Instant::now();
    let result = fn_test(text);
    let duration = now.elapsed();

    Test {
        span: text,
        context: &(),
        result,
        duration,
        failed: Cell::new(false),
    }
}

// -----------------------------------------------------------------------

impl<'s, P, I, O, E> Test<'s, P, I, O, E>
where
    I: AsBytes + Copy + Debug + 's,
    O: Debug,
    E: Debug,
{
    /// Sets the failed flag.
    pub fn flag_fail(&self) {
        self.failed.set(true);
    }

    /// Always fails.
    ///
    /// Finish the test with q().
    pub fn fail(&self) -> &Self {
        println!("FAIL: Unconditionally");
        self.flag_fail();
        self
    }

    /// Checks for ok.
    /// Any result that is not Err is ok.
    #[must_use]
    pub fn ok_any(&self) -> &Self {
        match &self.result {
            Ok(_) => {}
            Err(_) => {
                println!("FAIL: Expected ok, but was an error.");
                self.flag_fail();
            }
        }
        self
    }

    /// Checks for any error.
    ///
    /// Finish the test with q()
    #[must_use]
    pub fn err_any(&self) -> &Self {
        match &self.result {
            Ok(_) => {
                println!("FAIL: Expected error, but was ok!");
                self.flag_fail();
            }
            Err(_) => {}
        }
        self
    }

    /// Runs the associated Report. Depending on the type of the Report this
    /// can panic if any of the tests signaled a failure condition.
    ///
    /// Panic
    ///
    /// Panics if any test failed.
    #[track_caller]
    pub fn q<R: Report<Self> + Copy>(&self, r: R) {
        r.report(self);
    }
}

// works for any fn that uses a Span as input and returns a (Span, X) pair.
impl<'s, P, I, O, E> Test<'s, P, I, O, E>
where
    I: AsBytes + Copy + Debug + PartialEq + 's,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    O: Debug,
    E: Debug,
{
    /// Checks for ok results.
    ///
    /// This takes a CompareFn to convert the parser result to a type which can be compared
    /// with the test value.
    ///
    /// Finish the test with q()
    #[must_use]
    pub fn ok<V>(&'s self, eq: CompareFn<O, V>, test: V) -> &Self
    where
        V: Debug + Copy,
        O: Debug,
    {
        match &self.result {
            Ok((_, token)) => {
                if !eq(token, test) {
                    println!("FAIL: Value mismatch: {:?} <> {:?}", token, test);
                    self.flag_fail();
                }
            }
            Err(_) => {
                println!("FAIL: Expect ok, but was an error!");
                self.flag_fail();
            }
        }
        self
    }

    /// Tests the remaining string after parsing.
    ///
    /// Finish the test with q()
    #[must_use]
    pub fn rest<'a, T>(&'a self, test: T) -> &Self
    where
        &'a I: SpanFragment<Result = T>,
        I: 'a,
        T: PartialEq + Debug,
    {
        match &self.result {
            Ok((rest, _)) => {
                if rest.fragment() != test {
                    println!(
                        "FAIL: Rest mismatch {:?} <> {:?}",
                        restrict(DebugWidth::Medium, *rest),
                        test
                    );
                    self.flag_fail();
                }
            }
            Err(_) => {
                println!("FAIL: Expect ok, but was an error!");
                self.flag_fail();
            }
        }
        self
    }
}

// works for any NomFn.
impl<'s, P, I, O> Test<'s, P, I, O, nom::error::Error<I>>
where
    I: AsBytes + Copy + Debug + 's,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    O: Debug,
{
    /// Test for a nom error that occurred.
    #[must_use]
    pub fn nom_err(&self, kind: nom::error::ErrorKind) -> &Self {
        match &self.result {
            Ok(_) => {
                println!("FAIL: Expected error, but was ok!");
                self.flag_fail();
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                if e.code != kind {
                    println!("FAIL: {:?} <> {:?}", e.code, kind);
                    self.flag_fail();
                }
            }
            Err(nom::Err::Incomplete(_)) => {
                println!("FAIL: nom::Err::Incomplete");
                self.flag_fail();
            }
        }
        self
    }
}

impl<'s, P, C, I, O> Test<'s, P, I, O, ParserError<C, I>>
where
    I: AsBytes + Copy + Debug + 's,
    I: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    C: Code,
    O: Debug,
{
    /// Checks for an error.
    ///
    /// Finish the test with q()
    #[must_use]
    pub fn err(&self, code: C) -> &Self {
        match &self.result {
            Ok(_) => {
                println!("FAIL: Expected error, but was ok!");
                self.flag_fail();
            }
            Err(nom::Err::Error(e)) => {
                if e.code != code {
                    println!("ERROR: {:?} <> {:?}", e.code, code);
                    self.flag_fail();
                }
            }
            Err(nom::Err::Failure(e)) => {
                if e.code != code {
                    println!("FAILURE: {:?} <> {:?}", e.code, code);
                    self.flag_fail();
                }
            }
            Err(nom::Err::Incomplete(e)) => {
                println!("INCOMPLETE: {:?}", e);
                self.flag_fail();
            }
        }
        self
    }

    /// Checks for an expect value.
    ///
    /// Finish the test with q()
    #[must_use]
    pub fn expect(&self, code: C) -> &Self {
        match &self.result {
            Ok(_) => {
                println!("FAIL: {:?} was ok not an error.", code,);
                self.flag_fail();
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                if !e.is_expected(code) {
                    println!(
                        "FAIL: {:?} is not an expected token. {:?}",
                        code,
                        e.iter_expected().collect::<Vec<_>>()
                    );
                    self.flag_fail();
                }
            }
            Err(nom::Err::Incomplete(e)) => {
                println!("FAIL: {:?} was incomplete not an error. {:?}", code, e);
                self.flag_fail();
            }
        }

        self
    }
}

mod span {
    use crate::tracker::TrackSpan;
    use crate::Code;
    use nom::AsBytes;

    /// Compares a Span with a tuple (offset, str).
    /// To be used with Test::ok().
    #[allow(clippy::needless_lifetimes)]
    pub fn span<'a, 's, T, C>(span: &'a TrackSpan<'s, C, T>, value: (usize, T)) -> bool
    where
        T: AsBytes + Copy + PartialEq,
        C: Code,
    {
        **span == value.1 && span.location_offset() == value.0
    }

    /// Compares a tuple (Option<Span<'s>>, Span<'s>) with the test tuple (offset, str).
    /// Compares only the first tuple element. Fails if it is None.
    #[allow(clippy::needless_lifetimes)]
    pub fn span_0<'a, 's, T, C>(
        span: &'a (Option<TrackSpan<'s, C, T>>, TrackSpan<'s, C, T>),
        value: (usize, T),
    ) -> bool
    where
        T: AsBytes + Copy + PartialEq,
        C: Code,
    {
        if let Some(span) = &span.0 {
            **span == value.1 && span.location_offset() == value.0
        } else {
            false
        }
    }

    /// Check that the first element of a tuple (Option<Span<'s>>, Span<'s>) is None.
    #[allow(clippy::needless_lifetimes)]
    pub fn span_0_isnone<'a, 's, T, C>(
        span: &'a (Option<TrackSpan<'s, C, T>>, TrackSpan<'s, C, T>),
        _value: (),
    ) -> bool
    where
        T: AsBytes + Copy + PartialEq,
        C: Code,
    {
        span.0.is_none()
    }

    /// Compare a tuple (Option<Span<'s>>, Span<'s>) with the test tuple (offset, str).
    /// Compares the second element.
    #[allow(clippy::needless_lifetimes)]
    pub fn span_1<'a, 's, T, C>(
        span: &'a (Option<TrackSpan<'s, C, T>>, TrackSpan<'s, C, T>),
        value: (usize, T),
    ) -> bool
    where
        T: AsBytes + Copy + PartialEq,
        C: Code,
    {
        *span.1 == value.1 && span.1.location_offset() == value.0
    }
}

mod report {
    use crate::debug::{restrict, restrict_ref, DebugWidth};
    use crate::test::{Report, Test};
    use crate::tracker::{StdTracker, TrackSpan};
    use crate::Code;
    use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
    use nom_locate::LocatedSpan;
    use std::fmt::Debug;
    use std::ops::{RangeFrom, RangeTo};

    /// Do nothing report.
    #[derive(Clone, Copy)]
    pub struct NoReport;

    impl<'s, P, I, O, E> Report<Test<'s, P, I, O, E>> for NoReport {
        fn report(&self, _: &Test<'s, P, I, O, E>) {}
    }

    /// Dumps the Result data if any test failed.
    #[derive(Clone, Copy)]
    pub struct CheckDump;

    impl<'s, P, I, O, E> Report<Test<'s, P, I, O, E>> for CheckDump
    where
        I: AsBytes + Copy + Debug,
        I: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        O: Debug,
        E: Debug,
    {
        #[track_caller]
        fn report(&self, test: &Test<'s, P, I, O, E>) {
            if test.failed.get() {
                dump(test);
                panic!("test failed")
            }
        }
    }

    /// Dumps the Result data.
    #[derive(Clone, Copy)]
    pub struct Timing(pub u32);

    impl<'s, P, I, O, E> Report<Test<'s, P, I, O, E>> for Timing
    where
        I: AsBytes + Copy + Debug,
        I: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        O: Debug,
        E: Debug,
    {
        fn report(&self, test: &Test<'s, P, I, O, E>) {
            println!(
                "when parsing {:?} in {:?} =>",
                restrict(DebugWidth::Medium, test.span),
                test.duration / self.0
            );
            match &test.result {
                Ok(_) => {
                    println!("OK");
                }
                Err(_) => {
                    println!("ERROR");
                }
            }
        }
    }

    /// Dumps the Result data.
    #[derive(Clone, Copy)]
    pub struct Dump;

    impl<'s, P, I, O, E> Report<Test<'s, P, I, O, E>> for Dump
    where
        I: AsBytes + Copy + Debug,
        I: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        O: Debug,
        E: Debug,
    {
        fn report(&self, test: &Test<'s, P, I, O, E>) {
            dump(test)
        }
    }

    fn dump<P, I, O, E>(test: &Test<'_, P, I, O, E>)
    where
        I: AsBytes + Copy + Debug,
        I: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        O: Debug,
        E: Debug,
    {
        println!();
        println!(
            "when parsing {:?} in {:?} =>",
            restrict(DebugWidth::Medium, test.span),
            test.duration
        );
        match &test.result {
            Ok((rest, token)) => {
                println!("parsed");
                println!("    {:0?}", token);
                println!("rest");
                println!("    {}:{:?}", test.span.offset(rest), rest);
            }
            Err(e) => {
                println!("error");
                println!("    {:1?}", e);
            }
        }
    }

    /// Dumps the full parser trace if any test failed.
    #[derive(Clone, Copy)]
    pub struct CheckTrace;

    /// Dumps the full parser trace.
    #[derive(Clone, Copy)]
    pub struct Trace;

    impl<'s, C, T, O, E> Report<Test<'s, StdTracker<C, T>, TrackSpan<'s, C, T>, O, E>> for CheckTrace
    where
        T: AsBytes + Copy + Debug,
        T: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        C: Code,
        O: Debug,
        E: Debug,
    {
        #[track_caller]
        fn report(&self, test: &Test<'s, StdTracker<C, T>, TrackSpan<'s, C, T>, O, E>) {
            if test.failed.get() {
                trace(test);
                panic!("test failed")
            }
        }
    }

    impl<'s, C, T, O, E> Report<Test<'s, StdTracker<C, T>, TrackSpan<'s, C, T>, O, E>> for Trace
    where
        T: AsBytes + Copy + Debug,
        T: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        C: Code,
        O: Debug,
        E: Debug,
    {
        fn report(&self, test: &Test<'s, StdTracker<C, T>, TrackSpan<'s, C, T>, O, E>) {
            trace(test);
        }
    }

    fn trace<'s, C, T, O, E>(test: &Test<'s, StdTracker<C, T>, TrackSpan<'s, C, T>, O, E>)
    where
        T: AsBytes + Copy + Debug,
        T: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        C: Code,
        O: Debug,
        E: Debug,
    {
        println!();
        println!(
            "when parsing {:?} in {:?} =>",
            restrict_ref(DebugWidth::Medium, test.span.fragment()),
            test.duration
        );

        let tracks = test.context.results();
        print!("{:?}", tracks);

        match &test.result {
            Ok((rest, token)) => {
                println!("parsed");
                println!("    {:0?}", token);
                println!("rest");
                println!(
                    "    {}:{:?}",
                    rest.location_offset(),
                    restrict_ref(DebugWidth::Medium, rest.fragment()),
                );
            }
            Err(nom::Err::Error(e)) => {
                println!("error");
                println!("    {:1?}", e);
            }
            Err(nom::Err::Failure(e)) => {
                println!("failure");
                println!("    {:1?}", e);
            }
            Err(nom::Err::Incomplete(e)) => {
                println!("incomplete");
                println!("    {:1?}", e);
            }
        }
    }

    impl<'s, T, O, E> Report<Test<'s, (), LocatedSpan<T, ()>, O, E>> for CheckTrace
    where
        T: AsBytes + Copy + Debug,
        T: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        O: Debug,
        E: Debug,
    {
        #[track_caller]
        fn report(&self, test: &Test<'s, (), LocatedSpan<T, ()>, O, E>) {
            if test.failed.get() {
                trace_span(test);
                panic!("test failed")
            }
        }
    }

    impl<'s, T, O, E> Report<Test<'s, (), LocatedSpan<T, ()>, O, E>> for Trace
    where
        T: AsBytes + Copy + Debug,
        T: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        O: Debug,
        E: Debug,
    {
        fn report(&self, test: &Test<'s, (), LocatedSpan<T, ()>, O, E>) {
            trace_span(test);
        }
    }

    fn trace_span<'s, T, O, E>(test: &Test<'s, (), LocatedSpan<T, ()>, O, E>)
    where
        T: AsBytes + Copy + Debug,
        T: Offset
            + InputTake
            + InputIter
            + InputLength
            + Slice<RangeFrom<usize>>
            + Slice<RangeTo<usize>>,
        O: Debug,
        E: Debug,
    {
        println!();
        println!(
            "when parsing {:?} in {:?} =>",
            restrict_ref(DebugWidth::Medium, test.span.fragment()),
            test.duration
        );

        println!("trace");
        println!("    no trace");

        match &test.result {
            Ok((rest, token)) => {
                println!("parsed");
                println!("    {:0?}", token);
                println!("rest");
                println!(
                    "    {}:{:?}",
                    rest.location_offset(),
                    restrict_ref(DebugWidth::Medium, rest.fragment()),
                );
            }
            Err(nom::Err::Error(e)) => {
                println!("error");
                println!("    {:1?}", e);
            }
            Err(nom::Err::Failure(e)) => {
                println!("failure");
                println!("    {:1?}", e);
            }
            Err(nom::Err::Incomplete(e)) => {
                println!("incomplete");
                println!("    {:1?}", e);
            }
        }
    }

    impl<'s, O, E> Report<Test<'s, (), &'s str, O, E>> for CheckTrace
    where
        O: Debug,
        E: Debug,
    {
        #[track_caller]
        fn report(&self, test: &Test<'s, (), &'s str, O, E>) {
            if test.failed.get() {
                trace_less(test);
                panic!("test failed")
            }
        }
    }

    impl<'s, O, E> Report<Test<'s, (), &'s str, O, E>> for Trace
    where
        O: Debug,
        E: Debug,
    {
        fn report(&self, test: &Test<'s, (), &'s str, O, E>) {
            trace_less(test);
        }
    }

    fn trace_less<'s, O, E>(test: &Test<'s, (), &'s str, O, E>)
    where
        O: Debug,
        E: Debug,
    {
        println!();
        println!(
            "when parsing {:?} in {:?} =>",
            restrict_ref(DebugWidth::Medium, &test.span),
            test.duration
        );

        println!("trace");
        println!("    no trace");

        match &test.result {
            Ok((rest, token)) => {
                println!("parsed");
                println!("    {:0?}", token);
                println!("rest");
                println!("    {:?}", restrict_ref(DebugWidth::Medium, rest));
            }
            Err(nom::Err::Error(e)) => {
                println!("error");
                println!("    {:1?}", e);
            }
            Err(nom::Err::Failure(e)) => {
                println!("failure");
                println!("    {:1?}", e);
            }
            Err(nom::Err::Incomplete(e)) => {
                println!("incomplete");
                println!("    {:1?}", e);
            }
        }
    }

    impl<'s, O, E> Report<Test<'s, (), &'s [u8], O, E>> for CheckTrace
    where
        O: Debug,
        E: Debug,
    {
        #[track_caller]
        fn report(&self, test: &Test<'s, (), &'s [u8], O, E>) {
            if test.failed.get() {
                trace_less_b(test);
                panic!("test failed")
            }
        }
    }

    impl<'s, O, E> Report<Test<'s, (), &'s [u8], O, E>> for Trace
    where
        O: Debug,
        E: Debug,
    {
        fn report(&self, test: &Test<'s, (), &'s [u8], O, E>) {
            trace_less_b(test);
        }
    }

    fn trace_less_b<'s, O, E>(test: &Test<'s, (), &'s [u8], O, E>)
    where
        O: Debug,
        E: Debug,
    {
        println!();
        println!(
            "when parsing {:?} in {:?} =>",
            restrict_ref(DebugWidth::Medium, &test.span),
            test.duration
        );

        println!("trace");
        println!("    no trace");

        match &test.result {
            Ok((rest, token)) => {
                println!("parsed");
                println!("    {:0?}", token);
                println!("rest");
                println!("    {:?}", restrict_ref(DebugWidth::Medium, rest));
            }
            Err(nom::Err::Error(e)) => {
                println!("error");
                println!("    {:1?}", e);
            }
            Err(nom::Err::Failure(e)) => {
                println!("failure");
                println!("    {:1?}", e);
            }
            Err(nom::Err::Incomplete(e)) => {
                println!("incomplete");
                println!("    {:1?}", e);
            }
        }
    }
}
