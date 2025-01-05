extern crate proc_macro;
mod fields;
mod nom_packages;
mod string_matching;

use proc_macro::TokenStream;

use crate::fields::parse_fields;
use crate::nom_packages::update_nom_expression;
use quote::quote;
use syn::{parse_macro_input, Expr, ItemStruct, LitStr};
use crate::string_matching::parse_string_match;

#[proc_macro_attribute]
pub fn parse_from(attrs: TokenStream, object: TokenStream) -> TokenStream {
    let mut expression = parse_macro_input! { attrs as Expr };
    if let Err(e) = update_nom_expression(&mut expression) {
        return e.to_compile_error().into();
    }

    let object = parse_macro_input!(object as ItemStruct);
    let fields = parse_fields(&object.fields);
    let name = object.ident.clone();

    let names: Vec<_> = fields.iter().map(|field| field.get_name()).collect();

    let tokens = quote! {
        #object

        impl nom_parse_trait::ParseFrom<&str> for #name {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::*;

                let mut input = input;
                let (input, (#(#names),*)) = #expression.parse(input)?;
                Ok((input, Self { #(#names),* }))
            }
        }
    };

    tokens.into()
}

#[proc_macro_attribute]
pub fn parse_match(attrs: TokenStream, object: TokenStream) -> TokenStream {
    let literal = parse_macro_input! { attrs as LitStr };

    let object = parse_macro_input!(object as ItemStruct);
    let fields = parse_fields(&object.fields);
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
