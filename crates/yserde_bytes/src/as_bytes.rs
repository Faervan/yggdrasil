use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Variant;

use crate::{format_enum_fields::format_enum_variant, parse_field::parse_fields, AcceptedField, DataField, DataType, FieldAccess};

pub fn enum_as_bytes(variants: &Vec<&Variant>) -> TokenStream2 {
    let implementation = variants.into_iter().enumerate().fold(quote! {}, |acc, (index, variant)| {
        let (fields, _) = parse_fields(&variant.fields);
        let (push_fixed_bytes, _) = push_fixed_bytes(&fields, FieldAccess::Enum);
        let push_unknown_bytes = push_unknown_bytes(&fields, FieldAccess::Enum);
        let formatted_variant = format_enum_variant(*variant);
        quote! {
            #acc
            Self::#formatted_variant => {
                bytes.push(#index as u8);
                #push_fixed_bytes
                #push_unknown_bytes
            }
        }
    });
    quote! {
        match self {
            #implementation
        }
    }
}

pub fn push_fixed_bytes(fields: &Vec<AcceptedField>, access: FieldAccess) -> (TokenStream2, usize) {
    let mut fixed_buffer_size = 0;
    (fields.iter().fold(quote! {}, |tokens, field| {
        fixed_buffer_size += 1;
        let field_ident = &access.as_stream(&field.ident);
        let new_tokens = match &field.data {
            DataField::Option(_) => quote! {
                bytes.push(match #field_ident {
                    Some(_) => 1,
                    None => 0
                });
            },
            DataField::Vec(_) => quote! {
                bytes.push(#field_ident.len() as u8);
            },
            DataField::Type(ty) => push_fixed_part(ty, field_ident, Some(&mut fixed_buffer_size), &access)
        };
        quote! {
            #tokens
            #new_tokens
        }
    }), fixed_buffer_size)
}

pub fn push_unknown_bytes(fields: &Vec<AcceptedField>, access: FieldAccess) -> TokenStream2 {
    fields.iter().fold(quote! {}, |tokens, field| {
        let field_ident = &access.as_stream(&field.ident);
        let new_tokens = match &field.data {
            DataField::Option(ty) => {
                let push_fixed_part = push_fixed_part(ty, &quote! {#field_ident.as_ref().unwrap()}, None, &access);
                let push_unknown_part = push_unknown_part(ty, &quote! {#field_ident.as_ref().unwrap()});
                quote! {
                    if let Some(_) = #field_ident {
                        #push_fixed_part
                        #push_unknown_part
                    }
                }
            }
            DataField::Vec(ty) => {
                let push_fixed_part = push_fixed_part(ty, &quote! {#field_ident[i]}, None, &access);
                let push_unknown_part = push_unknown_part(ty, &quote! {#field_ident[i]});
                quote! {
                    let vec_len = 0..#field_ident.len();
                    for i in vec_len.clone() {
                        #push_fixed_part
                    }
                    for i in vec_len {
                        #push_unknown_part
                    }
                }
            }
            DataField::Type(ty) => push_unknown_part(ty, field_ident)
        };
        quote! {
            #tokens
            #new_tokens
        }
    })
}

fn push_fixed_part(ty: &DataType, field_ident: &TokenStream2, fixed_buffer_size: Option<&mut usize>, access: &FieldAccess) -> TokenStream2 {
    if let Some(size) = fixed_buffer_size {
        match ty {
            // Allready assigned +1, so need to substract that again
            DataType::Int(.., int_size) => *size += int_size -1,
            DataType::Package(_) => *size -= 1,
            _ => {}
        }
    }
    match ty {
        DataType::U8 => {
            match access {
                FieldAccess::Enum => quote! {bytes.push(*#field_ident);},
                FieldAccess::Struct => quote! {bytes.push(#field_ident);}
            }
        }
        DataType::Bool => quote! {
            bytes.push(match #field_ident {
                true => 1,
                false => 0
            });
        },
        DataType::String => quote! {
            bytes.push(#field_ident.len() as u8);
        },
        DataType::Int(..) => quote! {
            bytes.extend_from_slice(&#field_ident.to_ne_bytes());
        },
        DataType::Package(_) => quote! {}
    }
}

fn push_unknown_part(ty: &DataType, field_ident: &TokenStream2) -> TokenStream2 {
    match ty {
        DataType::String | DataType::Package(_) => quote! {
            bytes.extend_from_slice(&#field_ident.as_bytes());
        },
        _ => quote! {}
    }
}
