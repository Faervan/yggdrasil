use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

use crate::{AcceptedField, DataField, DataType};

pub fn read_fixed_bytes(fields: &Vec<AcceptedField>) -> TokenStream2 {
    let mut fixed_byte_index = 0;
    fields.iter().fold(quote! {}, |tokens, field| {
        let field_ident = &field.ident;
        let new_tokens = match &field.data {
            DataField::Option(_) => {
                let option_ident = Ident::new(format!("option_for_{}", field_ident.to_string()).as_str(), Span::call_site());
                quote! {
                    let #option_ident = match buf[#fixed_byte_index] {
                        1 => true,
                        _ => {
                            pkg.#field_ident = None;
                            false
                        }
                    };
                }
            }
            DataField::Vec(_) => {
                let vec_ident = Ident::new(format!("len_of_{}", field_ident.to_string()).as_str(), Span::call_site());
                quote! {
                    let #vec_ident = buf[#fixed_byte_index] as usize;
                }
            }
            DataField::Type(ty) => read_fixed_part(ty, field_ident, &mut fixed_byte_index)
        };
        match &field.data {
            DataField::Type(_) => {}
            _ => fixed_byte_index += 1
        }
        quote! {
            #tokens
            #new_tokens
        }
    })
}

pub fn read_unknown_bytes(fields: &Vec<AcceptedField>, read_method: TokenStream2) -> TokenStream2 {
    fields.iter().fold(quote! {}, |tokens, field| {
        let field_ident = &field.ident;
        let new_tokens = match &field.data {
            DataField::Option(_) => {
                quote! {
                }
            }
            DataField::Vec(_) => {
                quote! {
                }
            }
            DataField::Type(ty) => read_unknown_part(ty, field_ident, &read_method)
        };
        quote! {
            #tokens
            #new_tokens
        }
    })
}

fn read_fixed_part(ty: &DataType, field_ident: &TokenStream2, fixed_byte_index: &mut usize) -> TokenStream2 {
    let result = match ty {
        DataType::U8 => quote! {
            pkg.#field_ident = buf[#fixed_byte_index];
        },
        DataType::Bool => quote! {
            pkg.#field_ident = match buf[#fixed_byte_index] {
                1 => true,
                _ => false
            };
        },
        DataType::String => {
            let string_ident = Ident::new(format!("len_of_{}", field_ident.to_string()).as_str(), Span::call_site());
            quote! {
                let #string_ident = buf[#fixed_byte_index] as usize;
                println!("string_len: {}", #string_ident);
            }
        },
        DataType::Int(int_ident, size) => quote! {
            println!("slice in question: {:?}\nindex: {}\nsize: {}", &buf[#fixed_byte_index..#size], #fixed_byte_index, #size);
            pkg.#field_ident = #int_ident::from_ne_bytes(buf[#fixed_byte_index..#size].try_into().unwrap());
        },
        DataType::Package(_) => quote! {}
    };
    *fixed_byte_index = *fixed_byte_index + match ty {
        DataType::Int(.., int_size) => *int_size,
        DataType::Package(_) => 0,
        _ => 1
    };
    result
}

fn read_unknown_part(ty: &DataType, field_ident: &TokenStream2, read_method: &TokenStream2) -> TokenStream2 {
    match ty {
        DataType::String => {
            let string_ident = Ident::new(format!("len_of_{}", field_ident.to_string()).as_str(), Span::call_site());
            quote! {
                let mut buf = vec![0; #string_ident];
                socket.#read_method(&mut buf)?;
                pkg.#field_ident = String::from_utf8_lossy(&buf).to_string();
            }
        },
        DataType::Package(_) => quote! {},
        _ => quote! {}
    }
}
