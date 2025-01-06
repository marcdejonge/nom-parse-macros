use nom::error::Error;
use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFromExt;

#[parse_from(())]
#[derive(Debug, PartialEq)]
struct Test(i32);

#[test]
pub fn from_str() {
    assert_eq!(Ok::<_, Error<_>>(Test(32)), Test::parse_complete("32"));
}

#[test]
pub fn from_bytes() {
    assert_eq!(
        Ok::<_, Error<_>>(Test(32)),
        Test::parse_complete(b"32".as_ref())
    );
}
