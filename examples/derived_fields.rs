use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFrom;

/// This example shows how you can use the derived attribute.
/// A derived field is not actually parsed, but derived from all of the other
/// fields that are parsed.

#[parse_from(separated_pair({}, tuple(space0, ",", space0), {}))]
#[derive(Debug, PartialEq)]
struct NumberPair {
    x: u32,
    y: u32,
    #[derived(x + y)]
    sum: u32,
}

fn main() {
    let input = "1 ,  2";
    let pair = NumberPair::parse(input).unwrap();
    println!("Parsed \"{}\" as {:?}", input, pair.1);
}

