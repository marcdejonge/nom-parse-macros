use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFrom;

#[parse_from("({},{},{})")]
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
