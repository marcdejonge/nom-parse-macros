use crate::parse_format::generate_match_literal;
use quote::quote_spanned;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse, parse_quote_spanned, parse_str, Expr, ExprCall, ExprPath, ExprTuple, Lit, Path, Result,
};

const NOM_FUNCTIONS: phf::Map<&'static str, (&'static str, &'static [bool])> = phf::phf_map! {
    // From the nom::branch module
    "alt" => ("nom::branch::alt", &[]), // Special handling for alt
    // From the nom::bytes::complete module
    "tag" => ("nom::bytes::complete::tag", &[false]),
    "tag_no_case" => ("nom::bytes::complete::tag_no_case", &[false]),
    "is_not" => ("nom::bytes::complete::is_not", &[false]),
    "is_a" => ("nom::bytes::complete::is_a", &[false]),
    "take_while" => ("nom::bytes::complete::take_while", &[false]),
    "take_while1" => ("nom::bytes::complete::take_while1", &[false]),
    "take_while_m_n" => ("nom::bytes::complete::take_while_m_n", &[false, false, false]),
    "take_till" => ("nom::bytes::complete::take_till", &[false]),
    "take_till1" => ("nom::bytes::complete::take_till1", &[false]),
    "take" => ("nom::bytes::complete::take", &[false]),
    "take_until" => ("nom::bytes::complete::take_until", &[false]),
    "take_until1" => ("nom::bytes::complete::take_until1", &[false]),
    "escaped" => ("nom::bytes::complete::escaped", &[true, false, true]),
    "escaped_transform" => ("nom::bytes::complete::escaped_transform", &[true, false, true]),
    // From the nom::character::complete module
    "char" => ("nom::character::complete::char", &[false]),
    "satisfy" => ("nom::character::complete::satisfy", &[false]),
    "one_of" => ("nom::character::complete::one_of", &[false]),
    "none_of" => ("nom::character::complete::none_of", &[false]),
    "crlf" => ("nom::character::complete::crlf", &[]),
    "not_line_ending" => ("nom::character::complete::not_line_ending", &[]),
    "line_ending" => ("nom::character::complete::line_ending", &[]),
    "newline" => ("nom::character::complete::newline", &[]),
    "tab" => ("nom::character::complete::tab", &[]),
    "anychar" => ("nom::character::complete::anychar", &[]),
    "alpha0" => ("nom::character::complete::alpha0", &[]),
    "alpha1" => ("nom::character::complete::alpha1", &[]),
    "digit0" => ("nom::character::complete::digit0", &[]),
    "digit1" => ("nom::character::complete::digit1", &[]),
    "hex_digit0" => ("nom::character::complete::hex_digit0", &[]),
    "hex_digit1" => ("nom::character::complete::hex_digit1", &[]),
    "oct_digit0" => ("nom::character::complete::oct_digit0", &[]),
    "oct_digit1" => ("nom::character::complete::oct_digit1", &[]),
    "alphanumeric0" => ("nom::character::complete::alphanumeric0", &[]),
    "alphanumeric1" => ("nom::character::complete::alphanumeric1", &[]),
    "space0" => ("nom::character::complete::space0", &[]),
    "space1" => ("nom::character::complete::space1", &[]),
    "multispace0" => ("nom::character::complete::multispace0", &[]),
    "multispace1" => ("nom::character::complete::multispace1", &[]),
    "sign" => ("nom::character::complete::sign", &[]),
    // Primitive parsers
    "u8" => ("nom::character::complete::u8", &[]),
    "u16" => ("nom::character::complete::u16", &[]),
    "u32" => ("nom::character::complete::u32", &[]),
    "u64" => ("nom::character::complete::u64", &[]),
    "u128" => ("nom::character::complete::u128", &[]),
    "i8" => ("nom::character::complete::i8", &[]),
    "i16" => ("nom::character::complete::i16", &[]),
    "i32" => ("nom::character::complete::i32", &[]),
    "i64" => ("nom::character::complete::i64", &[]),
    "i128" => ("nom::character::complete::i128", &[]),
    // From the nom::number::complete module,
    "be_u8" => ("nom::number::complete::be_u8", &[]),
    "be_i8" => ("nom::number::complete::be_i8", &[]),
    "be_u16" => ("nom::number::complete::be_u16", &[]),
    "be_i16" => ("nom::number::complete::be_i16", &[]),
    "be_u24" => ("nom::number::complete::be_u24", &[]),
    "be_i24" => ("nom::number::complete::be_i24", &[]),
    "be_u32" => ("nom::number::complete::be_u32", &[]),
    "be_i32" => ("nom::number::complete::be_i32", &[]),
    "be_u64" => ("nom::number::complete::be_u64", &[]),
    "be_i64" => ("nom::number::complete::be_i64", &[]),
    "be_u128" => ("nom::number::complete::be_u128", &[]),
    "be_i128" => ("nom::number::complete::be_i128", &[]),
    "be_f32" => ("nom::number::complete::be_f32", &[]),
    "be_f64" => ("nom::number::complete::be_f64", &[]),
    "le_u8" => ("nom::number::complete::le_u8", &[]),
    "le_i8" => ("nom::number::complete::le_i8", &[]),
    "le_u16" => ("nom::number::complete::le_u16", &[]),
    "le_i16" => ("nom::number::complete::le_i16", &[]),
    "le_u24" => ("nom::number::complete::le_u24", &[]),
    "le_i24" => ("nom::number::complete::le_i24", &[]),
    "le_u32" => ("nom::number::complete::le_u32", &[]),
    "le_i32" => ("nom::number::complete::le_i32", &[]),
    "le_u64" => ("nom::number::complete::le_u64", &[]),
    "le_i64" => ("nom::number::complete::le_i64", &[]),
    "le_u128" => ("nom::number::complete::le_u128", &[]),
    "le_i128" => ("nom::number::complete::le_i128", &[]),
    "le_f32" => ("nom::number::complete::le_f32", &[]),
    "le_f64" => ("nom::number::complete::le_f64", &[]),
    "hex_u32" => ("nom::number::complete::hex_u32", &[]),
    "float" => ("nom::number::complete::float", &[]),
    "double" => ("nom::number::complete::double", &[]),
    // From the nom::combinator module
    "rest" => ("nom::combinator::rest", &[]),
    "rest_len" => ("nom::combinator::rest_len", &[]),
    "map" => ("nom::combinator::map", &[true, false]),
    "map_res" => ("nom::combinator::map_res", &[true, false]),
    "map_opt" => ("nom::combinator::map_opt", &[true, false]),
    "map_parser" => ("nom::combinator::map_parser", &[true, false]),
    "flat_map" => ("nom::combinator::flat_map", &[true, true]),
    "opt" => ("nom::combinator::opt", &[true]),
    "cond" => ("nom::combinator::cond", &[false, true]),
    "peek" => ("nom::combinator::peek", &[true]),
    "eof" => ("nom::combinator::eof", &[]),
    "complete" => ("nom::combinator::complete", &[true]),
    "all_consuming" => ("nom::combinator::all_consuming", &[true]),
    "verify" => ("nom::combinator::verify", &[true, false]),
    "value" => ("nom::combinator::value", &[false, true]),
    "not" => ("nom::combinator::not", &[true]),
    "recognize" => ("nom::combinator::recognize", &[true]),
    "consumed" => ("nom::combinator::consumed", &[true]),
    "cut" => ("nom::combinator::cut", &[true]),
    "into" => ("nom::combinator::into", &[true]),
    "success" => ("nom::combinator::success", &[]),
    "fail" => ("nom::combinator::fail", &[]),
    // From the nom::multi module
    "many0" => ("nom::multi::many0", &[true]),
    "many1" => ("nom::multi::many1", &[true]),
    "many_till" => ("nom::multi::many_till", &[true, true]),
    "separated_list0" => ("nom::multi::separated_list0", &[true, true]),
    "separated_list1" => ("nom::multi::separated_list1", &[true, true]),
    "many_m_n" => ("nom::multi::many_m_n", &[false, false, true]),
    "many0_count" => ("nom::multi::many0_count", &[true]),
    "many1_count" => ("nom::multi::many1_count", &[true]),
    "count" => ("nom::multi::count", &[true, false]),
    "fill" => ("nom::multi::fill", &[true, false]),
    "fold_many0" => ("nom::multi::fold_many0", &[true, false, false]),
    "fold_many1" => ("nom::multi::fold_many1", &[true, false, false]),
    "fold_many_m_n" => ("nom::multi::fold_many_m_n", &[false, false, true, false, false]),
    "length_data" => ("nom::multi::length_data", &[true]),
    "length_value" => ("nom::multi::length_value", &[true, true]),
    "length_count" => ("nom::multi::length_count", &[true, true]),
    // From the nom::sequence module
    "pair" => ("nom::sequence::pair", &[true, true]),
    "preceded" => ("nom::sequence::preceded", &[true, true]),
    "terminated" => ("nom::sequence::terminated", &[true, true]),
    "separated_pair" => ("nom::sequence::separated_pair", &[true, true, true]),
    "delimited" => ("nom::sequence::delimited", &[true, true, true]),
    "tuple" => ("nom::sequence::tuple", &[]), // Special handling for tuples
};

pub fn update_nom_expression(expr: &mut Expr) -> Result<()> {
    match expr {
        Expr::Block(block_expr) => {
            if block_expr.block.stmts.is_empty() {
                *expr = parse::<Expr>(
                    quote_spanned! { block_expr.span() => nom_parse_trait::ParseFrom::parse }
                        .into(),
                )?;
                Ok(())
            } else {
                Err(syn::Error::new_spanned(
                    block_expr,
                    "Only supporting building nom parsers from function calls and string literals",
                ))
            }
        }
        Expr::Call(call) => parse_call(call),
        Expr::Lit(lit_expr) => match &lit_expr.lit {
            Lit::Str(value) => {
                *expr = generate_match_literal(value.value().as_bytes(), value.span());
                Ok(())
            }
            Lit::ByteStr(value) => {
                *expr = generate_match_literal(&value.value(), value.span());
                Ok(())
            }
            Lit::Byte(value) => {
                *expr = generate_match_literal(&[value.value()], value.span());
                Ok(())
            }
            Lit::Char(value) => {
                *expr = generate_match_literal(value.value().to_string().as_bytes(), value.span());
                Ok(())
            }
            _ => Err(syn::Error::new_spanned(
                lit_expr.clone(),
                "Only supporting string, bytes or character literals for nom parsers",
            )),
        },
        Expr::Path(ExprPath { path, .. }) => parse_path(path),
        Expr::Tuple(ExprTuple { elems, .. }) => {
            if elems.is_empty() {
                // An empty tuple is used as a shortcut for the ParseFrom parser
                *expr = parse_quote_spanned! { elems.span() => nom_parse_trait::ParseFrom::parse };
            } else {
                // Tuples are assumed to be all parsers
                for elem in elems.iter_mut() {
                    update_nom_expression(elem)?;
                }
            }
            Ok(())
        }
        _ => Err(syn::Error::new_spanned(
            expr,
            "Only supporting building nom parsers from function calls and string literals",
        )),
    }
}

fn parse_call(call: &mut ExprCall) -> Result<()> {
    if let Expr::Path(ExprPath { path, .. }) = call.func.as_mut() {
        if path.segments.len() == 1 {
            let ident = path.segments[0].ident.to_string();
            let arguments = path.segments[0].arguments.clone();

            if let Some(&(nom_path, parameters)) = NOM_FUNCTIONS.get(ident.as_str()) {
                path.segments = parse_str::<Path>(nom_path)?.segments;
                path.segments.last_mut().unwrap().arguments = arguments;

                // For the tuple and alt functions, wrap the arguments in a tuple if they are not already
                // and handle the arguments as if they were all parsers
                if ident == "tuple" || ident == "alt" {
                    let args = call.args.clone();
                    if args.len() != 1 {
                        call.args = Punctuated::from(Punctuated::new());
                        call.args.push(Expr::Tuple(ExprTuple {
                            attrs: vec![],
                            paren_token: Default::default(),
                            elems: args,
                        }));
                    }

                    for arg in call.args.iter_mut() {
                        update_nom_expression(arg)?;
                    }
                // Nom functions without parameters should not be called, but referenced directly
                } else if parameters.len() == 0 {
                    if ident != "fail" {
                        return Err(syn::Error::new_spanned(
                            call.func.clone(),
                            format!("The function {} is a parser by itself and should be used here without parens", ident),
                        ));
                    }
                } else if parameters.len() != call.args.len() {
                    return Err(syn::Error::new_spanned(
                        call.func.clone(),
                        format!(
                            "The function {} expects {} arguments, but {} were provided",
                            ident,
                            parameters.len(),
                            call.args.len()
                        ),
                    ));
                // If the number of parameters is correct, we can make sure that parsers are handled correctly
                } else {
                    for (arg, &is_parser) in call.args.iter_mut().zip(parameters) {
                        if is_parser {
                            update_nom_expression(arg)?;
                        }
                    }
                }
            } else {
                // Assume that this is a custom function, for which all parameters need to be parsed as parsers
                for arg in call.args.iter_mut() {
                    update_nom_expression(arg)?;
                }
            }
        }

        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            call.func.clone(),
            "Only supporting simple function methods",
        ))
    }
}

fn parse_path(path_expr: &mut Path) -> Result<()> {
    if path_expr.segments.len() == 1 {
        let ident = path_expr.segments[0].ident.to_string();
        let arguments = path_expr.segments[0].arguments.clone();

        if let Some(&(nom_path, parameters)) = NOM_FUNCTIONS.get(ident.as_str()) {
            if !parameters.is_empty() {
                return Err(syn::Error::new_spanned(
                    path_expr,
                    format!(
                        "The function {} returns a parser, so it will need to be called with parameters",
                        ident
                    ),
                ));
            }

            path_expr.segments = parse_str::<Path>(nom_path)?.segments;
            path_expr.segments.last_mut().unwrap().arguments = arguments;
        }
    }

    Ok(())
}
