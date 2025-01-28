use nom::error::Error;
use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFrom;

#[parse_from]
#[derive(Debug, PartialEq)]
enum Command {
    #[format("a")]
    A,
    #[format("b")]
    B,
    #[format("c")]
    C,
    #[format(fail::<_, (), _>())]
    Unknown,
}

#[parse_from(separated_pair(separated_list1(",", {}), " => ", {}))]
#[derive(Debug, PartialEq)]
struct CommaSeparated<T>(Vec<T>, T);

#[test]
fn test_generic_u32() {
    assert_eq!(
        Ok::<_, nom::Err<Error<_>>>(("", CommaSeparated(vec![1, 2, 3], 4))),
        CommaSeparated::<u32>::parse("1,2,3 => 4")
    );
}

#[test]
fn test_generic_i64() {
    assert_eq!(
        Ok::<_, nom::Err<Error<_>>>(("", CommaSeparated(vec![1, -2, 3], 9))),
        CommaSeparated::<i64>::parse("1,-2,3 => 9")
    )
}

#[test]
fn test_generic_commands() {
    assert_eq!(
        Ok::<_, nom::Err<Error<_>>>(("", CommaSeparated(vec![Command::A, Command::B], Command::C))),
        CommaSeparated::<Command>::parse("a,b => c")
    )
}
