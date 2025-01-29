use crate::nom_packages::update_nom_expression;
use proc_macro2::Span;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{parse_quote, parse_quote_spanned, ExprBlock};

#[derive(Debug, PartialEq)]
pub enum ParseFormat {
    Match(syn::LitStr),
    Expr(syn::Expr),
    Default,
}

impl Parse for ParseFormat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(ParseFormat::Default)
        } else if input.peek(syn::Token![match]) {
            let _keyword = input.parse::<syn::Token![match]>()?;
            let literal = input.parse::<syn::LitStr>()?;
            Ok(ParseFormat::Match(literal))
        } else {
            let expr = input.parse::<syn::Expr>()?;
            Ok(ParseFormat::Expr(expr))
        }
    }
}

impl ToTokens for ParseFormat {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.to_expr() {
            Ok(expr) => expr.to_tokens(tokens),
            Err(err) => err.to_compile_error().to_tokens(tokens),
        }
    }
}

impl ParseFormat {
    fn to_expr(&self) -> syn::Result<syn::Expr> {
        match self {
            ParseFormat::Match(string) => generate_match_expression(string),
            ParseFormat::Expr(expr) => {
                let mut expr = expr.clone();
                update_nom_expression(&mut expr)?;
                Ok(expr)
            }
            ParseFormat::Default => {
                let mut expr: syn::Expr = parse_quote!(());
                update_nom_expression(&mut expr)?;
                Ok(expr)
            }
        }
    }
}

pub fn generate_match_expression(literal: &syn::LitStr) -> syn::Result<syn::Expr> {
    let value = literal.value();
    let mut block: ExprBlock = parse_quote!({});
    let statements = &mut block.block.stmts;
    let mut names = vec![];
    let mut first = true;

    for (index, part) in value.split("{}").enumerate() {
        if first {
            first = false;
        } else {
            let name = syn::Ident::new(&format!("field_{}", index), literal.span());
            statements.push(parse_quote_spanned! { literal.span() =>
                let (input, #name) = nom_parse_trait::ParseFrom::parse(input)?;
            });
            names.push(name);
        }

        if !part.is_empty() {
            let expr = generate_match_literal(part.as_bytes(), literal.span());
            statements.push(parse_quote_spanned! { literal.span() =>
                let (input, _) = #expr.parse(input)?;
            });
        }
    }

    statements.push(parse_quote_spanned! { literal.span() =>
        return Ok((input, (#(#names),*)));
    });

    Ok(parse_quote_spanned!( literal.span() => ( |input| #block )))
}

pub fn generate_match_literal(value: &[u8], span: Span) -> syn::Expr {
    let lit = syn::LitByteStr::new(value, span);
    parse_quote!(nom::bytes::complete::tag(#lit.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    #[test]
    fn test_match_expr() {
        let value = ParseFormat::Match(syn::LitStr::new("some {}", Span::call_site()));
        let expr = value.to_expr().unwrap();
        let expected: syn::Expr = parse_quote!(
            (|input| {
                let (input, _) = nom::bytes::complete::tag(b"some ".as_ref()).parse(input)?;
                let (input, field_1) = nom_parse_trait::ParseFrom::parse(input)?;
                return Ok((input, (field_1)));
            })
        );
        assert_eq!(expected, expr);
    }

    #[test]
    fn test_expr_expr() {
        let value = ParseFormat::Expr(parse_quote!(preceded("test", ())));
        let expr = value.to_expr().unwrap();
        let expected: syn::Expr = parse_quote!(nom::sequence::preceded(
            nom::bytes::complete::tag(b"test".as_ref()),
            nom_parse_trait::ParseFrom::parse
        ));
        assert_eq!(expected, expr);
    }

    #[test]
    fn test_default_expr() {
        let value = ParseFormat::Default;
        let expr = value.to_expr().unwrap();
        let expected: syn::Expr = parse_quote!(nom_parse_trait::ParseFrom::parse);
        assert_eq!(expected, expr);
    }

    #[test]
    fn test_generate_parser_expr() {
        let value = "test {}{} test";
        let expr = generate_match_expression(&syn::LitStr::new(value, Span::call_site())).unwrap();
        let expected: syn::Expr = parse_quote!(
            (|input| {
                let (input, _) = nom::bytes::complete::tag(b"test ".as_ref()).parse(input)?;
                let (input, field_1) = nom_parse_trait::ParseFrom::parse(input)?;
                let (input, field_2) = nom_parse_trait::ParseFrom::parse(input)?;
                let (input, _) = nom::bytes::complete::tag(b" test".as_ref()).parse(input)?;
                return Ok((input, (field_1, field_2)));
            })
        );
        assert_eq!(expected, expr);
    }

    #[test]
    fn test_generate_match_literal() {
        let value = b"test\0\"!!";
        let span = Span::call_site();
        let expr: syn::Expr = generate_match_literal(value, span);
        assert_eq!(
            "nom :: bytes :: complete :: tag (b\"test\\0\\\"!!\" . as_ref ())",
            &expr.to_token_stream().to_string()
        );
    }
}
