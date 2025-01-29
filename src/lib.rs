//! # nom-parse-trait
//!
//! This macro generates a `ParseFrom` implementation for a struct or enum using the provided
//! nom expression(s). The expression should return a tuple for the parsed fields.
//!
//! The [`parse_from()`] macro can be used in 2 separate ways.
//! The first one is using an expression that results in a nom parser. This is generic and can be
//! useful in many cases, since you have the full flexibility of nom functions and combinators.
//! The second one is a very simple one that matches a string verbatim. You do this by starting the
//! expression with the `match` keyword. This is useful when you have a very simple format that you
//! want to parse.
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
mod parse_format;
mod parsed_item;
mod parser_generator;

use crate::parse_format::ParseFormat;
use crate::parsed_item::ParsedItem;
use crate::parser_generator::ParserGenerator;
use proc_macro::TokenStream;
use quote::ToTokens;

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
/// #[parse_from(separated_pair({}, (space0, ",", space0), {}))]
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
/// #[parse_from(separated_pair({}, (space0, ",", space0), {}))]
/// struct NumberPair {
///     x: u32,
///     y: u32,
///     #[derived(x + y)]
///     sum: u32,
/// }
/// ```
///
/// ## Match verbatim
///
/// This example shows how to match a string verbatim. This is useful when you have a very simple
/// format that you want to parse. In this case, we match a vector inside braces. As you can see
/// the `{}` placeholders are replaced with the corresponding field parsers.
///
/// ```rust
/// use nom_parse_macros::parse_from;
///
/// #[parse_from(match "({}, {})")]
/// struct Vector {
///   x: f32,
///   y: f32,
/// }
/// ```

#[proc_macro_attribute]
pub fn parse_from(attrs: TokenStream, object: TokenStream) -> TokenStream {
    let parse_format = syn::parse_macro_input!(attrs as ParseFormat);
    let parsed_item = syn::parse_macro_input!(object as ParsedItem);

    ParserGenerator::new(parse_format, parsed_item)
        .to_token_stream()
        .into()
}
