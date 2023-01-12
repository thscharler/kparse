use crate::debug::{restrict, DebugWidth};
use crate::{Code, ParseContext, ParserError, Span, TrackingContext};
use std::cell::Cell;
use std::fmt::Debug;
use std::time::{Duration, Instant};

use crate::raw_context::{new_no_context_span, RawContext};
pub use report::*;
pub use span::*;

/// Value comparison.
pub type CompareFn<O, V> = for<'a> fn(&'a O, V) -> bool;

/// Test Results.
pub struct Test<'s, P, C, O, E>
where
    P: ParseContext<'s, C>,
    C: Code,
{
    pub text: &'s str,
    pub context: &'s P,
    pub result: Result<(Span<'s, C>, O), nom::Err<E>>,
    pub duration: Duration,
    pub failed: Cell<bool>,
}

/// Result reporting.
pub trait Report<T> {
    fn report(&self, test: T);
}

/// Runs the parser and records the results.
/// Use ok(), err(), ... to check specifics.
///
/// Finish the test with q().
#[must_use]
pub fn test_parse<'s, C: Code, O, E>(
    buf: &'s mut Option<TrackingContext<'s, C, true>>,
    text: &'s str,
    fn_test: impl Fn(Span<'s, C>) -> Result<(Span<'s, C>, O), nom::Err<E>>,
) -> Test<'s, TrackingContext<'s, C, true>, C, O, E> {
    buf.replace(TrackingContext::new(text));
    let context = buf.as_ref().expect("yes");

    let span = context.span();

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
pub fn test_parse_no_track<'s, C: Code, O, E>(
    buf: &'s mut Option<TrackingContext<'s, C, false>>,
    text: &'s str,
    fn_test: impl Fn(Span<'s, C>) -> Result<(Span<'s, C>, O), nom::Err<E>>,
) -> Test<'s, TrackingContext<'s, C, false>, C, O, E> {
    buf.replace(TrackingContext::new(text));
    let context = buf.as_ref().expect("yes");

    let span = context.span();

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
pub fn test_parse_raw<'s, C: Code, O, E>(
    buf: &'s mut Option<RawContext<'s, C>>,
    text: &'s str,
    fn_test: impl Fn(Span<'s, C>) -> Result<(Span<'s, C>, O), nom::Err<E>>,
) -> Test<'s, RawContext<'s, C>, C, O, E> {
    buf.replace(RawContext::new(text));
    let context = buf.as_ref().expect("yes");

    let span = context.span();

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
pub fn test_parse_noctx<'s, C: Code, O, E>(
    _buf: &'s mut Option<()>,
    text: &'s str,
    fn_test: impl Fn(Span<'s, C>) -> Result<(Span<'s, C>, O), nom::Err<E>>,
) -> Test<'s, (), C, O, E> {
    let span = new_no_context_span(text);

    let now = Instant::now();
    let result = fn_test(span);
    let elapsed = now.elapsed();

    Test {
        text,
        context: &(),
        result,
        duration: elapsed,
        failed: Cell::new(false),
    }
}

impl<'s, P, C, O, E> Test<'s, P, C, O, E>
where
    P: ParseContext<'s, C>,
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
    pub fn q<R: Report<Self> + Copy>(self, r: R) {
        r.report(self);
    }
}

// works for any fn that uses a Span as input and returns a (Span, X) pair.
impl<'s, P, C, O, E> Test<'s, P, C, O, E>
where
    P: ParseContext<'s, C>,
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
    pub fn rest(&self, test: &str) -> &Self {
        match &self.result {
            Ok((rest, _)) => {
                if **rest != test {
                    println!(
                        "FAIL: Rest mismatch {} <> {}",
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
impl<'s, P, C, O> Test<'s, P, C, O, nom::error::Error<Span<'s, C>>>
where
    P: ParseContext<'s, C>,
    C: Code,
    O: Debug,
{
    /// Test for a nom error that occurred.
    #[must_use]
    pub fn err(&self, kind: nom::error::ErrorKind) -> &Self {
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

impl<'s, P, C, O> Test<'s, P, C, O, ParserError<'s, C>>
where
    P: ParseContext<'s, C>,
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

    // /// Checks for an expect value.
    // ///
    // /// Finish the test with q()
    // #[must_use]
    // pub fn expect2(&self, code: C, parent: C) -> &Self {
    //     match &self.result {
    //         Ok(_) => {
    //             println!("FAIL: {:?} was ok not an error.", code,);
    //             self.flag_fail();
    //         }
    //         Err(e) => {
    //             if !e.is_expected2(code, parent) {
    //                 println!(
    //                     "FAIL: {:?} is not an expected token. {:?}",
    //                     code,
    //                     e.expect_as_ref()
    //                 );
    //                 self.flag_fail();
    //             }
    //         }
    //     }
    //
    //     self
    // }
}

mod span {
    use crate::{Code, Span};

    /// Compare with an Ok(Span<'s>)
    #[allow(clippy::needless_lifetimes)]
    pub fn span<'a, 'b, 's, C: Code>(span: &'a Span<'s, C>, value: (usize, &'b str)) -> bool {
        **span == value.1 && span.location_offset() == value.0
    }

    /// Compare with an Ok(Option<Span<'s>>, Span<'s>). Use the first span, fail on None.
    #[allow(clippy::needless_lifetimes)]
    pub fn span_0<'a, 'b, 's, C: Code>(
        span: &'a (Option<Span<'s, C>>, Span<'s, C>),
        value: (usize, &'b str),
    ) -> bool {
        if let Some(span) = &span.0 {
            **span == value.1 && span.location_offset() == value.0
        } else {
            false
        }
    }

    /// Compare with an Ok(Option<Span<'s>>, Span<'s>). Use the first span, fail on Some.
    #[allow(clippy::needless_lifetimes)]
    pub fn span_0_isnone<'a, 's, C: Code>(
        span: &'a (Option<Span<'s, C>>, Span<'s, C>),
        _value: (),
    ) -> bool {
        span.0.is_none()
    }

    /// Compare with an Ok(Option<Span<'s>>, Span<'s>). Use the second span.
    #[allow(clippy::needless_lifetimes)]
    pub fn span_1<'a, 'b, 's, C: Code>(
        span: &'a (Option<Span<'s, C>>, Span<'s, C>),
        value: (usize, &'b str),
    ) -> bool {
        *span.1 == value.1 && span.1.location_offset() == value.0
    }
}

mod report {
    use crate::debug::tracks::Tracks;
    use crate::debug::{restrict, restrict_str, DebugWidth};
    use crate::test::{Report, Test};
    use crate::{Code, ParseContext, TrackingContext};
    use std::fmt::Debug;

    #[derive(Clone, Copy)]
    pub struct NoReport;

    impl<'s, P, C, O, E> Report<Test<'s, P, C, O, E>> for NoReport
    where
        P: ParseContext<'s, C>,
        C: Code,
        O: Debug,
        E: Debug,
    {
        fn report(&self, _: Test<'s, P, C, O, E>) {}
    }

    /// Dumps the Result data if any test failed.
    #[derive(Clone, Copy)]
    pub struct CheckDump;

    impl<'s, P, C, O, E> Report<Test<'s, P, C, O, E>> for CheckDump
    where
        P: ParseContext<'s, C>,
        C: Code,
        O: Debug,
        E: Debug,
    {
        #[track_caller]
        fn report(&self, test: Test<'s, P, C, O, E>) {
            if test.failed.get() {
                dump(test);
                panic!("test failed")
            }
        }
    }

    /// Dumps the Result data.
    #[derive(Clone, Copy)]
    pub struct Timing(pub u32);

    impl<'s, P, C, O, E> Report<Test<'s, P, C, O, E>> for Timing
    where
        P: ParseContext<'s, C>,
        C: Code,
        O: Debug,
        E: Debug,
    {
        fn report(&self, test: Test<'s, P, C, O, E>) {
            println!(
                "when parsing '{}' in {} =>",
                restrict_str(DebugWidth::Medium, test.text),
                humantime::format_duration(test.duration / self.0)
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

    impl<'s, P, C, O, E> Report<Test<'s, P, C, O, E>> for Dump
    where
        P: ParseContext<'s, C>,
        C: Code,
        O: Debug,
        E: Debug,
    {
        fn report(&self, test: Test<'s, P, C, O, E>) {
            dump(test)
        }
    }

    fn dump<'s, P, C, O, E>(test: Test<'s, P, C, O, E>)
    where
        P: ParseContext<'s, C>,
        C: Code,
        O: Debug,
        E: Debug,
    {
        println!();
        println!(
            "when parsing '{}' in {} =>",
            restrict_str(DebugWidth::Medium, test.text),
            humantime::format_duration(test.duration)
        );
        match &test.result {
            Ok((rest, token)) => {
                println!("rest {}:\"{}\"", rest.location_offset(), rest);
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

    impl<'s, C, O, E, const TRACK: bool> Report<Test<'s, TrackingContext<'s, C, TRACK>, C, O, E>>
        for CheckTrace
    where
        C: Code,
        O: Debug,
        E: Debug,
    {
        #[track_caller]
        fn report(&self, test: Test<'s, TrackingContext<'s, C, TRACK>, C, O, E>) {
            if test.failed.get() {
                trace(test);
                panic!("test failed")
            }
        }
    }

    /// Dumps the full parser trace.
    #[derive(Clone, Copy)]
    pub struct Trace;

    impl<'s, C, O, E, const TRACK: bool> Report<Test<'s, TrackingContext<'s, C, TRACK>, C, O, E>>
        for Trace
    where
        C: Code,
        O: Debug,
        E: Debug,
    {
        fn report(&self, test: Test<'s, TrackingContext<'s, C, TRACK>, C, O, E>) {
            trace(test);
        }
    }

    fn trace<'s, C, O, E, const TRACK: bool>(test: Test<'s, TrackingContext<'s, C, TRACK>, C, O, E>)
    where
        C: Code,
        O: Debug,
        E: Debug,
    {
        println!();
        println!(
            "when parsing '{}' in {} =>",
            restrict_str(DebugWidth::Medium, test.text),
            humantime::format_duration(test.duration)
        );

        let tracks = test.context.results();
        println!("{:?}", Tracks(&tracks));

        match &test.result {
            Ok((rest, token)) => {
                println!(
                    "rest {}:\"{}\"",
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
