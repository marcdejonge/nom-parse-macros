use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Expr, FieldsNamed, FieldsUnnamed, Result, Type};

pub enum Field {
    Default { name: Ident, ty: Type },
    Derived { name: Ident, ty: Type, expr: Expr },
}

pub struct Fields {
    pub(crate) is_named: bool,
    pub(crate) fields: Vec<Field>,
}

pub fn parse_fields(fields: &mut syn::Fields) -> Result<Fields> {
    match fields {
        syn::Fields::Named(named_fields) => parse_named_fields(named_fields),
        syn::Fields::Unnamed(unnamed_fields) => Ok(parse_unnamed_fields(unnamed_fields)),
        syn::Fields::Unit => Ok(Fields {
            is_named: false,
            fields: Vec::new(),
        }),
    }
}

fn parse_named_fields(fields: &mut FieldsNamed) -> Result<Fields> {
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

    Ok(Fields {
        is_named: true,
        fields: result,
    })
}

fn parse_unnamed_fields(fields: &FieldsUnnamed) -> Fields {
    let fields = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let name = Ident::new(&format!("field_{}", index), Span::call_site());
            let ty = field.ty.clone();
            Field::Default { name, ty }
        })
        .collect();
    Fields {
        is_named: false,
        fields,
    }
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

    pub fn get_type(&self) -> &Type {
        match self {
            Field::Default { ty, .. } => ty,
            Field::Derived { ty, .. } => ty,
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

impl Fields {
    pub fn get_all_names(&self) -> Vec<Ident> {
        self.fields
            .iter()
            .map(|field| field.get_name().clone())
            .collect()
    }

    pub fn get_expression_names(&self) -> Vec<Ident> {
        self.fields
            .iter()
            .filter(|field| !matches!(field, Field::Derived { .. }))
            .map(|field| field.get_name().clone())
            .collect()
    }

    pub fn get_expression_types(&self) -> Vec<Type> {
        self.fields
            .iter()
            .filter(|field| !matches!(field, Field::Derived { .. }))
            .map(|field| field.get_type().clone())
            .collect()
    }

    pub fn get_derived_expressions(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .filter_map(|field| field.generate_derived_expression())
            .collect()
    }
}
