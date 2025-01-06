use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFromExt;

#[parse_from]
#[derive(Debug, PartialEq)]
enum TestStruct {
    Number(u32),
    #[format(delimited('(', separated_list0(",", ()), ')'))]
    Numbers(Vec<u32>, #[derived(field_0.iter().sum())] u32),
    #[format(delimited('"', map(alpha0, |s: &str| s.to_string()), '"'))]
    String {
        value: String,
        #[derived(value.len())]
        len: usize,
    },
}

#[test]
fn test_number() {
    assert_eq!(Ok(TestStruct::Number(32)), TestStruct::parse_complete("32"));
}

#[test]
fn test_numbers() {
    assert_eq!(
        Ok(TestStruct::Numbers(vec![32, 34, 46], 112)),
        TestStruct::parse_complete("(32,34,46)")
    );
}

#[test]
fn test_string() {
    assert_eq!(
        Ok(TestStruct::String {
            value: "dummy".to_string(),
            len: 5,
        }),
        TestStruct::parse_complete("\"dummy\"")
    )
}
