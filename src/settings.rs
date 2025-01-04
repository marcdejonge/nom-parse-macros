use crate::fields::Field;
use crate::nom_packages::apply_nom_namespaces;
use itertools::Itertools;
use proc_macro2::token_stream::IntoIter;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse, Error, Lit};

pub enum ParseSettings {
    Split(syn::Expr),
    Match(syn::LitStr),
}

impl ParseSettings {
    pub fn from(attrs: TokenStream) -> Result<Self, Error> {
        let mut token_iter = attrs.into_iter();
        let first = token_iter.next();
        match first {
            None => Ok(Self::Split(syn::parse_quote! { line_ending })),
            Some(TokenTree::Ident(ident)) => {
                let name = ident.to_string();
                match_equal_sign(&mut token_iter, &ident)?;

                match name.as_str() {
                    "split" => parse::<syn::Expr>(token_iter.collect::<TokenStream>().into())
                        .map(|expr| Self::Split(expr)),
                    "match" => Self::from_literal(parse::<syn::Lit>(
                        token_iter.collect::<TokenStream>().into(),
                    )?),
                    _ => Err(Error::new(
                        ident.span(),
                        "Unknown attribute name, expected 'split' or 'match'",
                    )),
                }
            }
            Some(TokenTree::Literal(literal)) => {
                Self::from_literal(parse::<syn::Lit>(literal.into_token_stream().into())?)
            }
            _ => Err(Error::new(
                first.span(),
                "Expected literal matching string or attributes as configuration",
            )),
        }
    }

    fn from_literal(lit: Lit) -> Result<ParseSettings, Error> {
        if let Lit::Str(lit) = lit {
            Ok(Self::Match(lit))
        } else {
            Err(Error::new(lit.span(), "Expected string literal"))
        }
    }

    pub fn generate_parse_expressions(&self, fields: &[Field]) -> Vec<TokenStream> {
        match self {
            ParseSettings::Split(expr) => {
                let mut expr = expr.clone();
                apply_nom_namespaces(&mut expr);
                let expr = quote! { let (input, _) = #expr.parse(input)?; };
                Itertools::intersperse(
                    fields
                        .iter()
                        .map(|field| field.generate_expression().into_token_stream()),
                    expr,
                )
                .collect()
            }
            ParseSettings::Match(literal) => {
                let value = literal.value();
                let parts: Vec<_> = value
                    .split("{}")
                    .map(|part| {
                        quote! { let (input, _) = nom::bytes::complete::tag(#part)(input)?; }
                    })
                    .collect();

                if parts.len() != fields.len() + 1 {
                    return vec![Error::new(
                        literal.span(),
                        "Number of {} does not match number of fields",
                    )
                    .to_compile_error()];
                }

                Itertools::interleave(
                    parts.into_iter(),
                    fields
                        .iter()
                        .map(|field| field.generate_expression().into_token_stream()),
                )
                .collect()
            }
        }
    }
}

fn match_equal_sign(token_iter: &mut IntoIter, ident: &Ident) -> Result<(), Error> {
    match token_iter.next() {
        None => Err(Error::new(ident.span(), "Missing '='")),
        Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => Ok(()),
        _ => Err(Error::new(ident.span(), "Missing '='")),
    }
}
