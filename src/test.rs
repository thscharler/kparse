//!
//! Test framework for parsers.
//!

use crate::debug::{restrict, DebugWidth};
use crate::{Code, NoContext, ParserError, Span, TrackingContext};
use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
use std::cell::Cell;
use std::fmt::Debug;
use std::ops::{RangeFrom, RangeTo};
use std::time::{Duration, Instant};

pub use report::*;
pub use span::*;

/// Value comparison.
pub type CompareFn<O, V> = for<'a> fn(&'a O, V) -> bool;

/// Test Results.
pub struct Test<'s, P, T, C, O, E>
where
    C: Code,
{
    /// text
    pub text: T,
    /// ParseContext
    pub context: &'s P,
    /// Test Result
    pub result: Result<(Span<'s, T, C>, O), nom::Err<E>>,
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
pub fn track_parse<'s, T: AsBytes + Copy + 's, C: Code, O, E>(
    buf: &'s mut Option<TrackingContext<'s, T, C>>,
    text: T,
    fn_test: impl Fn(Span<'s, T, C>) -> Result<(Span<'s, T, C>, O), nom::Err<E>>,
) -> Test<'s, TrackingContext<'s, T, C>, T, C, O, E> {
    buf.replace(TrackingContext::new(true));
    let context = buf.as_ref().expect("yes");

    let span = context.span(text);

    let now = Instant::now();
    let result = fn_test(span);
    let elapsed = now.elapsed();

    Test {
        text,
        context,
        result,
        duration: elapsed,
        failed: Cell::new(false),
    }
}

/// Runs the parser and records the results.
/// Use ok(), err(), ... to check specifics.
///
/// Finish the test with q().
#[must_use]
pub fn notrack_parse<'s, T: AsBytes + Copy + 's, C: Code, O, E>(
    buf: &'s mut Option<TrackingContext<'s, T, C>>,
    text: T,
    fn_test: impl Fn(Span<'s, T, C>) -> Result<(Span<'s, T, C>, O), nom::Err<E>>,
) -> Test<'s, TrackingContext<'s, T, C>, T, C, O, E> {
    buf.replace(TrackingContext::new(false));
    let context = buf.as_ref().expect("yes");

    let span = context.span(text);

    let now = Instant::now();
    let result = fn_test(span);
    let elapsed = now.elapsed();

    Test {
        text,
        context,
        result,
        duration: elapsed,
        failed: Cell::new(false),
    }
}

/// Runs the parser and records the results.
/// Use ok(), err(), ... to check specifics.
///
/// Finish the test with q().
#[must_use]
pub fn noctx_parse<'s, T: AsBytes + Copy + 's, C: Code, O, E>(
    buf: &'s mut Option<NoContext>,
    text: T,
    fn_test: impl Fn(Span<'s, T, C>) -> Result<(Span<'s, T, C>, O), nom::Err<E>>,
) -> Test<'s, NoContext, T, C, O, E> {
    buf.replace(NoContext);
    let context = buf.as_ref().expect("yes");

    let span = context.span(text);

    let now = Instant::now();
    let result = fn_test(span);
    let elapsed = now.elapsed();

    Test {
        text,
        context,
        result,
        duration: elapsed,
        failed: Cell::new(false),
    }
}

impl<'s, P, T, C, O, E> Test<'s, P, T, C, O, E>
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
impl<'s, P, T, C, O, E> Test<'s, P, T, C, O, E>
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
    /// Checks for ok.
    /// Uses an extraction function to get the relevant result.
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
impl<'s, P, T, C, O> Test<'s, P, T, C, O, nom::error::Error<Span<'s, T, C>>>
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

impl<'s, P, T, C, O> Test<'s, P, T, C, O, ParserError<'s, T, C>>
where
    T: AsBytes + Copy + Debug + 's,
    T: Offset
        + InputTake
        + InputIter
        + InputLength
        + Slice<RangeFrom<usize>>
        + Slice<RangeTo<usize>>,
    O: Debug,
    C: Code,
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
    use crate::{Code, Span};
    use nom::AsBytes;

    /// Compare with an Ok(Span<'s>)
    #[allow(clippy::needless_lifetimes)]
    pub fn span<'a, 's, T: AsBytes + Copy + PartialEq, C: Code>(
        span: &'a Span<'s, T, C>,
        value: (usize, T),
    ) -> bool {
        **span == value.1 && span.location_offset() == value.0
    }

    /// Compare with an Ok(Option<Span<'s>>, Span<'s>). Use the first span, fail on None.
    #[allow(clippy::needless_lifetimes)]
    pub fn span_0<'a, 's, T: AsBytes + Copy + PartialEq, C: Code>(
        span: &'a (Option<Span<'s, T, C>>, Span<'s, T, C>),
        value: (usize, T),
    ) -> bool {
        if let Some(span) = &span.0 {
            **span == value.1 && span.location_offset() == value.0
        } else {
            false
        }
    }

    /// Compare with an Ok(Option<Span<'s>>, Span<'s>). Use the first span, fail on Some.
    #[allow(clippy::needless_lifetimes)]
    pub fn span_0_isnone<'a, 's, T: AsBytes + Copy + PartialEq, C: Code>(
        span: &'a (Option<Span<'s, T, C>>, Span<'s, T, C>),
        _value: (),
    ) -> bool {
        span.0.is_none()
    }

    /// Compare with an Ok(Option<Span<'s>>, Span<'s>). Use the second span.
    #[allow(clippy::needless_lifetimes)]
    pub fn span_1<'a, 's, T: AsBytes + Copy + PartialEq, C: Code>(
        span: &'a (Option<Span<'s, T, C>>, Span<'s, T, C>),
        value: (usize, T),
    ) -> bool {
        *span.1 == value.1 && span.1.location_offset() == value.0
    }
}

mod report {
    use crate::debug::{restrict, restrict_str, DebugWidth, Tracks};
    use crate::test::{Report, Test};
    use crate::{Code, TrackingContext};
    use nom::{AsBytes, InputIter, InputLength, InputTake, Offset, Slice};
    use std::fmt::Debug;
    use std::ops::{RangeFrom, RangeTo};

    /// Do nothing report.
    #[derive(Clone, Copy)]
    pub struct NoReport;

    impl<P, T, C, O, E> Report<Test<'_, P, T, C, O, E>> for NoReport
    where
        C: Code,
        O: Debug,
        E: Debug,
    {
        fn report(&self, _: &Test<'_, P, T, C, O, E>) {}
    }

    /// Dumps the Result data if any test failed.
    #[derive(Clone, Copy)]
    pub struct CheckDump;

    impl<P, T, C, O, E> Report<Test<'_, P, T, C, O, E>> for CheckDump
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
        fn report(&self, test: &Test<'_, P, T, C, O, E>) {
            if test.failed.get() {
                dump(test);
                panic!("test failed")
            }
        }
    }

    /// Dumps the Result data.
    #[derive(Clone, Copy)]
    pub struct Timing(pub u32);

    impl<P, T, C, O, E> Report<Test<'_, P, T, C, O, E>> for Timing
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
        fn report(&self, test: &Test<'_, P, T, C, O, E>) {
            println!(
                "when parsing {:?} in {:?} =>",
                restrict_str(DebugWidth::Medium, test.text),
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

    impl<P, T, C, O, E> Report<Test<'_, P, T, C, O, E>> for Dump
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
        fn report(&self, test: &Test<'_, P, T, C, O, E>) {
            dump(test)
        }
    }

    fn dump<P, T, C, O, E>(test: &Test<'_, P, T, C, O, E>)
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
            restrict_str(DebugWidth::Medium, test.text),
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

    impl<'s, T, C, O, E> Report<Test<'s, TrackingContext<'s, T, C>, T, C, O, E>> for CheckTrace
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
        fn report(&self, test: &Test<'s, TrackingContext<'s, T, C>, T, C, O, E>) {
            if test.failed.get() {
                trace(test);
                panic!("test failed")
            }
        }
    }

    /// Dumps the full parser trace.
    #[derive(Clone, Copy)]
    pub struct Trace;

    impl<'s, T, C, O, E> Report<Test<'s, TrackingContext<'s, T, C>, T, C, O, E>> for Trace
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
        fn report(&self, test: &Test<'s, TrackingContext<'s, T, C>, T, C, O, E>) {
            trace(test);
        }
    }

    fn trace<'s, T, C, O, E>(test: &Test<'s, TrackingContext<'s, T, C>, T, C, O, E>)
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
            restrict_str(DebugWidth::Medium, test.text),
            test.duration
        );

        let tracks = test.context.results();
        println!("{:?}", Tracks(&tracks));

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
