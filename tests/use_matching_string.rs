use nom_parse_macros::parse_match;
use nom_parse_trait::ParseFromExt;

#[parse_match("({},{},{})")]
#[derive(Debug, PartialEq)]
struct Vector3 {
    x: u32,
    y: u32,
    z: u32,
}

#[test]
fn test_vector() {
    let input = "(1,3,4)";
    let vector = Vector3::parse_complete(input);
    assert_eq!(Ok(Vector3 { x: 1, y: 3, z: 4 }), vector);
}
