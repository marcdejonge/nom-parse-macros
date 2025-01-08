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
use std::default::Default;
use syn::{
    parse_macro_input, parse_quote, Expr, GenericParam, Generics, Item, ItemEnum, ItemStruct,
    LitStr, TypeParam, WhereClause, WherePredicate,
};

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
    let expression_names = fields.get_expression_names();
    let derived_expressions = fields.get_derived_expressions();
    let create_expr = fields.create_instance_expr(None);

    generate_parser(
        object.ident.clone(),
        object.generics.clone(),
        object,
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

    generate_parser(
        object.ident.clone(),
        object.generics.clone(),
        object,
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

    match parse_string_match(&fields, literal) {
        Ok(parts) => {
            let names: Vec<_> = fields.fields.iter().map(|field| field.get_name()).collect();
            generate_parser(
                object.ident.clone(),
                object.generics.clone(),
                object,
                quote! {
                    #(#parts)*
                    Ok((input, Self { #(#names),* }))
                },
            )
        }
        Err(e) => e.to_compile_error().into(),
    }
}

fn generate_parser(
    name: Ident,
    generics: Generics,
    object: impl ToTokens,
    content: impl ToTokens,
) -> TokenStream {
    let merged_generics = parser_generics(generics.clone());
    let (impl_generics, _, where_statement) = merged_generics.split_for_impl();
    let (_, type_generics, _) = generics.split_for_impl();

    let tokens = quote! {
        #object

        impl #impl_generics nom_parse_trait::ParseFrom<I, E> for #name #type_generics
        #where_statement
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

fn parser_generics(mut generics: Generics) -> Generics {
    // If there are no generics, start a new one
    if generics.params.is_empty() {
        generics = Generics::default();
        generics.lt_token = Some(Default::default());
        generics.gt_token = Some(Default::default());
    }

    // Generate some extra where predicates for the generics
    let extra_parse_from_traits: Vec<WherePredicate> = generics
        .params
        .iter()
        .flat_map(|param| {
            if let GenericParam::Type(TypeParam { ident, .. }) = param {
                Some(parse_quote! { #ident: nom_parse_trait::ParseFrom<I, E> })
            } else {
                None
            }
        })
        .collect();

    // Add the `I` and `E` generics that the ParseFrom implementation needs
    generics
        .params
        .push(GenericParam::Type(TypeParam::from(Ident::new(
            "I",
            Span::call_site(),
        ))));
    generics
        .params
        .push(GenericParam::Type(TypeParam::from(Ident::new(
            "E",
            Span::call_site(),
        ))));

    if generics.where_clause.is_none() {
        generics.where_clause = Some(WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
    }

    let where_clause = generics.where_clause.as_mut().unwrap();
    for extra_parse_from_traits in extra_parse_from_traits {
        where_clause.predicates.push(extra_parse_from_traits);
    }

    where_clause
        .predicates
        .push(parse_quote! { I: nom::InputTake + nom::InputLength + nom::Offset + nom::AsBytes });

    where_clause
        .predicates
        .push(parse_quote! { E: nom::error::ParseError<I> });
    where_clause.predicates.push(parse_quote! { I: Clone });
    where_clause.predicates.push(parse_quote! { I: nom::Slice<std::ops::RangeTo<usize>> + nom::Slice<std::ops::RangeFrom<usize>> + nom::Slice<std::ops::Range<usize>> });
    where_clause
        .predicates
        .push(parse_quote! { I: nom::InputTake + nom::InputLength + nom::Offset + nom::AsBytes });
    where_clause
        .predicates
        .push(parse_quote! { I: nom::InputIter });
    where_clause
        .predicates
        .push(parse_quote! { <I as nom::InputIter>::Item: nom::AsChar + Copy });
    where_clause
        .predicates
        .push(parse_quote! { <I as nom::InputIter>::IterElem: Clone });
    where_clause
        .predicates
        .push(parse_quote! { I: nom::InputTakeAtPosition });
    where_clause
        .predicates
        .push(parse_quote! { <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Copy });
    where_clause
        .predicates
        .push(parse_quote! { I: for<'a> nom::Compare<&'a [u8]> });
    where_clause
        .predicates
        .push(parse_quote! { I: nom::Compare<&'static str> });
    where_clause
        .predicates
        .push(parse_quote! { for<'a> &'a str: nom::FindToken<<I as nom::InputIter>::Item> });

    generics
}
