#![allow(dead_code)]

use kparse::error::Hints;
use kparse::tracking_context::Track;
use kparse::{
    Code, Context, CtxParserNomResult, CtxParserResult, CtxSpan, DynContext, NoContext,
    ParserError, TrackingContext,
};
use nom_locate::LocatedSpan;
use std::fmt::{Debug, Display, Formatter};
use std::mem::size_of;

enum ZCode {
    ZOne,
    ZTwo,
}

impl Copy for ZCode {}

impl Clone for ZCode {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Display for ZCode {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Debug for ZCode {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Eq for ZCode {}

impl PartialEq<Self> for ZCode {
    fn eq(&self, _: &Self) -> bool {
        todo!()
    }
}

impl Code for ZCode {
    const NOM_ERROR: Self = Self::ZOne;
}

#[derive(Debug, Clone, Copy)]
struct Nummer<'s> {
    nummer: u32,
    span: CtxSpan<'s, &'s str, ZCode>,
}

#[test]
fn test_size2() {
    dbg!(size_of::<usize>());
    dbg!(size_of::<u32>());
    dbg!(size_of::<&str>());

    dbg!(size_of::<ZCode>());
    dbg!(size_of::<CtxSpan<'_, &str, ZCode>>());

    dbg!(size_of::<ParserError<ZCode, &str, ()>>());
    dbg!(size_of::<Vec<Hints<ZCode, &str, ()>>>());

    dbg!(size_of::<Context>());
    dbg!(size_of::<CtxParserNomResult<&str, ZCode, ()>>());
    dbg!(size_of::<CtxParserResult<(), &str, ZCode, ()>>());
    dbg!(size_of::<CtxParserResult<Nummer<'_>, &str, ZCode, ()>>());
    dbg!(size_of::<CtxSpan<&str, ZCode>>());
    dbg!(size_of::<LocatedSpan<&str>>());

    dbg!(size_of::<DynContext<&str, ZCode>>());

    dbg!(size_of::<NoContext>());
    dbg!(size_of::<TrackingContext<&str, ZCode>>());
    dbg!(size_of::<Track<&str, ZCode>>());
}
