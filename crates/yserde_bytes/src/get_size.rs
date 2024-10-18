use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, Variant};

use crate::{parse_field::parse_fields, AcceptedField, DataField, DataType};

pub fn size_from_variants(variants: &Vec<&Variant>) -> TokenStream2 {
    let implementation = variants.into_iter().fold(quote! {}, |acc, variant| {
        let (fields, _) = parse_fields(&variant.fields);
        if fields.is_empty() {
            acc
        } else {
            let size_impl = size_from_fields(&fields);
            quote! {
                #acc,
                {
                    #size_impl
                }
            }
        }
    });
    quote! {
        max_size![
            0
            #implementation
        ] + 1
    }
}

pub fn size_from_fields(fields: &Vec<AcceptedField>) -> TokenStream2 {
    let mut total_size = 0;
    let implementation = fields.iter().fold(quote! {}, |acc, field| {
        match size_from_field(field) {
            Ok(size) => {
                total_size += size;
                acc
            }
            Err(implementation) => quote! {
                #acc
                #implementation
            }
        }
    });
    quote! {
        #total_size #implementation
    }
}

fn size_from_field(field: &AcceptedField) -> Result<usize, TokenStream2> {
    match &field.data {
        DataField::Vec(ty) => match size_of_datatype(ty) {
            Ok(size) => Ok(1 + 255 * size),
            Err(ident) => Err(quote! {
                + 1 + 255 * #ident::MAX_SIZE
            })
        },
        DataField::HashMap { key, value } => {
            let key_size = size_of_datatype(key);
            let val_size = size_of_datatype(value);
            match (key_size, val_size) {
                // Disgusting I know, but it does it's job
                (Ok(key_size), Ok(val_size)) => Ok(1 + 255 * key_size + 255 * val_size),
                (Err(key_ident), Err(val_ident)) => Err(quote! {
                    + 1 + 255 * #key_ident::MAX_SIZE + 255 * #val_ident::MAX_SIZE
                }),
                (Ok(size), Err(ident)) | (Err(ident), Ok(size)) => Err(quote! {
                    +1 + 255 * #size + 255 * #ident::MAX_SIZE
                })
            }
        }
        DataField::Option(ty) => match size_of_datatype(ty) {
            Ok(size) => Ok(1 + size),
            Err(ident) => Err(quote! {
                + 1 + #ident::MAX_SIZE
            })
        },
        DataField::Type(ty) => match size_of_datatype(ty) {
            Ok(size) => Ok(size),
            Err(ident) => Err(quote! {
                + #ident::MAX_SIZE + 1
            })
        }
    }
}

fn size_of_datatype(data: &DataType) -> Result<usize, Ident> {
    Ok(match data {
        DataType::U8 => 1,
        DataType::Bool => 1,
        DataType::String(length) => length.as_size(),
        DataType::Int(_, size) => *size,
        DataType::Package(ident) => return Err(ident.clone())
    })
}
