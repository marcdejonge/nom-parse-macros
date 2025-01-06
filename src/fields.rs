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
        syn::Fields::Unnamed(unnamed_fields) => parse_unnamed_fields(unnamed_fields),
        syn::Fields::Unit => Ok(Fields {
            is_named: false,
            fields: Vec::new(),
        }),
    }
}

fn parse_named_fields(fields: &mut FieldsNamed) -> Result<Fields> {
    parse_field_iterator(fields.named.iter_mut(), |_, field| {
        field.ident.clone().unwrap()
    })
    .map(|fields| Fields {
        is_named: true,
        fields,
    })
}

fn parse_unnamed_fields(fields: &mut FieldsUnnamed) -> Result<Fields> {
    parse_field_iterator(fields.unnamed.iter_mut(), |index, _| {
        Ident::new(&format!("field_{}", index), Span::call_site())
    })
    .map(|fields| Fields {
        is_named: false,
        fields,
    })
}

fn parse_field_iterator<'a>(
    fields: impl Iterator<Item = &'a mut syn::Field>,
    get_name: impl Fn(usize, &syn::Field) -> Ident,
) -> Result<Vec<Field>> {
    let mut result = Vec::new();

    for (index, field) in fields.enumerate() {
        let mut name = get_name(index, field);
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

    pub fn create_instance_expr(&self, variant_name: Option<&Ident>) -> TokenStream {
        let all_names = self.get_all_names();

        if let Some(name) = variant_name {
            if self.is_named {
                quote! { Self::#name { #(#all_names),* } }
            } else {
                quote! { Self::#name(#(#all_names),*) }
            }
        } else {
            if self.is_named {
                quote! { Self { #(#all_names),* } }
            } else {
                quote! { Self(#(#all_names),*) }
            }
        }
    }
}
