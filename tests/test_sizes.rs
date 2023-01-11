#![allow(dead_code)]

use kparse::{Code, Hints, HoldContext, ParserError, ParserResult, Span};
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
    span: Span<'s, ZCode>,
}

#[test]
fn test_sizes() {
    dbg!(size_of::<usize>());
    dbg!(size_of::<u32>());
    dbg!(size_of::<&str>());
    dbg!(size_of::<HoldContext<ZCode>>());

    // offset: usize,
    // line: u32,
    // fragment: T,
    // pub extra: X,

    dbg!(size_of::<Span<'_, ZCode>>());
    dbg!(size_of::<ZCode>());
    dbg!(size_of::<Vec<Hints<'_, ZCode, ()>>>());
    dbg!(size_of::<ParserError<ZCode, ()>>());
    dbg!(size_of::<ParserResult<Nummer<'_>, ZCode, ()>>());
}
