//! # nom-parse-trait
//!
//! This macro generates a `ParseFrom` implementation for a struct or enum using the provided
//! nom expression(s). The expression should return a tuple for the parsed fields.
//!
//! There are 2 separate macros available, [`parse_from()`] and [`parse_match()`].
//! The first one is really generic and can be useful in many cases, since you have the full
//! flexibility of nom functions and combinators. The second one is a very simple one that
//! matches a string verbatim. This is useful when you have a very simple format that you want
//! to parse.
//!
//! # nom functions
//!
//! The expression in the `parse_from` attribute will be translated to be using valid nom functions.
//! The main function here is to automatically put the namespace before the function name, so you
//! don't need a ton of use statements in your code. But there are also a couple of special cases:
//!
//! - `{}` or `()` will be replaced with a [`nom_parse_trait::ParseFrom::parse`] call for the
//! corresponding field. This is useful when you are using types that have implemented the
//! `ParseFrom` trait already.
//! - Strings, bytes strings and characters will be translated to match the input verbatim using
//! the [`nom::bytes::complete::tag`] function.
//!
//! # Input types that are supported
//!
//! The generated `ParseFrom` implementation is made to be very generic, where it supports any
//! input and error type from nom. This is done with a where clause with many traits that the input
//! should have implemented. All of these are true for the standard `&str` and `&[u8]` types.
//!
//! If you run into a situation where the trait limitations on the input type does not match your
//! use case, please open an issue on the GitHub repository.
//!
//! # Known limitations
//!
//! - When your try to use a custom parser combinator, the nom function parser will try to change
//! all parameters to be nom parsers. This is useful in many cases, but when you need to pass in
//! a normal string for example, it won't work. In these cases, you can define a separate function
//! to wrap the call. I'm not sure how to fix that right now, but I'm open to suggestions.
//!
//! - Since the generated input type is very generic, all functions that you want to use in the
//! nom expression should also be very generic. In the future I might add a way to specify if you
//! want to generate a specific input type, but for now it's not possible.

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

/// This macro generates a [`nom_parse_trait::ParseFrom`] implementation for a struct or enum using
/// the provided nom expression(s). The expression should return a tuple for the parsed fields.
///
/// # Examples
///
/// ## Basic struct with fields
///
/// This first example shows how to parse a simple struct with two fields, using the `separated_pair`
/// combinator. Here we also show some of the special parsing that goes on behind the scenes, where
/// the special {} syntax means that it infers the type parser it needs to use in that place. Also,
/// we accept normal strings as matching input, which will be translated to `tag` function calls.
///
/// ```rust
/// use nom_parse_macros::parse_from;
///
/// #[parse_from(separated_pair({}, tuple(space0, ",", space0), {}))]
/// struct NumberPair {
///     x: u32,
///     y: u32,
/// }
/// ```
///
/// ## Basic enum with variants
///
/// This example shows how we can define a format for each variant in an enum. The first variant
/// actually uses the default `ParseFrom` implementation for parsing the u32. The `Numbers` variant
/// uses a custom format, which is a delimited list of u32 values.
///
/// ```rust
/// use nom_parse_macros::parse_from;
///
/// #[parse_from]
/// enum MultipleTypes {
///     Number(u32),
///     #[format(delimited('(', separated_list0(",", {}), ')'))]
///     Numbers(Vec<u32>),
/// }
/// ```
///
/// ## Derived fields
///
/// Sometimes it's useful to have a field that is not actually parsed, but derived from the other
/// fields. This can be done with the `#[derived]` attribute. In this example, we derive the sum of
/// the two fields `x` and `y`.
///
/// ```rust
/// use nom_parse_macros::parse_from;
///
/// #[parse_from(separated_pair({}, tuple(space0, ",", space0), {}))]
/// struct NumberPair {
///     x: u32,
///     y: u32,
///     #[derived(x + y)]
///     sum: u32,
/// }
/// ```

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

/// The `parse_match` macro can be used to match strings verbatim. This is useful when you have
/// a very simple format that you want to parse. The {} gets replaced with a parser for the
/// corresponding field. The rest of the characters are matched verbatim.
///
/// # Example
///
/// This example shows how to parse a three-dimensional vector from a string with a fixed format.
/// As you can see, this macro is limited in its use, but is very straightforward to use in cases
/// where it works.
///
/// ```rust
/// use nom_parse_macros::parse_match;
///
/// #[parse_match("({},{},{})")]
/// struct Vector3 {
///     x: u32,
///     y: u32,
///     z: u32,
/// }
/// ```
///
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
