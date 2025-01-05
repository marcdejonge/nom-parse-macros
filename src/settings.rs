use crate::fields::Field;
use crate::nom_packages::apply_nom_namespaces;
use itertools::Itertools;
use proc_macro::Span;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parse, Error, Expr, LitStr, Token};

pub enum ParseSettings {
    Split {
        prefix: Option<Expr>,
        split: Expr,
        suffix: Option<Expr>,
    },
    Match(LitStr),
}

impl ParseSettings {
    pub fn from(attrs: TokenStream) -> syn::Result<Self> {
        parse::<ParseSettings>(attrs.into())
    }

    pub fn generate_parse_expressions(&self, fields: &[Field]) -> Vec<TokenStream> {
        match self {
            ParseSettings::Split{ prefix, split, suffix} => {
                let mut split = split.clone();
                apply_nom_namespaces(&mut split);

                let mut expressions: Vec<_> = Itertools::intersperse(
                    fields
                        .iter()
                        .map(|field| field.generate_expression().into_token_stream()),
                    quote! { let (input, _) = split.parse(input)?; },
                )
                .collect();

                if let Some(prefix) = prefix {
                    let mut prefix = prefix.clone();
                    apply_nom_namespaces(&mut prefix);
                    expressions.insert(0, quote! { let (input, _) = #prefix.parse(input)?; });
                }

                if let Some(suffix) = suffix {
                    let mut suffix = suffix.clone();
                    apply_nom_namespaces(&mut suffix);
                    expressions.push(quote! { let (input, _) = #suffix.parse(input)?; });
                }

                expressions.insert(0, quote! { let mut split = #split; });
                expressions
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

    custom_keyword!(prefix);
    custom_keyword!(split);
    custom_keyword!(suffix);
}

impl Parse for ParseSettings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            let lit = input.parse::<LitStr>()?;
            return Ok(ParseSettings::Match(lit));
        }

        let mut prefix: Option<Expr> = None;
        let mut split: Option<Expr> = None;
        let mut suffix: Option<Expr> = None;
        let mut first = true;

        while !input.is_empty() {
            if first {
                first = false;
            } else {
                input.parse::<Token![;]>()?;
            }

            let lookahead = input.lookahead1();
            if lookahead.peek(keywords::prefix) {
                input.parse::<keywords::prefix>()?;
                input.parse::<Token![=]>()?;
                prefix = Some(input.parse()?);
            } else if lookahead.peek(keywords::split) {
                input.parse::<keywords::split>()?;
                input.parse::<Token![=]>()?;
                split = Some(input.parse()?);
            } else if lookahead.peek(keywords::suffix) {
                input.parse::<keywords::suffix>()?;
                input.parse::<Token![=]>()?;
                suffix = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }

        if let Some(split) = split {
            Ok(ParseSettings::Split {
                prefix,
                split,
                suffix,
            })
        } else {
            Err(Error::new(Span::call_site().into(), "Missing `split` keyword"))
        }
    }
}
