# kparse

Addons for a nom parser.

* Trait Code for error codes.

* ParserError for full error collection and TokenizerError for fast inner loops.

* Tracking/Logging of the parser execution.

* Builder style tests that can do tests from simple ok/err to deep
  inspection of the results.
* With pluggable reporting.

* Extended set of postfix adapters for a parser. Inspired by 
  nom_supreme but integrated with the error Code and error types 
  of this crate.


* SourceStr and SourceBytes to get context information for a span.
  * Line/Column information
  * Context source lines.
  * Works without LocatedSpan.

* By default the tracking function is only active in debug mode.
* In release mode all the tracking is compiled away to nothing. 

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
and as a marker when tracking the execution of the parser.

All the nom errorkind are mapped to one parser error.

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
define_span!(ExSpan = ExCode, str);
pub type ExParserResult<'s, O> = ParserResult<ExCode, ExSpan<'s>, O>;
pub type ExTokenizerResult<'s, O> = TokenizerResult<ExCode, ExSpan<'s>, O>;
pub type ExParserError<'s> = ParserError<ExCode, ExSpan<'s>>;
pub type ExTokenizerError<'s> = TokenizerError<ExCode, ExSpan<'s>>;
```

define_span creates a type alias for the data. It's result differs between
debug and release mode. 

ParserError can hold more than one error code and various extra data. 
On the other side TokenizerError is only one error code and a span to minimize
it's size.

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
code. This function is available as a combinator function and as a function
defined for Result<>.

# Parser tracking

## Inside the parser

The tracker is added as the LocatedSpan.extra field, this way no extra
parameters are needed.

To access the tracker the Track struct is used.

```rust
fn parse_a(input: ESpan<'_>) -> EResult<'_, AstA> {
    Track.enter(ETagA, input);
    let (rest, tok) = nom_parse_a(input).track()?;

    if false {
        return Track.err(EParserError::new(EAorB, input));
    }

    Track.ok(rest, tok, AstA { span: tok })
}
```

enter() and ok() and err() capture the normal control flow of the
parser.

track() acts on Result to allow easy error propagation.

Note: There is track_as(code: ExCode) to change the error code.

## Calling the parser

Create a StdTracker and call the parser with an annotated span.

Depending on debug/release mode Track.span() returns a LocatedSpan or the
original text. 

```rust
fn main() {
    for txt in env::args() {
        let trk = Track.new_tracker();
        let span = Track.span(trk, txt.as_str());

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

## Getting the tracking data

The call to StdTracker::results() returns the tracking data.

# Testing the parser

The test module has several functions to run a test for one parser 
function and to evaluate the result.

str_parse() runs the parser and returns a Test struct with a variety of
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

Tracks the call to the subparser. Calls Track.enter() and 
Track.ok()/Track.err() before and after the subparser.

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

This trait is kind of a undo of parsing. It's called as a method of 
the input value and takes two parsed fragments. It then returns a new
fragment that covers both input fragments.

nom has consumed() and recognize() for this, which work fine too.

# SpanFragment

Provides the nom_locate functions fragment() via a trait.  
This way it's available for &str too. This is essential when switching
between LocatedSpan and plain &str.

# SourceStr and SourceBytes

They can be created with Track.source_str()/Track.source_bytes(). 

They can map any parsed fragment to line/column index, and they can 
extract the surrounding text lines. 

### ParserError vs TokenizerError

ParserError is double the size of TokenizerError due to a Vec with all
the extra data.

Personally I use TokenizerError for the lower level parsers and switch 
to ParserError at the point where I need the extra information.

With err_into() this is not too annoying.

