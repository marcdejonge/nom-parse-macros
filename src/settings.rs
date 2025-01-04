use crate::fields::Field;
use crate::nom_packages::apply_nom_namespaces;
use itertools::Itertools;
use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::parse;
use syn::spanned::Spanned;

pub enum ParseSettings {
    Split(syn::Expr),
    Match(String),
}

impl ParseSettings {
    pub fn from(attrs: TokenStream) -> Result<Self, syn::Error> {
        let mut token_iter = attrs.into_iter();
        let first = token_iter.next();
        match first {
            None => Ok(Self::Split(syn::parse_quote! { line_ending })),
            Some(TokenTree::Ident(ident)) => {
                let name = ident.to_string();
                match name.as_str() {
                    "split" => {
                        match token_iter.next() {
                            None => return Err(syn::Error::new(ident.span(), "Missing '='")),
                            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {}
                            _ => return Err(syn::Error::new(ident.span(), "Missing '='")),
                        }

                        parse::<syn::Expr>(token_iter.collect::<TokenStream>().into())
                            .map(|expr| Self::Split(expr))
                    }
                    "match" => Ok(Self::Match("".to_string())),
                    _ => Err(syn::Error::new(ident.span(), "Unknown attribute name")),
                }
            }
            Some(TokenTree::Literal(literal)) => Ok(Self::Match(literal.to_string())),
            _ => Err(syn::Error::new(
                first.span(),
                "Expected literal matching string or attributes as configuration",
            )),
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
                vec![syn::Error::new(literal.span(), "Match not yet implemented").to_compile_error()]
            }
        }
    }
}
