use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFrom;

#[parse_from(split = space1)]
#[derive(Debug, PartialEq)]
struct NumberPair {
    x: u32,
    y: u32,
}

fn main() {
    let input = "1 2";
    let pair = NumberPair::parse(input).unwrap();
    println!("Parsed \"{}\" as {:?}", input, pair.1);
}
