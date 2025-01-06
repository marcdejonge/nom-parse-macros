extern crate proc_macro;
mod fields;
mod nom_packages;
mod string_matching;

use crate::fields::parse_fields;
use crate::nom_packages::update_nom_expression;
use crate::string_matching::parse_string_match;
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Expr, Item, ItemEnum, ItemStruct, LitStr};

#[proc_macro_attribute]
pub fn parse_from(attrs: TokenStream, object: TokenStream) -> TokenStream {
    match parse_macro_input!(object as Item) {
        Item::Struct(item_struct) => {
            generate_struct_parser(attrs, item_struct)
        },
        Item::Enum(item_enum) => {
            generate_enum_parser(item_enum)
        },
        _ => (quote! { compiler_error!("Generating ParseFrom implementation only works for structs and enums") }).into()
    }
}

fn generate_struct_parser(attrs: TokenStream, mut object: ItemStruct) -> TokenStream {
    let mut expression = parse_macro_input! { attrs as Expr };
    if let Err(e) = update_nom_expression(&mut expression) {
        return e.to_compile_error().into();
    }

    let fields = match parse_fields(&mut object.fields) {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error().into(),
    };
    let name = object.ident.clone();

    let expression_names = fields.get_expression_names();
    let derived_expressions = fields.get_derived_expressions();
    let create_expr = fields.create_instance_expr(None);

    generate_parser(
        object,
        name,
        quote! {
            let (input, (#(#expression_names),*)) = #expression.parse(input)?;
            #(#derived_expressions)*
            Ok((input, #create_expr))
        },
    )
}

fn generate_enum_parser(mut object: ItemEnum) -> TokenStream {
    let mut mappings = Vec::new();
    let mut mapping_names = Vec::new();

    for variant in &mut object.variants {
        let format_expr = if let Some((index, attr)) = variant
            .attrs
            .iter()
            .find_position(|attr| attr.path().is_ident("format"))
        {
            let mut format_expr = match attr.meta.require_list() {
                Ok(list) => {
                    let tokens = list.tokens.clone().into_token_stream().into();
                    parse_macro_input! { tokens as Expr }
                }
                Err(e) => return e.to_compile_error().into(),
            };
            if let Err(e) = update_nom_expression(&mut format_expr) {
                return e.to_compile_error().into();
            }

            variant.attrs.remove(index);
            format_expr
        } else {
            let format_expr = quote! { nom_parse_trait::ParseFrom::parse };
            let format_expr = format_expr.into_token_stream().into();
            parse_macro_input! { format_expr as Expr }
        };

        let fields = match parse_fields(&mut variant.fields) {
            Ok(fields) => fields,
            Err(e) => return e.to_compile_error().into(),
        };
        let variant_name = variant.ident.clone();

        let expression_names = fields.get_expression_names();
        let expression_types = fields.get_expression_types();
        let derived_expressions = fields.get_derived_expressions();
        let create_expr = fields.create_instance_expr(Some(&variant_name));

        let mapping_name = Ident::new(
            &format!("map_{}", variant_name.to_string().to_lowercase()),
            Span::call_site(),
        );
        mapping_names.push(mapping_name.clone());

        if expression_names.is_empty() {
            // Parsing a variant without fields
            mappings.push(quote! {
                let #mapping_name = nom::combinator::map(
                    #format_expr,
                    |_| { #create_expr }
                );
            })
        } else {
            mappings.push(quote! {
                let #mapping_name = nom::combinator::map(
                    #format_expr,
                    |(#(#expression_names),*): (#(#expression_types),*)| {
                        #(#derived_expressions)*
                        #create_expr
                    }
                );
            })
        }
    }

    let name = object.ident.clone();

    generate_parser(
        object,
        name,
        quote! {
            #(#mappings)*
            nom::branch::alt((
                #(#mapping_names),*
            )).parse(input)
        },
    )
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
            let names: Vec<_> = fields.fields.iter().map(|field| field.get_name()).collect();
            generate_parser(
                object,
                name,
                quote! {
                    #(#parts)*
                    Ok((input, Self { #(#names),* }))
                },
            )
        }
        Err(e) => e.to_compile_error().into(),
    }
}

fn generate_parser(object: impl ToTokens, name: Ident, content: impl ToTokens) -> TokenStream {
    let tokens = quote! {
        #object

        impl<I, E> nom_parse_trait::ParseFrom<I, E> for #name
        where
            E: nom::error::ParseError<I>,
            I: Clone,
            I: nom::Slice<std::ops::RangeTo<usize>> + nom::Slice<std::ops::RangeFrom<usize>> + nom::Slice<std::ops::Range<usize>>,
            I: nom::InputTake + nom::InputLength + nom::Offset + nom::AsBytes,
            I: nom::InputIter,
            <I as nom::InputIter>::Item: nom::AsChar + Copy,
            <I as nom::InputIter>::IterElem: Clone,
            I: nom::InputTakeAtPosition,
            <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Copy,
            I: for<'a> nom::Compare<&'a [u8]>,
            I: nom::Compare<&'static str>,
        {
            fn parse(input: I) -> nom::IResult<I, Self, E> {
                use nom::*;
                use nom_parse_trait::ParseFrom;

                #content
            }
        }
    };
    tokens.into()
}
