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
