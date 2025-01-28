use nom::error::Error;
use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFromExt;

#[test]
fn test_single_named_field() {
    #[parse_from(())]
    #[derive(Debug, PartialEq)]
    struct Test {
        field: u32,
    }

    let input = "32";
    let expected = Test { field: 32 };

    assert_eq!(Ok::<_, Error<_>>(expected), Test::parse_complete(input));
}

#[test]
fn test_multiple_named_fields() {
    #[parse_from(separated_pair({}, (space0, ",", space0), {}))]
    #[derive(Debug, PartialEq)]
    struct Test {
        a: u32,
        b: i64,
    }

    let input = "32 ,  45";
    let expected = Test { a: 32, b: 45 };

    assert_eq!(Ok::<_, Error<_>>(expected), Test::parse_complete(input));
}

#[test]
fn test_derived_named_field() {
    #[parse_from(())]
    #[derive(Debug, PartialEq)]
    struct Test {
        a: u32,
        #[derived(a as i64 * 2)]
        b: i64,
    }

    let input = "32";
    let expected = Test { a: 32, b: 64 };

    assert_eq!(Ok::<_, Error<_>>(expected), Test::parse_complete(input));
}

#[test]
fn test_singe_unnamed_field() {
    #[parse_from(())]
    #[derive(Debug, PartialEq)]
    struct Test(u32);

    let input = "32";
    let expected = Test(32);

    assert_eq!(Ok::<_, Error<_>>(expected), Test::parse_complete(input));
}

#[test]
fn test_multiple_unnamed_fields() {
    #[parse_from(separated_pair({}, (space0, ",", space0), {}))]
    #[derive(Debug, PartialEq)]
    struct Test(u32, i64);

    let input = "32 ,  45";
    let expected = Test(32, 45);

    assert_eq!(Ok::<_, Error<_>>(expected), Test::parse_complete(input));
}

#[test]
fn test_derived_unnamed_field() {
    #[parse_from(())]
    #[derive(Debug, PartialEq)]
    struct Test(u32, #[derived(field_0 as i64 * 2)] i64);

    let input = "32";
    let expected = Test(32, 64);

    assert_eq!(Ok::<_, Error<_>>(expected), Test::parse_complete(input));
}
