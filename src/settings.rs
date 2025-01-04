use crate::fields::Field;
use crate::nom_packages::apply_nom_namespaces;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse, Error, Expr, LitStr, Token};

pub enum ParseSettings {
    Split(Expr),
    Match(LitStr),
}

impl ParseSettings {
    pub fn from(attrs: TokenStream) -> syn::Result<Self> {
        let arguments = parse::<Arguments>(attrs.into())?;

        match arguments {
            Arguments::Split { expr, .. } => Ok(ParseSettings::Split(expr)),
            Arguments::Match { lit, .. } => Ok(ParseSettings::Match(lit)),
            Arguments::LiteralMatch { lit } => Ok(ParseSettings::Match(lit)),
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

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(split);
}

#[allow(dead_code)]
enum Arguments {
    Split {
        token: keywords::split,
        eq_token: Token![=],
        expr: Expr,
    },
    Match {
        token: Token![match],
        eq_token: Token![=],
        lit: LitStr,
    },
    LiteralMatch {
        lit: LitStr,
    },
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keywords::split) {
            let token = input.parse::<keywords::split>()?;
            let eq_token = input.parse::<Token![=]>()?;
            let expr = input.parse::<Expr>()?;
            Ok(Arguments::Split {
                token,
                eq_token,
                expr,
            })
        } else if lookahead.peek(Token![match]) {
            let token = input.parse::<Token![match]>()?;
            let eq_token = input.parse::<Token![=]>()?;
            let lit = input.parse::<LitStr>()?;
            Ok(Arguments::Match {
                token,
                eq_token,
                lit,
            })
        } else {
            let lit = input.parse::<LitStr>()?;
            Ok(Arguments::LiteralMatch { lit })
        }
    }
}
