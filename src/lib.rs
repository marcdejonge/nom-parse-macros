mod settings;
mod fields;
mod nom_packages;

extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;
use syn::ItemStruct;
use crate::fields::parse_fields;
use crate::settings::ParseSettings;

#[proc_macro_attribute]
pub fn parse_from(attrs: TokenStream, object: TokenStream) -> TokenStream {
    let parse_settings = match ParseSettings::from(attrs.into()) {
        Ok(settings) => settings,
        Err(e) => return e.to_compile_error().into(),
    };

    let object = syn::parse_macro_input!(object as ItemStruct);
    let fields = parse_fields(&object.fields);
    let name = object.ident.clone();

    let expressions = parse_settings.generate_parse_expressions(&fields);
    let names: Vec<_> = fields.iter().map(|field| field.get_name()).collect();

    let tokens = quote! {
        #object

        impl nom_parse_trait::ParseFrom<&str> for #name {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom::Parser;

                let mut input = input;
                #(#expressions)*
                Ok((input, Self { #(#names),* }))
            }
        }
    };

    tokens.into()
}
