# kparse

Addons for a nom parser.

* A error code trait.
* A richer error type ParserError.
* Traits to integrate external errors.
* A tracking system for the parser.
* A simple framework to test parser functions.
* SpanLines and SpanBytes to get the context around a span.

The complete code can be found as [examples/example1.rs].

```rust
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
are recommended.

```rust
pub type ESpan<'s> = TrackSpan<'s, ECode, &'s str>;
pub type EResult<'s, O> = TrackResult<ECode, ESpan<'s>, O, ()>;
pub type ENomResult<'s> = TrackResult<ECode, ESpan<'s>, ESpan<'s>, ()>;
pub type EParserError<'s> = ParserError<ECode, ESpan<'s>, ()>;

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
fn token_number(i: ESpan<'_>) -> EResult<'_, AstNumber<'_>> {
    match nom_number(i) {
        Ok((rest, (tok, val))) => Ok((
            rest,
            AstNumber {
                number: val,
                span: tok,
            },
        )),
        Err(e) => Err(e.with_code(ENumber)),
    }
}
```

## IResult

ParserError implements nom::error::ParseError, it can be used instead of
nom::error::Error.

## Error handling

### WithSpan

The trait WithSpan is used to convert an external error to a ParserError
and add an error code and a span at the same time.

```rust
impl<'s, C: Code, Y: Copy> WithSpan<'s, C, nom::Err<ParserError<'s, C, Y>>>
for std::num::ParseIntError
{
    fn with_span(self, code: C, span: Span<'s, C>) -> nom::Err<ParserError<'s, C, Y>>
    {
        nom::Err::Failure(ParserError::new(code, span))
    }
}
```

With the combinator ```transform(...)``` this can be integrated in the
parser.

```rust
fn nom_number(i: ESpan<'_>) -> EResult<'_, (ESpan<'_>, u32)> {
    consumed(transform(
        terminated(digit1, nom_ws),
        |v| (*v).parse::<u32>(),
        ENumber,
    ))(i)
}
```

### WithCode

The trait WithCode allows altering the error code. The previous error code
is kept as a hint.

```rust
fn token_number(i: ESpan<'_>) -> EResult<'_, AstNumber<'_>> {
    match nom_number(i) {
        Ok((rest, (tok, val))) => Ok((
            rest,
            AstNumber {
                number: val,
                span: tok,
            },
        )),
        Err(e) => Err(e.with_code(ENumber)),
    }
}
```

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
    track_parse(&mut None, "", parse_ab).err_any().q(CheckTrace);
    track_parse(&mut None, "ab", parse_ab)
        .ok_any()
        .q(CheckTrace);
    track_parse(&mut None, "aba", parse_ab)
        .rest("a")
        .q(CheckTrace);
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

## transform()

Combines parsing and value conversion. If the external error type implements
WithSpan this looks quite smooth.

## error_code()

Change the error_code of a partial parser.

## conditional()

Runs a condition function on the input and only runs the parser function
if it succeeds. There is nom::cond(), but it's not the the same.


# Error reporting

# SpanExt

This trait is kind of a undo of parsing. It takes two output spans and
can create a span that covers both of them and anything between.

nom has consumed() and recognize() for this, which work fine too.

# SpanLines and SpanBytes

"Ok, so now I got the error, but what was the context?"

SpanLines can help. It contains the complete parser input and can
find the text lines surrounding any given span returned by the error.

SpanBytes does the same with &[u8]. 

