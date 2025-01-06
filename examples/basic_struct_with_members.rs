//! This example shows basic usage of parsing a normal struct using nom functions.
//! The expected outcome of the nom expression should be a tuple with the same
//! amount of parameters as the struct has.

use nom::IResult;
use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFrom;

#[parse_from(separated_pair({}, tuple(space0, ",", space0), {}))]
#[derive(Debug, PartialEq)]
struct NumberPair {
    x: u32,
    y: u32,
}

fn main() {
    let input = "1 ,  2";
    let pair: IResult<_, _> = NumberPair::parse(input);
    println!("Parsed \"{}\" as {:?}", input, pair.unwrap().1);
}
