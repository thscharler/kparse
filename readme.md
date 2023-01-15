
# IParse

Outline for a handwritten parser.

_The code can be found as example1.rs._

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
      Err(nom::Err::Error(e)) if e.is_kind(nom::error::ErrorKind::Tag) => {
         Err(e.into_code(ICTerminalA))
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

There is another one track_as() that let's you change the errorcode in 
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
   let ctx: TrackingContext<'_, ICode, true> = TrackingContext::new("A");
   let span = ctx.span();
   
   let mut trace: CTracer<_, true> = CTracer::new();
   ParseTerminalA::parse(&mut trace, Span::new("A"))
}
```

The other interesting one is NoContext which does almost nothing.

```rust
fn run_parser() -> IParserResult<'static, TerminalA<'static>> {
   let span = NoContext.span("A");
   parse_terminal_a(span)
}
```

The current context is added to the LocatedSpan and is propagated this way.
This is the reason why the span has to created this way.

6. Testing

Use kparse::test::Test. It has functions to test a single parser and 
check on the results.

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

   let (rest, tok) = transform(nom_parse_c, |v| (*tok).parse::<u32>(), ICInteger)(i).track()?;

   Context.ok(rest, tok.span, tok)
}
```

## Note 2

The trait iparse::tracer::TrackParseResult make the composition of parser 
easier. It provides a track() function for the parser result, that notes a 
potential error and returns the result. This in turn can be used for the ? 
operator. 

It has a second method track_as() that allows to change the error code.

```rust
fn parse_non_terminal1(rest: Span<'_>) -> IParserResult<'_, NonTerminal1<'_>> {
   Context.enter(ICNonTerminal1, &rest);

   let (rest, a) = parse_terminal_a(rest).track()?;
   let (rest, b) = parse_terminal_b(rest).track()?;

   let span = unsafe { Context.span_union(&a.span, &b.span) };

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

It is good to have the full span for non-terminals in the parser. There is no
way to glue the spans together via nom, so there is span_union(). 
It is unsafe as is, as it relies on the fact that both spans are 
part of the original span and are in order.

Or you can use consumed as above. Maybe do that. This is a relic.

```rust
fn sample() {
   let span = unsafe { Context.span_union(&a.span, &b.span) };
}
```

## Note 4

Handling optional terms is using opt() from nom.

```rust
fn parse_non_terminal_2(rest: Span<'_>) -> IParserResult<'_, NonTerminal2<'_>> {
   Context.enter(ICNonTerminal1, &rest);

   let (rest, a) = opt(parse_terminal_a)(rest).track()?;
   let (rest, b) = parse_terminal_b(rest).track()?;
   let (rest, c) = parse_terminal_c(rest).track()?;

   let span = if let Some(a) = &a {
      unsafe { Context.span_union(&a.span, &c.span) }
   } else {
      c.span
   };

   Context.ok(rest, span, NonTerminal2 { a, b, c, span })
}
```

## Note 5

Some example for a loop. 
Looks solid to use a mut loop-variable but only modify it at the border.

Note ```err.add(e)``` as a way to collect multiple errors.

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
            err.add(e)?;
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

There is the trait DataFrames and it's implementations ByteFrames, StrLines
and SpanLines.

They work with the original buffer and can give back

* get_lines_around()
* current() // line
* start() and end() lines of a fragment.
* iterate forward_from() and backward_from()
* calculate the offset()

These are all unsafe and UB if you call them with a fragment not derived
from the original buffer. 


# Appendix B

## Noteworthy 1

These are the conversion traits.
* WithSpan
* WithCode
* std::convert::From

The std::convert::From is implemented for nom types to do a 
default conversion into ParserError.

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
