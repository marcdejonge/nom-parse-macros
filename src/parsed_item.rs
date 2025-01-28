use crate::fields::{parse_fields, Fields};
use crate::parse_format::ParseFormat;
use itertools::Itertools;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Item, ItemEnum, ItemStruct, Result, Variant};

pub enum ParsedItem {
    Struct {
        object: ItemStruct,
        fields: Fields,
    },
    Enum {
        object: ItemEnum,
        variants: Vec<ParsedVariant>,
    },
}

pub struct ParsedVariant {
    pub name: Ident,
    pub fields: Fields,
    pub format: ParseFormat,
}

impl Parse for ParsedItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let item = input.parse::<Item>()?;
        match item {
            Item::Struct(mut item_struct) => {
                let fields = parse_fields(&mut item_struct.fields)?;
                Ok(ParsedItem::Struct {
                    object: item_struct,
                    fields,
                })
            }
            Item::Enum(mut item_enum) => Ok(ParsedItem::Enum {
                variants: parse_variants(&mut item_enum.variants)?,
                object: item_enum,
            }),
            _ => Err(syn::Error::new(item.span(), "Expected struct or enum")),
        }
    }
}

fn parse_variants(variants: &mut Punctuated<Variant, Comma>) -> Result<Vec<ParsedVariant>> {
    let mut result = Vec::with_capacity(variants.len());

    for variant in variants {
        let format = if let Some((index, attr)) = variant
            .attrs
            .iter()
            .find_position(|attr| attr.path().is_ident("format"))
        {
            let tokens = attr.meta.require_list()?.tokens.to_token_stream().into();
            variant.attrs.remove(index);
            syn::parse::<ParseFormat>(tokens)?
        } else {
            ParseFormat::Default
        };

        let fields = parse_fields(&mut variant.fields)?;
        let name = variant.ident.clone();

        result.push(ParsedVariant {
            name,
            fields,
            format,
        });
    }

    Ok(result)
}
