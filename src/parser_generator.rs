use crate::parse_format::ParseFormat;
use crate::parsed_item::{ParsedItem, ParsedVariant};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, GenericParam, Generics, TypeParam, WhereClause, WherePredicate};

pub(crate) struct ParserGenerator {
    parse_format: ParseFormat,
    parsed_item: ParsedItem,
}

impl ParserGenerator {
    pub fn new(parse_format: ParseFormat, parsed_item: ParsedItem) -> Self {
        Self {
            parse_format,
            parsed_item,
        }
    }
}

impl ToTokens for ParserGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match &self.parsed_item {
            ParsedItem::Struct { object, fields } => {
                tokens.extend(object.to_token_stream());

                let expression_names = fields.get_expression_names();
                let expression = &self.parse_format;
                let derived_expressions = fields.get_derived_expressions();
                let create_expr = fields.create_instance_expr(None);

                generate_parser(
                    tokens,
                    &object.ident,
                    &object.generics,
                    quote! {
                        let (input, (#(#expression_names),*)) = #expression.parse(input)?;
                        #(#derived_expressions)*
                        Ok((input, #create_expr))
                    },
                );
            }
            ParsedItem::Enum { object, variants } => {
                if self.parse_format != ParseFormat::Default {
                    tokens.extend(quote! { compile_error!("Enums cannot have a format") });
                    return;
                }

                tokens.extend(object.to_token_stream());

                let (mapping_names, mappings): (Vec<_>, Vec<_>) = variants
                    .iter()
                    .map(|variant| generate_variant(variant))
                    .unzip();

                generate_parser(
                    tokens,
                    &object.ident,
                    &object.generics,
                    quote! {
                        #(#mappings)*
                        nom::branch::alt((
                            #(#mapping_names),*
                        )).parse(input)
                    },
                );
            }
        }
    }
}

fn generate_variant(variant: &ParsedVariant) -> (Ident, TokenStream) {
    let mapping_name = Ident::new(
        &format!("map_{}", variant.name.to_string().to_lowercase()),
        Span::call_site(),
    );
    let format_expr = variant.format.to_token_stream();

    let expression_names = variant.fields.get_expression_names();
    let create_expr = variant.fields.create_instance_expr(Some(&variant.name));

    let mapping = if expression_names.is_empty() {
        // Parsing a variant without fields
        quote! {
            let #mapping_name = nom::combinator::map(#format_expr, |_| { #create_expr } );
        }
    } else {
        let expression_types = variant.fields.get_expression_types();
        let derived_expressions = variant.fields.get_derived_expressions();

        quote! {
            let #mapping_name = nom::combinator::map(
                #format_expr,
                |(#(#expression_names),*): (#(#expression_types),*)| {
                    #(#derived_expressions)*
                    #create_expr
                }
            );
        }
    };

    (mapping_name, mapping)
}

fn generate_parser(
    token_stream: &mut TokenStream,
    name: &Ident,
    generics: &Generics,
    content: impl ToTokens,
) {
    let (_, type_generics, _) = generics.split_for_impl();
    let parser_generics = parser_generics(&generics);
    let (impl_generics, _, where_statement) = parser_generics.split_for_impl();

    token_stream.extend(quote! {
        impl #impl_generics nom_parse_trait::ParseFrom<I, E> for #name #type_generics
        #where_statement
        {
            fn parse(input: I) -> nom::IResult<I, Self, E> {
                use nom::*;
                use nom_parse_trait::ParseFrom;

                #content
            }
        }
    });
}

fn parser_generics(generics: &Generics) -> Generics {
    let mut generics = generics.clone();

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

    let predicates = &mut generics.where_clause.as_mut().unwrap().predicates;
    for extra_parse_from_traits in extra_parse_from_traits {
        predicates.push(extra_parse_from_traits);
    }

    predicates.push(parse_quote! { E: nom::error::ParseError<I> });
    predicates.push(parse_quote! { I: nom::Input + nom::AsBytes });
    predicates.push(parse_quote! { <I as nom::Input>::Item: nom::AsChar + Copy });
    predicates.push(parse_quote! { I: for<'a> nom::Compare<&'a [u8]> });
    predicates.push(parse_quote! { I: nom::Compare<&'static str> });
    predicates.push(parse_quote! { for<'a> &'a str: nom::FindToken<<I as nom::Input>::Item> });

    generics
}
