# 2.0

## New

* Add SpanStr for error analysis. This allows calculating the row/column
from a plain &str. This finally got my test parser on par with a PEST parser
for the same grammar.

* Got some inspiration from nom_supreme and added the KParser trait 
to enable postfix transformation of parsers. This works nicely with err_into(),
with_code() etc. Couldn't make sense of all their choices so a few were left 
away.

* Add pchar() parser as a replacement for nom::char(). Easier to use name
and returns the input type instead of char.
* Add separated_list_trailing0() and separated_list_trailing1() similar
to separated_list0() and separated_list1(), but they allow a trailing 
separator.

## Breaking

* Add a second, leaner error type TokenizerError. 

* FindTracker renamed to Tracking and TrackError to ResultTracking.
* FindTracker accepts any error type with a Debug.
* FindTracker::err() and ok() are gone. This is now done in Context.
  This reduces the trait to empty functions for the no tracking case.
* FindTracker::exit_err() takes a &E instead of a string.
  This removes a .to_string() if no tracking is active.
* TrackError stops converting error types.
* Trait Tracker reduced to a single function. 

* Trait KParseError for acceptable error types. This replaces the
  requirement for ParserError in most places.
* Merged WithCode into KParseError.

* Remove Y type parameter from ParserError and replaced the functionality
  with a Box<dyn Any>. 

* Trait ErrWrapped allows accepting a nom::Err<E> and a plan E for Context.

* Remove traits WithSpan and ResultWithSpan.

* Rename error_code() to with_code().
* Rename transform() to map_res().

* Change the Copy requirement to Clone.

## Features

* Add a feature track_nom. If it is not active, no details for nom errors
  are collected. This helps to avoid creating a hint-vec for ParserError.
  This speeds up the case when parsing fails just to indicate a wrong branch.

## Other

* Add Context::ok_section() and Context::err_section(). Can be used to
  add compartments within one parser function.
* Add inline to all Context functions.

# 1.1

Not everything was as good as I thought.

## Major

* Found a completely different formulation for Context.
  FindTracker now acts directly on the input type. This way there can be only
  one
  impl for Context. And FindTracker now can be used as a constraint for the
  input type.
  This way it was possible to change TrackError so it doesn't have to bend over
  backwards to achieve it's job. I can have one blanket impl for TrackError now.
  Surprisingly the perceived api didn't change with this.

* Span extension traits have been renamed: SpanExt->SpanUnion, LocatedSpanExt->
  SpanLocation and Fragment->SpanFragment. All of them now work seamlessly with
  cfg(debug_assertions) too.

* Parser combinator error_code() changed it's signature. The generic error type
  changed to ParserError. This helps with type inference.
* Parser combinator transform() now only works with ParserError too.

* FindTracker::exit_err() and Tracker::exit_err() now take a String instead of
  a &dyn Error. It has always been immediately converted to a String anyway.

* ParserError::new_suggest() was an oversight. Removed now.
* ParserError::append_expected() and ParserError::append_suggested() change to
  Iterator.
* ParserError::with_cause() uses a type parameter instead of Box<dyn Error>.

* Test had some quirks when working with cfg(debug_assertions). Switching
  between TrackSpan<&str> and &str works nicely now.
* Trace and CheckTrace now work with cfg(debug_assertions) too.

## New

* New combinator transform_p() for when the transformation directly returns a
  parser error and no conversion is needed. This case doesn't fit well with
  transform().

## Behaviour

* Context.err() takes a 'parsed' span as parameter. Often this is difficult
  to achieve. So now if you give it the original input it tries to make sense
  from it and shows the parsed part as input.offset..remainder.offset.

## Minor

* Type parameter Y needs a Debug constraint everywhere.
* Error::is_expected() checks the main error too.
* WithSpan for ().
