use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFromExt;

#[parse_from(separated_pair({}, tuple(space0, ",", space0), {}))]
#[derive(Debug, PartialEq)]
struct NumberPair {
    x: u32,
    y: u32,
}

#[test]
fn test_parsing_number_pair() {
    let input = "1 ,  2";
    let pair = NumberPair::parse_complete(input);
    assert_eq!(Ok(NumberPair { x: 1, y: 2 }), pair);
}
