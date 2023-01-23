
# KParse

Outline for a handwritten parser.

_The code can be found as example1.rs._

_tests contains two test runs test_parser4 and test_parser_cmds that
 contain a parser impl._

1. Define your function/error codes. They are used interchangeably.
   Add a variant for nom::error::Error to work with nom.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ICode {
    ICNomError,

    ICTerminalA,
    ICInt
}
```

2. Mark it as trait Code. This needs Copy + Display + Debug + Eq 

```rust
impl Code for ICode {
   const NOM_ERROR: Self = Self::ICNomError;
}

impl Display for ICode {
   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
      let name = match self {
         ICode::ICNomError => "NomError",
         ICode::ICTerminalA => "TerminalA",
      };
      write!(f, "{}", name)
   }
}
```

3. Add type aliases for 
   * Span
   * Result type of the parser fn
   * Result type for nom parser fn.
   * The ParserError itself.

There is a () type parameter for extra userdata in the ParserError.

```rust
pub type Span<'s> = kparse::Span<'s, ICode>;
pub type IParserResult<'s, O> = ParserResult<'s, O, ICode, ()>;
pub type INomResult<'s> = ParserNomResult<'s, ICode, ()>;
pub type IParserError<'s> = ParserError<'s, ICode, ()>;
```

4. Define the AST structs. There are no constraints from IParse.

```rust
pub struct TerminalA<'s> {
   pub term: String,
   pub span: Span<'s>,
}
```

5. Create the nom parsers for your terminals. 

```rust
pub fn nom_parse_a(i: Span<'_>) -> INomResult<'_> {
   tag("A")(i)
}
```

6. Create a transform fn for each nom fn. This translates the nom errors to our parsers errors.
This is also a good point for conversions from string.

```rust
pub fn parse_a(rest: Span<'_>) -> IParserResult<'_, TerminalA> {
   match nom_parse_a(rest) {
      Ok((rest, token)) => Ok((
         rest,
         TerminalA {
            term: token.to_string(),
            span: token,
         },
      )),
      Err(nom::Err::Error(e)) if e.is_error_kind(nom::error::ErrorKind::Tag) => {
         Err(nom::Err::Error(e.with_code(ICTerminalA)))
      }
      Err(e) => Err(e.into()),
   }
}
```

4. Implement the parser with regular fn's.

For the parser tracking the Context must be used.
The first thing should be the call to Context.enter(), and each exit point 
must be covered.

Context.ok() and Context.err() take care of the simple case.

```rust
fn parse_terminal_a(rest: Span<'_>) -> IParserResult<'_, TerminalA<'_>> {
   Context.enter(ICTerminalA, &rest);

   let (rest, token) = match parse_a(rest) {
      Ok((rest, token)) => (rest, token),
      Err(e) => return Context.err(e),
   };

   Context.ok(rest, token.span, token)
}
```

There is TrackParserError::track() to make everything a bit easier. 
This tracks the error case with the Context and let's you work with ?.

There is another one track_as() that let's you change the error-code in 
passing.

```rust
fn parse_terminal_a2(rest: Span<'_>) -> IParserResult<'_, TerminalA<'_>> {
   Context.enter(ICTerminalA, &rest);

   let (rest, token) = parse_a(rest).track()?;

   Context.ok(rest, token.span, token)
}
```

5. To call the parser use any impl of ParseContext. 

The tracking context is named TrackingContext.
The const type argument states whether the actual tracking will be done or not.

```rust
fn run_parser() -> IParserResult<'static, TerminalA<'static>> {
   let ctx: TrackingContext<'_, &str, ICode> = TrackingContext::new(true);
   let span = ctx.span("A");
   let _r = parse_terminal_a(span);
}
```

The other interesting one is NoContext which does almost nothing.

```rust
fn run_parser() -> IParserResult<'static, TerminalA<'static>> {
   let span = NoContext.span("A");
   let _r = parse_terminal_a(span);
}
```

The current context is added to the LocatedSpan and is propagated this way.
This is the reason why the span has to created this way.

6. Testing

Use kparse::test::Test. 

It has functions to test a single parser and check the results.

* ok() - Compare the Ok() result with a test fn().
* err() - Check for a specific error-code.
* expect() - Check for an additional error-code stored as expected result.
* rest() - Compare the rest after parse.
* okok() - Result should be any Ok() value.
* errerr() - Result should be any Err() value.
* fail() - Always fail.
* nom_err() - Result contains this nom error-code.
* 

The last fn is q() which collects the checks and creates a report.
There are some reports available
* Dump - Output the error/success. Doesn't panic.
* CheckDump - Output the error/success. Panics if any of the test-fn failed.
* Trace - Output the complete trace. Doesn't panic.
* CheckTrace - Output the complete trace. Panics if any of the test-fn failed.
* Timing - Output only the timings. 

```rust
const R: Trace = Trace;

#[test]
pub fn test_terminal_a() {
   track_parse(&mut None, "A", parse_terminal_a).okok().q(R);
   track_parse(&mut None, "AA", parse_terminal_a).errerr().q(R);
}
 ```

The output of a trace looks like this (this is from [examples/parser4.rs])

```txt
when parsing "Content-Type: text/x-zim-wiki\r\nWiki-Form" in 9.4558ms =>
trace
  Anbauplan: enter with "Content-Type: text/x"
    Plan: enter with "====== Plan 2022 ==="
    Plan: ok -> [ "====== Plan 2022 ===", "\r\n\r\n===== Monat Janu" ]
    KdNr: enter with "===== Monat January "
    KdNr: err NomError expects  for span 82 "===== Monat January " 
    Monat: enter with "===== Monat January "
    Monat: ok -> [ "===== Monat January ", "\r\n\r\n==== Woche 03.01" ]
    Woche: enter with "==== Woche 03.01.202"
    Woche: ok -> [ "==== Woche 03.01.202", "\r\n\r\n\r\n=> Überwintern" ]
    Aktion: enter with "=> Überwintern\r\n    "
    Aktion: ok -> [ "=> Überwintern", "\r\n        @ W2 Karot" ]
    Parzelle: enter with "@ W2 Karotten (+12w)"
      Wochen: enter with "+12w)\r\nKarotten\r\n   "
      Wochen: err Nummer expects NomError:"+12w)\r\nKarotten\r\n   " for span 191 "+12w)\r\nKarotten\r\n   " 
      Plus_Wochen: enter with "+12w)\r\nKarotten\r\n   "
      Plus_Wochen: ok -> [ "12w", ")\r\nKarotten\r\n       " ]
    Parzelle: ok -> [ "@ W2 Karotten (+12w)", "\r\nKarotten\r\n        " ]
    Kultur: enter with "Karotten\r\n        @ "
    ...
```

# Appendix A

## Note 1

There is WithSpan that can be implemented to import external errors. It
should return a nom::Err wrapped ParserError. 

```rust
impl<'s, C: Code, Y: Copy> WithSpan<'s, C, nom::Err<ParserError<'s, C, Y>>>
for std::num::ParseIntError
{
   fn with_span(self, code: C, span: Span<'s, C>) -> nom::Err<ParserError<'s, C, Y>> {
       nom::Err::Failure(ParserError::new(code, span))
   }
}
```

And to use it there is the transform() fn that takes a regular parser,
applies the closure to the result and converts the error via with_span().

```rust
fn parse_terminal_c(rest: Span<'_>) -> IParserResult<'_, TerminalC<'_>> {
   Context.enter(ICTerminalC, &rest);

   let (rest, (tok, v)) =
           consumed(transform(nom_parse_c, |v| (*v).parse::<u32>(), ICInteger))(rest).track()?;

   Context.ok(rest, tok, TerminalC { term: v, span: tok })
}
```

## Note 2

The trait kparse::TrackParseResult make the composition of parser 
easier. It provides a track() function for the parser result, that notes a 
potential error and returns the result. This in turn can be used for the ? 
operator. 

It has a second method track_as() that allows to change the error code.

```rust
fn parse_non_terminal1(rest: Span<'_>) -> IParserResult<'_, NonTerminal1<'_>> {
   Context.enter(ICNonTerminal1, &input);

   let (rest, a) = parse_terminal_a(input).track()?;
   let (rest, b) = parse_terminal_b(rest).track()?;

   let span = input.span_union(&a.span, &b.span);

   Context.ok(rest, span, NonTerminal1 { a, b, span })
}
```

Or you can use the nom combinators.

```rust
fn parse_non_terminal1_1(rest: Span<'_>) -> IParserResult<'_, NonTerminal1<'_>> {
    Context.enter(ICNonTerminal1, &rest);

    let (rest, (token, (a, b))) =
        consumed(tuple((parse_terminal_a, parse_terminal_b)))(rest).track()?;

   Context.ok(rest, token, NonTerminal1 { a, b, span: token })
}
```

## Note 3

It is often useful to keep the parsed span. With SpanExt::span_union() you
can glue together two fragments of the original span.

Or you can use consumed().

```rust
fn sample() {
   let _span = input.span_union(&a.span, &b.span);
}
```

## Note 4

Handling optional terms is using opt() from nom.

```rust
fn parse_non_terminal_2(rest: Span<'_>) -> IParserResult<'_, NonTerminal2<'_>> {
   Context.enter(ICNonTerminal1, &input);

   let (rest, a) = opt(parse_terminal_a)(input).track()?;
   let (rest, b) = parse_terminal_b(rest).track()?;
   let (rest, c) = parse_terminal_c(rest).track()?;

   let span = if let Some(a) = &a {
      input.span_union(&a.span, &c.span)
   } else {
      c.span
   };

   Context.ok(rest, span, NonTerminal2 { a, b, c, span })
}
```

## Note 5

Some example for a loop. 
Looks solid to use a mut loop-variable but only modify it at the border.

Note ```err.append(e)``` as a way to collect multiple errors.

```rust
fn parse_non_terminal_3(rest: Span<'_>) -> IParserResult<'_, ()> {
   Context.enter(ICNonTerminal3, &rest);

   let mut loop_rest = rest;
   let mut err = None;
   loop {
      let rest2 = loop_rest;
      let (rest2, _a) = opt(parse_terminal_a)(rest2).track()?;
      let (rest2, _b) = match parse_terminal_b(rest2) {
         Ok((rest3, b)) => (rest3, Some(b)),
         Err(e) => {
            err.append(e)?;
            (rest2, None)
         }
      };

      if rest2.is_empty() {
         break;
      }

      // endless loop
      if loop_rest == rest2 {
         return Context.err(ParserError::new(ICNonTerminal3, rest2));
      }

      loop_rest = rest2;
   }

   Context.ok(rest, rest.take(0), ())
}
```

## Note 5

There is SpanLines and SpanBytes that make the diagnostics easier.

Both start with the original buffer and accept a parsed fragment for

* ascii_column(), utf8_column() similar to LocatedSpan.
* current() - Extract the full text line for a fragment.
* forward_from(), backward_from() - Iterate surrounding text lines.
* get_lines_around() - retrieve context lines for a fragment.

# Appendix B

## Noteworthy 1

These are the conversion traits.
* WithSpan
* WithCode
* std::convert::From

The std::convert::From is implemented for nom types to do a 
default conversion into ParserError.

WithCode and WithSpan for conversion of external errors and adding 
context.

## Noteworthy 2

There are two parser combinators

* error_code

Wraps another parser and sets the errorcode on error.

* transform

Wraps a parser and a conversion fn. Transforms the conversion error
via with_span and every parser error via with_code.

## Noteworthy 3

ParserErrors can be combined via the trait AppendParserError.
It exists for ParserError, Option<ParserError> and can take ParserError 
itself or wrapped in a nom::Err.
