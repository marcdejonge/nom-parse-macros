use nom::error::*;
use nom_parse_macros::parse_match;
use nom_parse_trait::ParseFromExt;

#[test]
fn test_vector() {
    #[parse_match("({},{},{})")]
    #[derive(Debug, PartialEq)]
    struct Test {
        x: u32,
        y: u32,
        z: u32,
    }

    assert_eq!(
        Ok::<_, Error<_>>(Test { x: 1, y: 3, z: 4 }),
        Test::parse_complete("(1,3,4)")
    );

    assert_eq!(
        Err(Error::from_error_kind("_", ErrorKind::Tag)),
        Test::parse_complete("(1,2_")
    );

    assert_eq!(
        Err(Error::from_error_kind(")", ErrorKind::Tag)),
        Test::parse_complete("(1,2)")
    );
}
