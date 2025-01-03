extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;
use syn::ItemStruct;

#[proc_macro_attribute]
pub fn parse_from(_attrs: TokenStream, object: TokenStream) -> TokenStream {
    let object = syn::parse_macro_input!(object as ItemStruct);
    let name = object.ident.clone();

    let tokens = quote! {
        #object

        impl nom_parse_trait::ParseFrom<&str> for #name {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                use nom_parse_trait::ParseFrom;

                nom::combinator::map(
                    nom::sequence::separated_pair(
                        u32::parse,
                        nom::character::complete::space1,
                        u32::parse,
                    ),
                    |(x, y)| Self { x, y },
                )(input)
            }
        }
    };

    tokens.into()
}
