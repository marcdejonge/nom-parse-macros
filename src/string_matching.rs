use crate::fields::Field;
use crate::nom_packages::generate_match_expression;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::{Error, LitStr, Result};

pub fn parse_string_match(fields: &[Field], literal: LitStr) -> Result<Vec<TokenStream>> {
    let value = literal.value();
    let parts: Vec<_> = value
        .split("{}")
        .map(|part| {
            let expr = generate_match_expression(part.as_bytes(), literal.span());
            quote_spanned! { literal.span() => let (input, _) = #expr.parse(input)?; }
        })
        .collect();

    if parts.len() != fields.len() + 1 {
        return Err(Error::new(
            literal.span(),
            "Number of {} parts in the literal is not equal to the number of fields",
        ));
    }

    Ok(Itertools::interleave(
        parts.into_iter(),
        fields
            .iter()
            .map(|field| field.generate_expression().into_token_stream()),
    )
    .collect())
}
