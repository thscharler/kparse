#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::hint::black_box;
use std::time::Instant;

#[derive(Parser)]
#[grammar = "tests/anbauplan4.pest"]
struct AnbauplanParser;

#[test]
pub fn timing() {
    let s = include_str!("2022_Anbauplan.txt");
    let now = Instant::now();
    let cnt = 100;
    for _i in 0..cnt {
        let _r = black_box(AnbauplanParser::parse(Rule::AnbauPlan, s));
    }
    let duration = now.elapsed();
    println!("{:?}", duration / cnt);
}
