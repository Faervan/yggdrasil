use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, Variant};

use crate::{format_enum_fields::format_enum_variant, parse_field::parse_fields, AcceptedField, DataField, DataType, FieldAccessPush, Length};

pub fn enum_as_bytes(variants: &Vec<&Variant>) -> TokenStream2 {
    let implementation = variants.into_iter().enumerate().fold(quote! {}, |acc, (index, variant)| {
        let (fields, _) = parse_fields(&variant.fields);
        let (push_fixed_bytes, _) = push_fixed_bytes(&fields, FieldAccessPush::Enum);
        let push_unknown_bytes = push_unknown_bytes(&fields, FieldAccessPush::Enum);
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

pub fn push_fixed_bytes(fields: &Vec<AcceptedField>, access: FieldAccessPush) -> (TokenStream2, usize) {
    let mut fixed_buffer_size = 0;
    (fields.iter().fold(quote! {}, |tokens, field| {
        fixed_buffer_size += 1;
        let field_access = &access.as_stream(&field.ident);
        let new_tokens = match &field.data {
            DataField::Option(_) => quote! {
                bytes.push(match #field_access {
                    Some(_) => 1,
                    None => 0
                });
            },
            DataField::Vec(_) => quote! {
                bytes.push(#field_access.len() as u8);
            },
            DataField::HashMap {..} => quote! {
                bytes.push(#field_access.len() as u8);
            },
            DataField::Type(ty) => push_fixed_part(ty, &field.ident, field_access, Some(&mut fixed_buffer_size), &access)
        };
        quote! {
            #tokens
            #new_tokens
        }
    }), fixed_buffer_size)
}

pub fn push_unknown_bytes(fields: &Vec<AcceptedField>, access: FieldAccessPush) -> TokenStream2 {
    fields.iter().fold(quote! {}, |tokens, field| {
        let field_ident = &field.ident;
        let field_access = &access.as_stream(&field.ident);
        let new_tokens = match &field.data {
            DataField::Option(ty) => {
                let push_fixed_part = push_fixed_part(ty, field_access, &quote! {#field_access.as_ref().unwrap()}, None, &access);
                let push_unknown_part = push_unknown_part(ty, field_access, &quote! {#field_access.as_ref().unwrap()});
                quote! {
                    if let Some(_) = #field_access {
                        #push_fixed_part
                        #push_unknown_part
                    }
                }
            }
            DataField::Vec(ty) => {
                let push_fixed_part = push_fixed_part(ty, field_access, &quote! {#field_access[i]}, None, &access);
                let push_unknown_part = push_unknown_part(ty, field_access, &quote! {#field_access[i]});
                quote! {
                    let vec_len = 0..#field_access.len();
                    for i in vec_len.clone() {
                        #push_fixed_part
                    }
                    for i in vec_len {
                        #push_unknown_part
                    }
                }
            }
            DataField::HashMap {key, value} => {
                let fixed_part_key = push_fixed_part(key, field_ident, &quote! {k}, None, &access);
                let unknown_part_key = push_unknown_part(key, field_ident, &quote! {k});
                let fixed_part_value = push_fixed_part(value, field_ident, &quote! {v}, None, &access);
                let unknown_part_value = push_unknown_part(value, field_ident, &quote! {v});
                quote! {
                    for (k, v) in #field_access.iter() {
                        #fixed_part_key
                        #unknown_part_key
                        #fixed_part_value
                        #unknown_part_value
                    }
                }
            }
            DataField::Type(ty) => push_unknown_part(ty, field_ident, field_access)
        };
        quote! {
            #tokens
            #new_tokens
        }
    })
}

fn push_fixed_part(ty: &DataType, field_ident: &TokenStream2, field_access: &TokenStream2, fixed_buffer_size: Option<&mut usize>, access: &FieldAccessPush) -> TokenStream2 {
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
                FieldAccessPush::Enum => quote! {bytes.push(*#field_access);},
                FieldAccessPush::Struct => quote! {bytes.push(#field_access);}
            }
        }
        DataType::Bool => quote! {
            bytes.push(match #field_access {
                true => 1,
                false => 0
            });
        },
        DataType::String(length) => {
            let len_ident = length.as_ident();
            match length {
                Length::U8 => quote! {bytes.push(#field_access.len() as #len_ident);},
                Length::U16 => quote! {bytes.extend_from_slice(&(#field_access.len() as #len_ident).to_ne_bytes());}
            }
        }
        DataType::Int(..) => quote! {
            bytes.extend_from_slice(&#field_access.to_ne_bytes());
        },
        DataType::Package(_) => {
            let pkg_ident = Ident::new(format!("bytes_{}", field_ident.to_string()).as_str(), Span::call_site());
            quote! {
                let #pkg_ident = #field_access.as_bytes();
                bytes.push(#pkg_ident.len() as u8);
            }
        }
    }
}

fn push_unknown_part(ty: &DataType, field_ident: &TokenStream2, field_access: &TokenStream2) -> TokenStream2 {
    match ty {
        DataType::String(_) => quote! {
            bytes.extend_from_slice(&#field_access.as_bytes());
        },
        DataType::Package(_) => {
            let pkg_ident = Ident::new(format!("bytes_{}", field_ident.to_string()).as_str(), Span::call_site());
            quote! {
                bytes.extend_from_slice(&#pkg_ident);
            }
        }
        _ => quote! {}
    }
}
