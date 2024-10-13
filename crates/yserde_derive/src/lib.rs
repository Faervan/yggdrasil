use as_bytes::{push_fixed_bytes, push_unknown_bytes};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields, GenericArgument, Ident, Index, Meta, PathArguments, Type};

const INT_PRIMITIVES: [&str; 13] = ["u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64"];
const INT_BYTE_SIZES: [usize; 13] = [2, 4, 8, 16, std::mem::size_of::<usize>(), 1, 2, 4, 8, 16, std::mem::size_of::<isize>(), 4, 8];

mod as_bytes;

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

    let fields: Vec<AcceptedField> = fields.into_iter().filter(|f| {
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
    }).enumerate().fold(vec![], |mut fields, (index, field)| {
        match field.ty {
            Type::Path(ty_path) => {
                let path = ty_path.path.segments.first().unwrap_or_else(|| unreachable!("path has no segments?"));
                let field_ident = match is_named {
                    true => {
                        let ident = field.ident.unwrap_or_else(|| unreachable!("named field has no identifier?"));
                        quote! {#ident}
                    }
                    false => {
                        let index = Index::from(index);
                        quote! {#index}
                    }
                };
                if let Some(field) = get_field(&field_ident, path.ident.clone(), path.arguments.clone()) {
                    fields.push(field);
                }
                fields
            }
            _ => panic!("Field has an unsupported type: {}", field.ty.to_token_stream())
        }
    });

    let (push_fixed_bytes, fixed_buffer_size) = push_fixed_bytes(&fields);
    let push_unknown_bytes = push_unknown_bytes(&fields);

    let stream = quote! {
        impl Package for #ident {
            fn get_new(&self) -> Box<dyn Package> {
                Box::new(#ident::default())
            }
            fn as_bytes(&self) -> Vec<u8> {
                let mut bytes = vec![];
                #push_fixed_bytes
                #push_unknown_bytes
                bytes
            }
            fn from_tcp(&self, socket: &mut std::net::TcpStream) -> std::io::Result<Box<dyn Any>> {
                let mut pkg = #ident::default();
                let mut buf = [0; #fixed_buffer_size];
                Ok(Box::new(#ident::default()))
            }
            fn from_async_tcp(&self, socket: &mut tokio::net::TcpStream) -> tokio::io::Result<Box<dyn Any>> {
                let mut pkg = #ident::default();
                let mut buf = [0; #fixed_buffer_size];
                Ok(Box::new(pkg))
            }
        }
    }.into();
    println!("stream: {stream}");
    stream
}

struct AcceptedField {
    ident: TokenStream2,
    data: DataField,
}

enum DataField {
    Type(DataType),
    Vec(DataType),
    Option(DataType)
}
enum DataType {
    // u8 is a byte, so it doesn't need conversion like the other ints
    U8,
    Int(Ident, usize),
    Bool,
    String,
    Package(Ident),
}

fn get_field(field_ident: &TokenStream2, data_ident: Ident, path_args: PathArguments) -> Option<AcceptedField> {
    let field = match path_args {
        PathArguments::None => {
            match verify_data_type(data_ident) {
                None => return None,
                Some(ty) => AcceptedField { ident: field_ident.clone(), data: DataField::Type(ty) }
            }
        }
        PathArguments::AngleBracketed(generic_args) => {
            let sub_type = match generic_args.args.first().unwrap_or_else(|| unreachable!("There has to be some arg...?")) {
                GenericArgument::Type(ty) => match ty {
                    Type::Path(ty) => ty.path.require_ident().unwrap_or_else(|e| panic!("8463: {e}")).clone(),
                    _ => panic!("Path con only consist of one basic segment (like 'u16')")
                }
                _ => panic!("Argument has to be a normal type like 'u16'")
            };
            let container = data_ident.to_string();
            let container = container.as_str();
            if container == "Vec" || container == "Option" {
                match verify_data_type(sub_type) {
                    None => return None,
                    Some(ty) => AcceptedField {
                        ident: field_ident.clone(),
                        data: match container {
                            "Vec" => DataField::Vec(ty),
                            "Option" => DataField::Option(ty),
                            _ => unreachable!("huh")
                        }
                    }
                }
            } else {panic!("Only supported containers are Vec and Option")}
        },
        PathArguments::Parenthesized(_) => panic!("Tuple subtypes are not yet implemented")
    };
    Some(field)
}

fn verify_data_type(ident: Ident) -> Option<DataType> {
    let ty = match ident.to_string().as_str() {
        "bool" => DataType::Bool,
        "String" => DataType::String,
        "u8" => DataType::U8,
        "char" => return None,
        int if INT_PRIMITIVES.contains(&int) => {
                let (index, _) = INT_PRIMITIVES.iter().enumerate().find(|(_, i)| **i == int)
                    .unwrap_or_else(|| unreachable!("..."));
                DataType::Int(ident, INT_BYTE_SIZES[index])
        }
        _ => DataType::Package(ident)
    };
    Some(ty)
}

/*fn get_conversions(fields: &Vec<(TokenStream2, DataType)>, buffer_length: &mut usize) -> Vec<AcceptedField> {
    fields.iter().fold(vec![], |fields, (ident, ty)| {
        let new_tokens = match ty {
            DataType::Normal(ty) => check_type(
                ty.to_string().as_str(),
                ident,
                buffer_length,
                false,
                quote! {}
            ).unwrap_or(TokenMap::default()),
            DataType::Vec(ty) => {
                let (push_value, pull_value) = check_type(ty.to_string().as_str(), ident, buffer_length, true, quote! {[i]})
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
                let (push_value, pull_value) = check_type(ty.to_string().as_str(), ident, buffer_length, true, quote! {.as_ref().unwrap()})
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
        token_map + new_tokens
    })
}

fn check_type(ty: &str, ident: &TokenStream2, buffer_length: &mut usize, get_raw_value: bool, precision: TokenStream2) -> Option<TokenMap> {
    let (ser, de) = match ty {
        "bool" => {
            bool_conversion(ident, buffer_length, precision)
        }
        "String" => {
            string_conversion(ident, buffer_length, precision)
        }
        "char" => {
            println!("\n\x1b[93mWARNING\x1b[0m: The char type is not (yet) supported. \n\t Annotate field '\x1b[95m{ident}\x1b[0m' with \x1b[95m#[yserde_ignore]\x1b[0m to disable this warning.\n");
            return None;
        }
        _ => {
            if let Some((index, ty)) = INT_PRIMITIVES.iter().enumerate().find(|(_, i)| **i == ty.to_string().as_str()) {
                int_conversion(index, ty, ident, buffer_length, precision)
            } else {
                package_type_conversion(ty, ident, buffer_length, precision)
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

fn bool_conversion(ident: &TokenStream2, buffer_length: &mut usize, precision: TokenStream2) -> TokenMap {
    let index = buffer_length;
    *buffer_length += 1;
    TokenMap {
        as_bytes: quote! {
            bytes.push(match &self.#ident #precision {
                true => 1,
                false => 0
            });
        },
        from_tcp: quote! {
            *#ty_ident::default().from_tcp(socket)?.downcast::<#ty_ident>().unwrap()
        },
        from_async_tcp: quote! {
            match buf[#index] {
                1 => true,
                _ => false
            }
        }
    }
}

fn string_conversion(ident: &TokenStream2, buffer_length: &mut usize, precision: TokenStream2) -> TokenMap {
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
    TokenMap {
        as_bytes: quote! {
            bytes.extend_from_slice(&self.#ident #precision.as_bytes());
        },
        from_tcp: quote! {
            *#ty_ident::default().from_tcp(socket)?.downcast::<#ty_ident>().unwrap()
        },
        from_async_tcp: quote! {
            *#ty_ident::default().from_async_tcp(socket)?.downcast::<#ty_ident>().unwrap()
        }
    }
}

fn int_conversion(index: usize, ty: &str, ident: &TokenStream2, buffer_length: &mut usize, precision: TokenStream2) -> TokenMap {
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
    TokenMap {
        as_bytes: quote! {
            bytes.extend_from_slice(&self.#ident #precision.as_bytes());
        },
        from_tcp: quote! {
            *#ty_ident::default().from_tcp(socket)?.downcast::<#ty_ident>().unwrap()
        },
        from_async_tcp: quote! {
            *#ty_ident::default().from_async_tcp(socket)?.downcast::<#ty_ident>().unwrap()
        }
    }
}

fn package_type_conversion(ty: &str, ident: &TokenStream2, buffer_length: &mut usize, precision: TokenStream2) -> TokenMap {
    let ty_ident = Ident::new(ty, Span::call_site());
    TokenMap {
        as_bytes: quote! {
            bytes.extend_from_slice(&self.#ident #precision.as_bytes());
        },
        from_tcp: quote! {
            *#ty_ident::default().from_tcp(socket)?.downcast::<#ty_ident>().unwrap()
        },
        from_async_tcp: quote! {
            *#ty_ident::default().from_async_tcp(socket)?.downcast::<#ty_ident>().unwrap()
        }
    }
}*/
