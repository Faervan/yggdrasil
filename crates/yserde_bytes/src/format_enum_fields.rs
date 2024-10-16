use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, Variant};

use crate::{parse_field::parse_fields, AcceptedField};

pub fn format_enum_variant(variant: &Variant) -> TokenStream2 {
    let ident = &variant.ident;
    let (fields, is_named) = parse_fields(&variant.fields);
    if fields.is_empty() {
        return quote! {#ident};
    }
    let formatted = format_enum_fields(fields, is_named);
    match is_named {
        true => quote! {#ident {#formatted}},
        false => quote! {#ident(#formatted)}
    }
}

fn format_enum_fields(fields: Vec<AcceptedField>, is_named: bool) -> TokenStream2 {
    fields.into_iter().fold(quote! {}, |acc, field| {
        let field_ident = Ident::new(format!("field_{}",
            field.ident.to_string()).as_str(),
            Span::call_site()
        );
        let ident = field.ident;
        match is_named {
            true => quote! {#acc #ident: #field_ident,},
            false => quote! {#acc #field_ident,}
        }
    })
}
