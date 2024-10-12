use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Index, Meta, Type};

const INT_PRIMITIVES: [&str; 14] = ["u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64"];
const INT_BYTE_SIZES: [usize; 14] = [1, 2, 4, 8, 16, std::mem::size_of::<usize>(), 1, 2, 4, 8, 16, std::mem::size_of::<isize>(), 4, 8];

#[proc_macro_derive(Package, attributes(do_not_send))]
pub fn package_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let ident = input.ident;
    println!("ident: {ident}");
    let fields = match input.data {
        Data::Struct(data) => {data}
        _ => panic!("Package can only be derived for structs (as of now)"),
    };
    if let Fields::Unit = fields.fields {
        return quote! {
            impl Package for #ident {
                fn get_new(&self) -> Box<dyn Package> {
                    Box::new(#ident)
                }
                fn from_bytes(&self, _socket: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
                    Ok(Box::new(#ident))
                }
            }
        }.into();
    }
    let (fields, is_named) = match fields.fields {
        Fields::Named(fields) => (fields.named, true),
        Fields::Unnamed(fields) => (fields.unnamed, false),
        _ => unreachable!("Logical error or breaking API change of syn crate")
    };

    let fields: Vec<(TokenStream2, Ident)> = fields.into_iter().filter(|f| {
        for attr in f.attrs.iter() {
            if let Meta::Path(path) = &attr.meta {
                if let Some(ident) = path.get_ident() {
                    if ident.to_string().as_str() == "do_not_send" {
                        return false;
                    }
                }
            }
        }
        true
    }).enumerate().map(|(index, field)| {
        match field.ty {
            Type::Path(ty_path) => {
                (
                    match is_named {
                         true => {
                             let ident = field.ident.unwrap_or_else(|| panic!("Field has to be named"));
                             quote! {#ident}
                         },
                         false => {
                             let index = Index::from(index);
                             quote! {#index}
                         }
                     },
                    ty_path.path.get_ident().unwrap_or_else(|| panic!("Field value can't be a path constisting of multiple segments")).clone()
                )
            }
            _ => panic!("Field has an unsupported type")
        }
    }).collect();
    println!("fields: {fields:#?}");

    let (from_int, to_int) = get_int_primitives(&fields);
    let (from_bool, to_bool) = get_booleans(&fields);
    let (from_string, to_string) = get_strings(&fields);

    let stream = quote! {
        impl Package for #ident {
            fn get_new(&self) -> Box<dyn Package> {
                Box::new(#ident::default())
            }
            fn as_bytes(&self) -> Vec<u8> {
                let mut bytes = vec![];
                #from_int
                #from_bool
                #from_string
                bytes
            }
            fn from_bytes(&self, _socket: &mut TcpStream) -> tokio::io::Result<Box<dyn Any>> {
                let mut pkg = #ident::default();
                #to_int
                #to_bool
                #to_string
                Ok(Box::new(pkg))
            }
        }
    }.into();
    println!("stream: {stream}");
    stream
}

fn get_int_primitives(fields: &Vec<(TokenStream2, Ident)>) -> (TokenStream2, TokenStream2) {
    fields.iter().fold((quote! {}, quote! {}), |(ser, de), (ident, ty)| {
        if let Some((int_index, int)) = INT_PRIMITIVES.iter().enumerate().find(|(_, i)| **i == ty.to_string().as_str()) {
            println!("MATCH!!! {}: {}", ident, ty);
            let byte_size = INT_BYTE_SIZES[int_index];
            let int_ident = Ident::new(*int, Span::call_site());
            let int = match *int {
                "u8" | "i8" => quote! {buf[0]},
                _ => quote! {#int_ident::from_ne_bytes(buf)}
            };
            (
                quote! {
                    #ser
                    bytes.extend_from_slice(&self.#ident.to_ne_bytes());
                }, quote! {
                    #de
                    let mut buf = [0; #byte_size];
                    _socket.try_read(&mut buf)?;
                    pkg.#ident = #int;
                }
            )
        } else {(ser, de)}
    })
}

fn get_booleans(fields: &Vec<(TokenStream2, Ident)>) -> (TokenStream2, TokenStream2) {
    fields.iter().fold((quote! {}, quote! {}), |(ser, de), (ident, ty)| {
        if ty.to_string().as_str() == "bool" {
            println!("MATCH!!! {}: {}", ident, ty);
            (
                quote! {
                    #ser
                    bytes.push(match &self.#ident {
                        true => 1,
                        false => 0
                    });
                }, quote! {
                    #de
                    let mut buf = [0; 1];
                    _socket.try_read(&mut buf)?;
                    pkg.#ident = match buf[0] {
                        1 => true,
                        _ => false
                    };
                }
            )
        } else {(ser, de)}
    })
}

fn get_strings(fields: &Vec<(TokenStream2, Ident)>) -> (TokenStream2, TokenStream2) {
    fields.iter().fold((quote! {}, quote! {}), |(ser, de), (ident, ty)| {
        if ty.to_string().as_str() == "String" {
            println!("MATCH!!! {}: {}", ident, ty);
            (
                quote! {
                    #ser
                    bytes.push(self.#ident.len() as u8);
                    bytes.extend_from_slice(&self.#ident.as_bytes());
                }, quote! {
                    #de
                    let mut buf = [0; 1];
                    _socket.try_read(&mut buf)?;
                    let mut string_buf: Vec<u8> = vec![0; buf[0].into()];
                    _socket.try_read(&mut string_buf)?;
                    pkg.#ident = String::from_utf8_lossy(&string_buf).to_string();
                }
            )
        } else {(ser, de)}
    })
}
