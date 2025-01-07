use crate::fields::Fields;
use crate::nom_packages::generate_match_expression;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{Error, LitStr, Result};

pub fn parse_string_match(fields: &Fields, literal: LitStr) -> Result<Vec<TokenStream>> {
    let value = literal.value();
    let parts: Vec<_> = value
        .split("{}")
        .map(|part| {
            let expr = generate_match_expression(part.as_bytes(), literal.span());
            quote_spanned! { literal.span() => let (input, _) = #expr.parse(input)?; }
        })
        .collect();

    if parts.len() != fields.fields.len() + 1 {
        return Err(Error::new(
            literal.span(),
            "Number of {} parts in the literal is not equal to the number of fields",
        ));
    }

    let mut result = vec![parts[0].clone()];
    for index in 0..fields.fields.len() {
        if let Some(expr) = fields.fields[index].generate_expression() {
            result.push(expr);
        }
        result.push(parts[index + 1].clone());
    }

    for field in &fields.fields {
        if let Some(expr) = field.generate_derived_expression(fields) {
            result.push(expr);
        }
    }

    Ok(result)
}
