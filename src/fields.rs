use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Expr, Fields, FieldsNamed, FieldsUnnamed, Result, Type};

pub enum Field {
    Default { name: Ident, ty: Type },
    Derived { name: Ident, ty: Type, expr: Expr },
}

pub fn parse_fields(fields: &mut Fields) -> Result<Vec<Field>> {
    match fields {
        Fields::Named(named_fields) => parse_named_fields(named_fields),
        Fields::Unnamed(unnamed_fields) => Ok(parse_unnamed_fields(unnamed_fields)),
        Fields::Unit => Ok(Vec::new()),
    }
}

fn parse_named_fields(fields: &mut FieldsNamed) -> Result<Vec<Field>> {
    let mut result = Vec::new();

    for field in fields.named.iter_mut() {
        let mut name = field.ident.clone().unwrap();
        name.set_span(Span::call_site());
        let ty = field.ty.clone();

        if let Some((ix, attr)) = field
            .attrs
            .iter()
            .find_position(|attr| attr.path().is_ident("derived"))
        {
            let expr = attr.parse_args::<Expr>()?;
            field.attrs.remove(ix);
            result.push(Field::Derived { name, ty, expr });
        } else {
            result.push(Field::Default { name, ty });
        }
    }

    Ok(result)
}

fn parse_unnamed_fields(fields: &FieldsUnnamed) -> Vec<Field> {
    fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let name = Ident::new(&format!("field_{}", index), Span::call_site());
            let ty = field.ty.clone();
            Field::Default { name, ty }
        })
        .collect()
}

impl Field {
    pub fn generate_expression(&self) -> Option<TokenStream> {
        match self {
            Field::Default { name, ty } => Some(quote! {
                let (input, #name): (_, #ty) = nom_parse_trait::ParseFrom::parse(input)?;
            }),
            Field::Derived { .. } => None,
        }
    }

    pub fn get_name(&self) -> &Ident {
        match self {
            Field::Default { name, .. } => name,
            Field::Derived { name, .. } => name,
        }
    }

    pub fn generate_derived_expression(&self) -> Option<TokenStream> {
        match self {
            Field::Default { .. } => None,
            Field::Derived { name, expr, ty } => Some(quote! {
                let #name: #ty = #expr;
            }),
        }
    }
}
