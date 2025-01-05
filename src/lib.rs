extern crate proc_macro;
mod fields;
mod nom_packages;
mod string_matching;

use proc_macro::TokenStream;

use crate::fields::{parse_fields, Field};
use crate::nom_packages::update_nom_expression;
use crate::string_matching::parse_string_match;
use quote::quote;
use syn::{parse_macro_input, Expr, ItemStruct, LitStr};

#[proc_macro_attribute]
pub fn parse_from(attrs: TokenStream, object: TokenStream) -> TokenStream {
    let mut expression = parse_macro_input! { attrs as Expr };
    if let Err(e) = update_nom_expression(&mut expression) {
        return e.to_compile_error().into();
    }

    let mut object = parse_macro_input!(object as ItemStruct);
    let fields = match parse_fields(&mut object.fields) {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error().into(),
    };
    let name = object.ident.clone();

    let expression_names: Vec<_> = fields
        .iter()
        .filter(|field| !matches!(field, Field::Derived { .. }))
        .map(|field| field.get_name())
        .collect();
    let derived_expressions: Vec<_> = fields
        .iter()
        .filter_map(|field| field.generate_derived_expression())
        .collect();
    let all_names: Vec<_> = fields.iter().map(|field| field.get_name()).collect();

    let tokens = quote! {
        #object

        impl nom_parse_trait::ParseFrom<&str> for #name {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::*;

                let mut input = input;
                let (input, (#(#expression_names),*)) = #expression.parse(input)?;
                #(#derived_expressions)*
                Ok((input, Self { #(#all_names),* }))
            }
        }
    };

    tokens.into()
}

#[proc_macro_attribute]
pub fn parse_match(attrs: TokenStream, object: TokenStream) -> TokenStream {
    let literal = parse_macro_input! { attrs as LitStr };

    let mut object = parse_macro_input!(object as ItemStruct);
    let fields = match parse_fields(&mut object.fields) {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error().into(),
    };
    let name = object.ident.clone();

    match parse_string_match(&fields, literal) {
        Ok(parts) => {
            let names: Vec<_> = fields.iter().map(|field| field.get_name()).collect();
            let tokens = quote! {
                #object

                impl nom_parse_trait::ParseFrom<&str> for #name {
                    fn parse(input: &str) -> nom::IResult<&str, Self> {
                        use nom::*;

                        let mut input = input;
                        #(#parts)*
                        Ok((input, Self { #(#names),* }))
                    }
                }
            };

            tokens.into()
        }
        Err(e) => e.to_compile_error().into(),
    }
}
