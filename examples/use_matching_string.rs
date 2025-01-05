//! This example shows how you can use a string matching pattern for parsing.
//! The {} gets replaced with a parser for the corresponding field. The rest of
//! the characters are matched verbatim.

use nom_parse_macros::parse_match;
use nom_parse_trait::ParseFrom;

#[parse_match("({},{},{})")]
#[derive(Debug, PartialEq)]
struct Vector3 {
    x: u32,
    y: u32,
    z: u32,
}

fn main() {
    let input = "(1,3,4)";
    let pair = Vector3::parse(input).unwrap();
    println!("Parsed \"{}\" as {:?}", input, pair.1);
}
