use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, GenericArgument, Ident, Index, Meta, PathArguments, Type};

const INT_PRIMITIVES: [&str; 14] = ["u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64"];
const INT_BYTE_SIZES: [usize; 14] = [1, 2, 4, 8, 16, std::mem::size_of::<usize>(), 1, 2, 4, 8, 16, std::mem::size_of::<isize>(), 4, 8];

#[proc_macro_derive(Package, attributes(yserde_ignore))]
pub fn package_derive(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let ident = input.ident;
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
                fn from_tcp(&self, socket: &mut std::net::TcpStream) -> std::io::Result<Box<dyn Any>> {
                    Ok(Box::new(#ident))
                }
                fn from_async_tcp(&self, socket: &mut tokio::net::TcpStream) -> tokio::io::Result<Box<dyn Any>> {
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

    let fields: Vec<(TokenStream2, DataType)> = fields.into_iter().filter(|f| {
        for attr in f.attrs.iter() {
            if let Meta::Path(path) = &attr.meta {
                if let Some(ident) = path.get_ident() {
                    if ident.to_string().as_str() == "yserde_ignore" {
                        return false;
                    }
                }
            }
        }
        true
    }).enumerate().map(|(index, field)| {
        match field.ty {
            Type::Path(ty_path) => {
                let path = ty_path.path.segments.first().unwrap_or_else(|| unreachable!("path has no segments?"));
                (
                    match is_named {
                         true => {
                             let ident = field.ident.unwrap_or_else(|| unreachable!("named field has no identifier?"));
                             quote! {#ident}
                         },
                         false => {
                             let index = Index::from(index);
                             quote! {#index}
                         }
                     },
                     {
                         match &path.arguments {
                             PathArguments::None => DataType::Normal(path.ident.clone()),
                             PathArguments::AngleBracketed(generic_args) => {
                                 let sub_type = match generic_args.args.first().unwrap_or_else(|| unreachable!("There has to be some arg...?")) {
                                     GenericArgument::Type(ty) => match ty {
                                         Type::Path(ty) => ty.path.require_ident().unwrap_or_else(|e| panic!("8463: {e}")),
                                         _ => panic!("Path con only consist of one basic segment (like 'u16')")
                                     }
                                     _ => panic!("Argument has to be a normal type like 'u16'")
                                 };
                                 match path.ident.to_string().as_str() {
                                     "Vec" => DataType::Vec(sub_type.clone()),
                                     "Option" => DataType::Option(sub_type.clone()),
                                     _ => panic!("Only supported containers are Vec and Option")
                                 }
                             },
                             PathArguments::Parenthesized(_) => panic!("Tuple subtypes are not yet implemented")
                         }
                     }
                )
            }
            _ => panic!("Field has an unsupported type: {}", field.ty.to_token_stream())
        }
    }).collect();

    let (as_bytes, from_bytes) = get_conversions(&fields);

    quote! {
        impl Package for #ident {
            fn get_new(&self) -> Box<dyn Package> {
                Box::new(#ident::default())
            }
            fn as_bytes(&self) -> Vec<u8> {
                let mut bytes = vec![];
                #as_bytes
                bytes
            }
            fn from_tcp(&self, socket: &mut std::net::TcpStream) -> std::io::Result<Box<dyn Any>> {
                Ok(Box::new(#ident::default()))
            }
            fn from_async_tcp(&self, socket: &mut tokio::net::TcpStream) -> tokio::io::Result<Box<dyn Any>> {
                let mut pkg = #ident::default();
                let mut buf = [0; 1];
                #from_bytes
                Ok(Box::new(pkg))
            }
        }
    }.into()
}

enum DataType {
    Normal(Ident),
    Vec(Ident),
    Option(Ident)
}

fn get_conversions(fields: &Vec<(TokenStream2, DataType)>) -> (TokenStream2, TokenStream2) {
    fields.iter().fold((quote! {}, quote! {}), |(ser, de), (ident, ty)| {
        let (to_bytes, from_bytes) = match ty {
            DataType::Normal(ty) => check_type(
                ty.to_string().as_str(),
                ident,
                false,
                quote! {}
            ).unwrap_or((quote! {}, quote! {})),
            DataType::Vec(ty) => {
                let (push_value, pull_value) = check_type(ty.to_string().as_str(), ident, true, quote! {[i]})
                    .unwrap_or_else(|| panic!("Vector holds an unsupported type (annotate with #[yserde_ignore] to ignore)"));
                (quote! {
                    bytes.push(self.#ident.len() as u8);
                    for i in 0..self.#ident.len() {
                        #push_value
                    }
                }, quote! {
                    socket.try_read(&mut buf)?;
                    for _ in 0..buf[0].into() {
                        pkg.#ident.push(#pull_value);
                    }
                })
            }
            DataType::Option(ty) => {
                let (push_value, pull_value) = check_type(ty.to_string().as_str(), ident, true, quote! {.as_ref().unwrap()})
                    .unwrap_or_else(|| panic!("Option holds an unsupported type (annotate with #[yserde_ignore] to ignore)"));
                (quote! {
                    match &self.#ident {
                        Some(value) => {
                            bytes.push(1);
                            #push_value
                        }
                        None => bytes.push(0)
                    }
                }, quote! {
                    socket.try_read(&mut buf)?;
                    pkg.#ident = match buf[0] {
                        1 => Some(#pull_value),
                        _ => None
                    };
                })
            }
        };
        (
            quote! {
                #ser
                #to_bytes
            }, quote! {
                #de
                #from_bytes
            }
        )
    })
}

fn check_type(ty: &str, ident: &TokenStream2, get_raw_value: bool, precision: TokenStream2) -> Option<(TokenStream2, TokenStream2)> {
    let (ser, de) = match ty {
        "bool" => {
            bool_conversion(ident, precision)
        }
        "String" => {
            string_conversion(ident, precision)
        }
        "char" => {
            println!("\n\x1b[93mWARNING\x1b[0m: The char type is not (yet) supported. \n\t Annotate field '\x1b[95m{ident}\x1b[0m' with \x1b[95m#[yserde_ignore]\x1b[0m to disable this warning.\n");
            return None;
        }
        _ => {
            if let Some((index, ty)) = INT_PRIMITIVES.iter().enumerate().find(|(_, i)| **i == ty.to_string().as_str()) {
                int_conversion(index, ty, ident, precision)
            } else {
                package_type_conversion(ty, ident, precision)
            }
        }
    };
    match get_raw_value {
        true => Some((ser, de)),
        false => Some((
            ser,
            quote! {pkg.#ident = #de;}
        ))
    }
}

fn bool_conversion(ident: &TokenStream2, precision: TokenStream2) -> (TokenStream2, TokenStream2) {
    (
        quote! {
            bytes.push(match &self.#ident #precision {
                true => 1,
                false => 0
            });
        },
        quote! {{
            socket.try_read(&mut buf)?;
            match buf[0] {
                1 => true,
                _ => false
            }
        }}
    )
}

fn string_conversion(ident: &TokenStream2, precision: TokenStream2) -> (TokenStream2, TokenStream2) {
    (
        quote! {
            bytes.push(self.#ident #precision.len() as u8);
            bytes.extend_from_slice(&self.#ident #precision.as_bytes());
        },
        quote! {{
            socket.try_read(&mut buf)?;
            let mut string_buf: Vec<u8> = vec![0; buf[0].into()];
            socket.try_read(&mut string_buf)?;
            String::from_utf8_lossy(&string_buf).to_string()
        }}
    )
}

fn int_conversion(index: usize, ty: &str, ident: &TokenStream2, precision: TokenStream2) -> (TokenStream2, TokenStream2) {
    let byte_size = INT_BYTE_SIZES[index];
    let int_ident = Ident::new(ty, Span::call_site());
    let int = match ty {
        "u8" => quote! {buf[0]},
        _ => quote! {#int_ident::from_ne_bytes(buf)}
    };
    (
        quote! {
            bytes.extend_from_slice(&self.#ident #precision.to_ne_bytes());
        },
        quote! {{
            let mut buf = [0; #byte_size];
            socket.try_read(&mut buf)?;
            #int
        }}
    )
}

fn package_type_conversion(ty: &str, ident: &TokenStream2, precision: TokenStream2) -> (TokenStream2, TokenStream2) {
    let ty_ident = Ident::new(ty, Span::call_site());
    (
        quote! {
            bytes.extend_from_slice(&self.#ident #precision.as_bytes());
        },
        quote! {
            *#ty_ident::default().from_async_tcp(socket)?.downcast::<#ty_ident>().unwrap()
        }
    )
}
