use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFromExt;

#[parse_from]
#[derive(Debug, PartialEq)]
enum SomeConfig {
    Number(u32),
    #[format(delimited('(', separated_list0(",", ()), ')'))]
    Numbers(Vec<u32>),
    #[format(delimited('"', map(alpha0, |s: &str| s.to_string()), '"'))]
    String {
        value: String,
    },
}

#[test]
fn test_number() {
    assert_eq!(Ok(SomeConfig::Number(32)), SomeConfig::parse_complete("32"));
}

#[test]
fn test_numbers() {
    assert_eq!(
        Ok(SomeConfig::Numbers(vec![32, 34, 46])),
        SomeConfig::parse_complete("(32,34,46)")
    );
}

#[test]
fn test_string() {
    assert_eq!(
        Ok(SomeConfig::String {
            value: "dummy".to_string()
        }),
        SomeConfig::parse_complete("\"dummy\"")
    )
}
