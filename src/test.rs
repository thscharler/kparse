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

use crate::{Code, NoTracker, ParserError, StdTracker, TrackSpan};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use std::cell::Cell;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};
use std::time::{Duration, Instant};

use crate::debug::{restrict, DebugWidth};
pub use report::*;
pub use span::*;

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
    buf.replace(StdTracker::new(true));
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

/// Runs the parser and records the results.
/// Use ok(), err(), ... to check specifics.
///
/// Finish the test with q().
#[must_use]
pub fn notrack_parse<'s, C, T, O, E>(
    buf: &'s mut Option<StdTracker<C, T>>,
    text: T,
    fn_test: impl Fn(TrackSpan<'s, C, T>) -> Result<(TrackSpan<'s, C, T>, O), nom::Err<E>>,
) -> Test<'s, StdTracker<C, T>, TrackSpan<'s, C, T>, O, E>
where
    T: AsBytes + Copy,
    C: Code,
{
    buf.replace(StdTracker::new(false));
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

/// Runs the parser and records the results.
/// Use ok(), err(), ... to check specifics.
///
/// Finish the test with q().
#[must_use]
pub fn noctx_parse<'s, C, T, O, E>(
    buf: &'s mut Option<NoTracker>,
    text: T,
    fn_test: impl Fn(TrackSpan<'s, C, T>) -> Result<(TrackSpan<'s, C, T>, O), nom::Err<E>>,
) -> Test<'s, NoTracker, TrackSpan<'s, C, T>, O, E>
where
    T: AsBytes + Copy + 's,
    C: Code + 's,
{
    buf.replace(NoTracker);
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

/// Runs the parser and records the results.
/// Use ok(), err(), ... to check specifics.
///
/// Finish the test with q().
#[must_use]
pub fn str_parse<'s, O, E>(
    text: &str,
    fn_test: impl Fn(&str) -> Result<(&str, O), nom::Err<E>>,
) -> Test<'s, (), &str, O, E> {
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

/// Runs the parser and records the results.
/// Use ok(), err(), ... to check specifics.
///
/// Finish the test with q().
#[must_use]
pub fn byte_parse<'s, O, E>(
    text: &[u8],
    fn_test: impl Fn(&[u8]) -> Result<(&[u8], O), nom::Err<E>>,
) -> Test<'s, (), &[u8], O, E> {
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

impl<'s, P, C, T, O, E> Test<'s, P, TrackSpan<'s, C, T>, O, E>
where
    T: AsBytes + Copy + Debug + 's,
    C: Code,
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
    pub fn okok(&self) -> &Self {
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
    pub fn errerr(&self) -> &Self {
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
impl<'s, P, C, T, O, E> Test<'s, P, TrackSpan<'s, C, T>, O, E>
where
    T: AsBytes + Copy + Debug + PartialEq + 's,
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
    pub fn rest(&self, test: T) -> &Self {
        match &self.result {
            Ok((rest, _)) => {
                if **rest != test {
                    println!(
                        "FAIL: Rest mismatch {:?} <> {:?}",
                        restrict(DebugWidth::Medium, *rest).fragment(),
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
impl<'s, P, C, T, O> Test<'s, P, TrackSpan<'s, C, T>, O, nom::error::Error<TrackSpan<'s, C, T>>>
where
    T: AsBytes + Copy + Debug + 's,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    C: Code,
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

impl<'s, P, C, T, O> Test<'s, P, TrackSpan<'s, C, T>, O, ParserError<C, TrackSpan<'s, C, T>>>
where
    T: AsBytes + Copy + Debug + 's,
    T: Offset
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
    use crate::{Code, TrackSpan};
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
    use crate::debug::{restrict, restrict_str, DebugWidth};
    use crate::test::{Report, Test};
    use crate::{Code, StdTracker, TrackSpan};
    use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
    use std::fmt::Debug;
    use std::ops::{RangeFrom, RangeTo};

    /// Do nothing report.
    #[derive(Clone, Copy)]
    pub struct NoReport;

    impl<'s, P, C, T, O, E> Report<Test<'s, P, TrackSpan<'s, C, T>, O, E>> for NoReport
    where
        C: Code,
    {
        fn report(&self, _: &Test<'s, P, TrackSpan<'s, C, T>, O, E>) {}
    }

    /// Dumps the Result data if any test failed.
    #[derive(Clone, Copy)]
    pub struct CheckDump;

    impl<'s, P, C, T, O, E> Report<Test<'s, P, TrackSpan<'s, C, T>, O, E>> for CheckDump
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
        fn report(&self, test: &Test<'s, P, TrackSpan<'s, C, T>, O, E>) {
            if test.failed.get() {
                dump(test);
                panic!("test failed")
            }
        }
    }

    /// Dumps the Result data.
    #[derive(Clone, Copy)]
    pub struct Timing(pub u32);

    impl<'s, P, C, T, O, E> Report<Test<'s, P, TrackSpan<'s, C, T>, O, E>> for Timing
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
        fn report(&self, test: &Test<'s, P, TrackSpan<'s, C, T>, O, E>) {
            println!(
                "when parsing {:?} in {:?} =>",
                restrict_str(DebugWidth::Medium, test.span),
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

    impl<'s, P, C, T, O, E> Report<Test<'s, P, TrackSpan<'s, C, T>, O, E>> for Dump
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
        fn report(&self, test: &Test<'s, P, TrackSpan<'s, C, T>, O, E>) {
            dump(test)
        }
    }

    fn dump<'s, P, C, T, O, E>(test: &Test<'s, P, TrackSpan<'s, C, T>, O, E>)
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
            restrict_str(DebugWidth::Medium, test.span),
            test.duration
        );
        match &test.result {
            Ok((rest, token)) => {
                println!("rest {}:{:?}", rest.location_offset(), rest);
                println!("{:0?}", token);
            }
            Err(e) => {
                println!("error");
                println!("{:1?}", e);
            }
        }
    }

    /// Dumps the full parser trace if any test failed.
    #[derive(Clone, Copy)]
    pub struct CheckTrace;

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

    /// Dumps the full parser trace.
    #[derive(Clone, Copy)]
    pub struct Trace;

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
            restrict(DebugWidth::Medium, test.span),
            test.duration
        );

        let tracks = test.context.results();
        println!("{:?}", tracks);

        match &test.result {
            Ok((rest, token)) => {
                println!(
                    "rest {}:{:?}",
                    rest.location_offset(),
                    restrict(DebugWidth::Medium, *rest)
                );
                println!("{:0?}", token);
            }
            Err(e) => {
                println!("error");
                println!("{:1?}", e);
            }
        }
    }
}
