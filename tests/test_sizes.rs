#![allow(dead_code)]

use kparse::error::Hints;
use kparse::tracker::{
    DynTracker, StdTracker, Track, TrackParserResult, TrackParserResultSpan, TrackSpan,
};
use kparse::{Code, Context, ParserError};
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
    span: TrackSpan<'s, ZCode, &'s str>,
}

#[test]
fn test_size2() {
    dbg!(size_of::<usize>());
    dbg!(size_of::<u32>());
    dbg!(size_of::<&str>());

    dbg!(size_of::<ZCode>());
    dbg!(size_of::<TrackSpan<'_, ZCode, &str>>());

    dbg!(size_of::<ParserError<ZCode, &str, ()>>());
    dbg!(size_of::<Vec<Hints<ZCode, &str, ()>>>());

    dbg!(size_of::<Context>());
    dbg!(size_of::<TrackParserResultSpan<ZCode, &str, ()>>());
    dbg!(size_of::<TrackParserResult<ZCode, &str, (), ()>>());
    dbg!(size_of::<TrackParserResult<ZCode, &str, (), Nummer<'_>>>());
    dbg!(size_of::<TrackSpan<ZCode, &str>>());
    dbg!(size_of::<LocatedSpan<&str>>());

    dbg!(size_of::<DynTracker<ZCode, &str>>());

    dbg!(size_of::<StdTracker<ZCode, &str>>());
    dbg!(size_of::<Track<ZCode, &str>>());
}
