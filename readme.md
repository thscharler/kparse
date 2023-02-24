# kparse

Addons for a nom parser.

* Trait Code for basic error codes.

* ParserError for full error collection and TokenizerError for fast inner loops.

* Tracking/Logging of the parser execution.

* Builder style tests that can do tests from simple ok/err to deep inspection
  of the results.
* With simple pluggable reporting too.

* Extended set of postfix adapters for a parser. Inspired by nom_supreme
  but integrated with the error Code and error types of this crate.

* SpanLines and SpanBytes to get context information around a span.
* Can also retrieve line/column information.
* From a plain &str too.

* All of the extras can be easily cfg'ed away for a release build.
* Usually it's just cfg(debug_assertions) vs cfg(not(debug_assertions)) to
  change the Input type from TrackSpan to plain &str.

The complete code can be found as [examples/example1.rs].

```rust
// := a* b
fn parse_a_star_b(input: ExSpan<'_>) -> ExParserResult<'_, AstAstarB> {
    track(
        ExAstarB,
        tuple((many0(parse_a), parse_b))
            .map(|(a, b)| AstAstarB { a, b }),
    )
        .err_into()
        .parse(input)
}

// := ( a | b )*
fn parse_a_b_star(input: ESpan<'_>) -> EResult<'_, AstABstar> {
    Context.enter(EABstar, input);

    let mut loop_rest = input;
    let mut res = AstABstar {
        a: vec![],
        b: vec![],
    };
    let mut err = None;

    loop {
        let rest2 = loop_rest;

        let rest2 = match parse_a(rest2) {
            Ok((rest3, a)) => {
                res.a.push(a);
                rest3
            }
            Err(e) => match parse_b(rest2) {
                Ok((rest3, b)) => {
                    res.b.push(b);
                    rest3
                }
                Err(e2) => {
                    err.append(e)?;
                    err.append(e2)?;
                    rest2
                }
            },
        };

        if let Some(err) = err {
            return Context.err(err);
        }
        if rest2.is_empty() {
            break;
        }

        loop_rest = rest2;
    }

    Context.ok(loop_rest, input, res)
}
```

# Basics

## prelude

There is a prelude for all common traits. 

## Error code

Define the error code enum. The error codes are used in actual error reporting
and as a marker when tracing the execution of the parser.

All the nom errorkind are mapped to one parser error and it's kept as extra
info.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ECode {
    ENomError,

    ETagA,
    ETagB,
    ENumber,

    EAthenB,
    EAoptB,
    EAstarB,
    EABstar,
    EAorB,
    EABNum,
}

impl Code for ECode {
    const NOM_ERROR: Self = Self::ENomError;
}
```

This crate is very heavy on type variables. The following type aliases
are recommended. With the two cfg's the parser can switch from detailed
tracking to release performance.

```rust
#[cfg(debug_assertions)]
pub type ExSpan<'s> = TrackSpan<'s, ExCode, &'s str>;
#[cfg(not(debug_assertions))]
pub type ExSpan<'s> = &'s str;
pub type ExParserResult<'s, O> = ParserResult<ExCode, ExSpan<'s>, O>;
pub type ExTokenizerResult<'s, O> = TokenizerResult<ExCode, ExSpan<'s>, O>;
pub type ExParserError<'s> = ParserError<ExCode, ExSpan<'s>>;
pub type ExTokenizerError<'s> = TokenizerError<ExCode, ExSpan<'s>>;
```

## AST

Define your parsers output as you wish. No constraints here.

```rust
#[derive(Debug)]
struct AstNumber<'s> {
    pub number: u32,
    pub span: ESpan<'s>,
}
```

## Parser functions

Parser functions are the same as with a plain nom parser, just using
different input and error types

```rust
fn token_number(i: ExSpan<'_>) -> ExParserResult<'_, AstNumber<'_>> {
    nom_number
        .map(|(span, number)| AstNumber { number, span })
        .parse(i)
}
```

## IResult

ParserError and TokenizerError implement nom::error::ParseError, they can be
used instead of nom::error::Error.

## Error handling

### err_into()

Error conversion with the From trait.

### parse_from_str()

Parsing with the FromStr trait. Takes a error code to create a error on fail.

### with_code()

Changes the error code of an error. The old error code is kept as an expected
code.

# Parser tracking

## Inside the parser

The tracker is added as the LocatedSpan.extra field, this way no extra
parameters are needed.

To access the tracker the Context struct is used.

```rust
fn parse_a(input: ESpan<'_>) -> EResult<'_, AstA> {
    Context.enter(ETagA, input);
    let (rest, tok) = nom_parse_a(input).track()?;

    if false {
        return Context.err(EParserError::new(EAorB, input));
    }

    Context.ok(rest, tok, AstA { span: tok })
}
```

enter() and ok() and err() capture the normal control flow of the
parser.

track() acts on Result to allow easy error propagation.

Note: There are track_as() and track_ok() too.

## Calling the parser

Create a StdTracker and call the parser with an annotated span.

```rust
fn main() {
    for txt in env::args() {
        let trk = StdTracker::new();
        let span = trk.span(txt.as_str());

        match parse_a_b_star(span) {
            Ok((rest, val)) => {}
            Err(e) => {
                println!("{:?}", trk.results());
                println!("{:?}", e);
            }
        }
    }
}
```

Tracking only works if a TrackSpan is used in the parser.

If the type alias points to a &str, a &[u8] or any LocatedSpan<T, ()>
everything still works, just without tracking.

## Getting the tracking data

The call to StdTracker::results() returns the tracking data.

# Testing the parser

The test module has several functions to run a test for one parser function
and to evaluate the result.

track_parse() runs the parser and returns a Test struct with a variety of
builder like functions to check the results. If any check went wrong the
q() call reports this as failed test.

q() takes one parameter that defines the actual report done.
CheckTrace is one of them, it dumps the trace and the error and panics.

```rust
#[test]
fn test_1() {
    str_parse(&mut None, "", parse_ab).err_any().q(Timing(1));
    str_parse(&mut None, "ab", parse_ab).ok_any().q(Timing(1));
    str_parse(&mut None, "aba", parse_ab).rest("a").q(Timing(1));
}
```

The result looks like this.

```txt
FAIL: Expected ok, but was an error.

when parsing LocatedSpan { offset: 0, line: 1, fragment: "aabc", extra:  } in 43.4µs =>
trace
  (A | B)*: enter with "aabc"
    a: enter with "aabc"
    a: ok -> [ "a", "abc" ]
    a: enter with "abc"
    a: ok -> [ "a", "bc" ]
    a: enter with "bc"
    a: err ENomError  errorkind Tag for span LocatedSpan { offset: 2, line: 1, fragment: "bc", extra:  } 
    b: enter with "bc"
    b: ok -> [ "b", "c" ]
    a: enter with "c"
    a: err ENomError  errorkind Tag for span LocatedSpan { offset: 3, line: 1, fragment: "c", extra:  } 
    b: enter with "c"
    b: err ENomError  errorkind Tag for span LocatedSpan { offset: 3, line: 1, fragment: "c", extra:  } 
  (A | B)*: err ENomError  errorkind Tag for span LocatedSpan { offset: 3, line: 1, fragment: "c", extra:  } 

error
ParserError nom for LocatedSpan { offset: 3, line: 1, fragment: "c", extra:  }
errorkind=Tag
```

# Combinators

Just some things I have been missing.

## track()

Tracks the call to the subparser.


## pchar()

Similar to nom's char function, but with an easier name and returns the
input type instead of char.

## err_into()

Error conversion with the From trait.

## with_code()

Changes the error code.

## separated_list_trailing0 and 1

Similar to separated_list, but allows for a trailing separator.

# Error reporting

# SpanUnion

This trait is kind of a undo of parsing. It takes two output spans and
can create a span that covers both of them and anything between.

nom has consumed() and recognize() for this, which work fine too.

# SpanLocation and SpanFragment

Provides the nom_locate functions location_offset(), location_line() and fragment() via traits. This way they are also available for &str etc.

# SpanLines, SpanStr and SpanBytes 

"Ok, so now I got the error, but what was the context?"

SpanLines can help. It contains the complete parser input and can
find the text lines surrounding any given span returned by the error.

It can also provide line number an column.

SpanBytes does the same with &[u8], SpanStr for &str.

# Performance

Expect some overhead when tracking is enabled.
When disabled with a different Span type the calls to Context etc boil down
to no-ops, so there should be no difference to a equivalent nom-only parser.

It is also possible to replace LocatedSpan completely which gives quite a 
boost. See example1.rs

### ParserError vs TokenizerError

ParserError is double the size of TokenizerError due to a Vec with all
the extra data. 

But as it tries to keep the nom ErrorKind it almost immediately allocates
for the vec. As most parser combinators work heavily with Err results this
can be quite heavy. There is the feature dont_track_nom to avoid this pit.

But maybe the better way is to use TokenizerError for lower level parsers
and switch to ParserError at the point where the extra features are needed.
With err_into() this is not too annoying.

