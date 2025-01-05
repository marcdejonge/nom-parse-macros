use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFromExt;

#[parse_from(separated_pair({}, tuple(space0, ",", space0), {}))]
#[derive(Debug, PartialEq)]
struct SomeStruct {
    x: u32,
    y: u32,
    #[derived(x + y)]
    sum: u32,
}

#[test]
fn test_pair() {
    let input = "1 ,  2";
    let result = SomeStruct::parse_complete(input);
    assert_eq!(Ok(SomeStruct { x: 1, y: 2, sum: 3 }), result);
}
