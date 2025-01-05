use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Fields, FieldsNamed, FieldsUnnamed, Type};

pub enum Field {
    Default { name: Ident, ty: Type },
}

pub fn parse_fields(fields: &Fields) -> Vec<Field> {
    match fields {
        Fields::Named(named_fields) => parse_named_fields(named_fields),
        Fields::Unnamed(unnamed_fields) => parse_unnamed_fields(unnamed_fields),
        Fields::Unit => vec![],
    }
}

fn parse_named_fields(fields: &FieldsNamed) -> Vec<Field> {
    fields
        .named
        .iter()
        .map(|field| {
            let mut name = field.ident.clone().unwrap();
            name.set_span(Span::call_site());
            let ty = field.ty.clone();
            Field::Default { name, ty }
        })
        .collect()
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
    pub fn generate_expression(&self) -> proc_macro2::TokenStream {
        match self {
            Field::Default { name, ty } => {
                quote! {
                    let (input, #name): (_, #ty) = nom_parse_trait::ParseFrom::parse(input)?;
                }
            }
        }
    }

    pub fn get_name(&self) -> &Ident {
        match self {
            Field::Default { name, .. } => name,
        }
    }

    pub fn get_type(&self) -> &Type {
        match self {
            Field::Default { ty, .. } => ty,
        }
    }
}
