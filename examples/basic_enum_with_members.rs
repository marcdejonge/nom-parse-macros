//! This example shows basic usage of parsing a normal struct using nom functions.
//! The expected outcome of the nom expression should be a tuple with the same
//! amount of parameters as the struct has.

use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFrom;

#[parse_from]
#[derive(Debug, PartialEq)]
enum SomeConfig {
    Number(u32),
    #[format(delimited('(', separated_list0(",", ()), ')'))]
    Numbers(Vec<u32>),
}

fn main() {
    let input = "32";
    let number = SomeConfig::parse(input).unwrap().1;
    println!("Parsed \"{}\" as {:?}", input, number);

    let input = "(32,34,46)";
    let numbers = SomeConfig::parse(input).unwrap().1;
    println!("Parsed \"{}\" as {:?}", input, numbers);
}
