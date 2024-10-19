use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Ident, Variant};

use crate::{format_enum_fields::{format_enum_variant, init_enum_fields}, parse_field::parse_fields, AcceptedField, DataField, DataType, FieldAccessPull, Length};

pub fn enum_from_buf(variants: &Vec<&Variant>) -> TokenStream2 {
    let implementation = variants.into_iter().enumerate().fold(quote! {}, |acc, (index, variant)| {
        let (fields, _) = parse_fields(&variant.fields);
        let init_fields = init_enum_fields(&fields);
        let field_impl = from_buf(&fields, FieldAccessPull::Enum);
        let formatted_variant = format_enum_variant(*variant);
        quote! {
            #acc
            #index => {
                #init_fields
                #field_impl
                Ok(Self::#formatted_variant)
            }
        }
    });
    quote! {
        match buf[0] as usize {
            #implementation
            _ => Err("Given variant index was invalid")
        }
    }
}

pub fn from_buf(fields: &Vec<AcceptedField>, access: FieldAccessPull) -> TokenStream2 {
    let mut buf_index = match access {
        FieldAccessPull::Struct => 0,
        FieldAccessPull::Enum => 1
    };
    let read_fixed_bytes = read_fixed_bytes(fields, &mut buf_index, &access);
    let read_unknown_bytes = read_unknown_bytes(fields, &access);
    quote! {
        #read_fixed_bytes
        let mut buf_index = #buf_index;
        #read_unknown_bytes
    }
}

fn read_fixed_bytes(fields: &Vec<AcceptedField>, buf_index: &mut usize, access: &FieldAccessPull) -> TokenStream2 {
    fields.iter().fold(quote! {}, |tokens, field| {
        *buf_index += 1;
        let field_access = &access.as_stream(&field.ident);
        let field_ident = &field.ident;
        let new_tokens = match &field.data {
            DataField::Option(_) => {
                let option_ident = Ident::new(format!("option_for_{}", field_ident.to_string()).as_str(), Span::call_site());
                quote! {
                    let #option_ident = match buf[#buf_index-1] {
                        1 => true,
                        _ => {
                            #field_access = None;
                            false
                        }
                    };
                }
            }
            DataField::Vec(_) => {
                let vec_ident = Ident::new(format!("len_of_{}", field_ident.to_string()).as_str(), Span::call_site()); quote! {
                    let #vec_ident = buf[#buf_index-1] as usize;
                }
            }
            DataField::HashMap {..} => {
                let map_ident = Ident::new(format!("len_of_{}", field_ident.to_string()).as_str(), Span::call_site()); quote! {
                    let #map_ident = buf[#buf_index-1] as usize;
                }
            }
            DataField::Type(ty) => read_fixed_part(ty, field_ident, field_access, buf_index)
        };
        quote! {
            #tokens
            #new_tokens
        }
    })
}

fn read_unknown_bytes(fields: &Vec<AcceptedField>, access: &FieldAccessPull) -> TokenStream2 {
    fields.iter().fold(quote! {}, |tokens, field| {
        let field_access = &access.as_stream(&field.ident);
        let field_ident = &field.ident;
        let new_tokens = match &field.data {
            DataField::Option(ty) => {
                let option_ident = Ident::new(format!("option_for_{}", field_ident.to_string()).as_str(), Span::call_site());
                let type_impl = get_wrapped_ty_impl(ty);
                quote! {
                    if #option_ident {
                        #field_access = Some({#type_impl});
                    }
                }
            }
            DataField::Vec(ty) => {
                let vec_ident = Ident::new(format!("len_of_{}", field_ident.to_string()).as_str(), Span::call_site());
                let type_impl = get_wrapped_ty_impl(ty);
                quote! {
                    for _ in 0..#vec_ident {
                        #field_access.push({#type_impl});
                    }
                }
            }
            DataField::HashMap { key, value } => {
                let map_ident = Ident::new(format!("len_of_{}", field_ident.to_string()).as_str(), Span::call_site());
                let key_impl = get_wrapped_ty_impl(key);
                let value_impl = get_wrapped_ty_impl(value);
                quote! {
                    for _ in 0..#map_ident {
                        #field_access.insert({#key_impl}, {#value_impl});
                    }
                }
            }
            DataField::Type(ty) => {
                let unknown_part = read_unknown_part(ty, field_ident, field_access);
                quote! {
                    #unknown_part;
                }
            }
        };
        quote! {
            #tokens
            #new_tokens
        }
    })
}

fn read_fixed_part(ty: &DataType, field_ident: &TokenStream2, field_access: &TokenStream2, buf_index: &mut usize) -> TokenStream2 {
    *buf_index -= 1;
    let result = match ty {
        DataType::U8 => quote! {
            #field_access = buf[#buf_index];
        },
        DataType::Bool => quote! {
            #field_access = match buf[#buf_index] {
                1 => true,
                _ => false
            };
        },
        DataType::String(length) => {
            let string_ident = Ident::new(format!("len_of_{}", field_ident.to_string()).as_str(), Span::call_site());
            let string_len = match length {
                Length::U8 => quote!{buf[#buf_index]},
                Length::U16 => {
                    *buf_index += 1;
                    quote! {u16::from_ne_bytes(buf[#buf_index-1..#buf_index+1].try_into().unwrap())}
                }
            };
            quote! {
                let #string_ident = #string_len as usize;
            }
        },
        DataType::Int(int_ident, size) => quote! {
            #field_access = #int_ident::from_ne_bytes(buf[#buf_index..#size+#buf_index].try_into().unwrap());
        },
        DataType::Package(_) => {
            let pkg_ident = Ident::new(format!("pkg_len_{}", field_ident.to_string()).as_str(), Span::call_site());
            quote! {
                let #pkg_ident = u32::from_ne_bytes(buf[#buf_index..#buf_index+4].try_into().unwrap()) as usize;
            }
        }
    };
    *buf_index += match ty {
        DataType::Int(.., int_size) => *int_size,
        DataType::Package(_) => 4,
        _ => 1
    };
    result
}

fn read_unknown_part(ty: &DataType, field_ident: &TokenStream2, field_access: &TokenStream2) -> TokenStream2 {
    match ty {
        DataType::String(_) => {
            let string_ident = Ident::new(format!("len_of_{}", field_ident.to_string()).as_str(), Span::call_site());
            quote! {
                #field_access = String::from_utf8_lossy(&buf[buf_index..#string_ident+buf_index]).to_string();
                buf_index += #string_ident;
            }
        },
        DataType::Package(ty) => {
            let pkg_ident = Ident::new(format!("pkg_len_{}", field_ident.to_string()).as_str(), Span::call_site());
            quote! {
                #field_access = #ty::from_buf(&buf[buf_index..#pkg_ident+buf_index]).unwrap();
                buf_index += #pkg_ident;
            }
        }
        _ => quote! {}
    }
}

fn get_wrapped_ty_impl(ty: &DataType) -> TokenStream2 {
    match ty {
        DataType::U8 => quote! {
            let x = buf[buf_index];
            buf_index += 1;
            x
        },
        DataType::Bool => quote! {
            let x = match buf[buf_index] {
                1 => true,
                _ => false
            };
            buf_index += 1;
            x
        },
        DataType::String(length) => {
            let string_len = match length {
                Length::U8 => quote!{buf[buf_index]},
                Length::U16 => quote! {{
                    buf_index += 1;
                    u16::from_ne_bytes(buf[buf_index-1..buf_index+1].try_into().unwrap())
                }}
            };
            quote! {
                let len = #string_len as usize;
                buf_index += 1;
                let x = String::from_utf8_lossy(&buf[buf_index..len+buf_index]).to_string();
                buf_index += len;
                x
            }
        }
        DataType::Int(int_ident, size) => quote! {
            let x = #int_ident::from_ne_bytes(buf[buf_index..#size+buf_index].try_into().unwrap());
            buf_index += #size;
            x
        },
        DataType::Package(ty_ident) => quote! {
            let len = u32::from_ne_bytes(buf[buf_index..buf_index+4].try_into().unwrap()) as usize;
            buf_index += 4;
            let x = #ty_ident::from_buf(&buf[buf_index..len+buf_index]).unwrap();
            buf_index += len;
            x
        },
    }
}
