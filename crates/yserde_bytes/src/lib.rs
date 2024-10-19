use as_bytes::{enum_as_bytes, push_fixed_bytes, push_unknown_bytes};
use from_buf::{enum_from_buf, from_buf};
use get_size::{size_from_fields, size_from_variants};
use parse_field::parse_fields;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, parse_str, Data, DeriveInput, Fields, Ident, Variant};

const INT_PRIMITIVES: [&str; 13] = ["u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64"];
const INT_BYTE_SIZES: [usize; 13] = [2, 4, 8, 16, std::mem::size_of::<usize>(), 1, 2, 4, 8, 16, std::mem::size_of::<isize>(), 4, 8];

const MAX_SIZE_MACRO: &str = r#"
    macro_rules! max_size {
        () => { 0 };
        ($head:expr) => { $head };
        ($head:expr, $($tail:expr),+) => {
            if $head > max_size!($($tail),+) {
                $head
            } else {
                max_size!($($tail),+)
            }
        };
    }
"#;

mod parse_field;
mod get_size;
mod as_bytes;
mod format_enum_fields;
mod from_buf;


#[proc_macro_derive(AsBytes, attributes(yignore, u16))]
pub fn as_bytes_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let (implementation, max_size_macro) = match input.data {
        Data::Enum(data) => {
            (build_enum_impl(data.variants.iter().collect()),
            parse_str(MAX_SIZE_MACRO).unwrap())
        }
        Data::Struct(data) => {
            (build_struct_impl(data.fields), quote! {})
        }
        _ => panic!("Currently only Enums and Structs can use this derive")
    };
    let x = quote! {
        #max_size_macro
        impl #ident {
            #implementation
        }
    }.into();
    println!("{x}");x
}

fn build_enum_impl(variants: Vec<&Variant>) -> TokenStream2 {
    let size_impl = size_from_variants(&variants);
    let push_bytes = enum_as_bytes(&variants);
    let from_buf = enum_from_buf(&variants);
    quote! {
        #[allow(unused_comparisons)]
        const MAX_SIZE: usize = #size_impl;
        fn as_bytes(&self) -> Vec<u8> {
            let bytes_uncounted = self.as_bytes_uncounted();
            let mut bytes = (bytes_uncounted.len() as u32).to_ne_bytes().to_vec();
            bytes.extend(bytes_uncounted);
            bytes
        }
        fn as_bytes_uncounted(&self) -> Vec<u8> {
            let mut bytes = vec![];
            #push_bytes
            bytes
        }
        fn from_buf(buf: &[u8]) -> Result<Self, &str> {
            #from_buf
        }
    }
}

fn build_struct_impl(fields: Fields) -> TokenStream2 {
    let (fields, _) = parse_fields(&fields);
    let size_impl = size_from_fields(&fields);
    let (push_fixed_bytes, _) = push_fixed_bytes(&fields, FieldAccessPush::Struct);
    let push_unknown_bytes = push_unknown_bytes(&fields, FieldAccessPush::Struct);
    let from_buf = from_buf(&fields, FieldAccessPull::Struct);
    quote! {
        const MAX_SIZE: usize = #size_impl;
        fn as_bytes(&self) -> Vec<u8> {
            let bytes_uncounted = self.as_bytes_uncounted();
            let mut bytes = (bytes_uncounted.len() as u32).to_ne_bytes().to_vec();
            bytes.extend(bytes_uncounted);
            bytes
        }
        fn as_bytes_uncounted(&self) -> Vec<u8> {
            let mut bytes = vec![];
            #push_fixed_bytes
            #push_unknown_bytes
            bytes
        }
        fn from_buf(buf: &[u8]) -> Result<Self, &str> {
            let mut pkg = Self::default();
            #from_buf
            Ok(pkg)
        }
    }
}

#[derive(Debug)]
struct AcceptedField {
    ident: TokenStream2,
    data: DataField,
}

impl AcceptedField {
    fn data_type(&self) -> TokenStream2 {
        match &self.data {
            DataField::Vec(_) => quote! {Vec},
            DataField::HashMap {..} => quote! {HashMap},
            DataField::Option(_) => quote! {Option},
            DataField::Type(ty) => match ty {
                DataType::U8 => quote! {u8},
                DataType::Bool => quote! {bool},
                DataType::String(_) => quote! {String},
                DataType::Int(ident, _) => quote! {#ident},
                DataType::Package(ident) => quote! {#ident}
            }
        }
    }
}

#[derive(Debug)]
enum DataField {
    Type(DataType),
    Vec(DataType),
    HashMap {
        key: DataType,
        value: DataType
    },
    Option(DataType)
}
#[derive(Debug)]
enum DataType {
    // u8 is a byte, so it doesn't need conversion like the other ints
    U8,
    Int(Ident, usize),
    Bool,
    String(Length),
    Package(Ident),
}

#[derive(Debug, Clone, Copy)]
enum Length {
    U8,
    U16
}

impl Length {
    fn as_ident(&self) -> Ident {
        match self {
            Length::U8 => Ident::new("u8", Span::call_site()),
            Length::U16 => Ident::new("u16", Span::call_site()),
        }
    }
    fn as_size(&self) -> usize {
        match self {
            Length::U8 => 255,
            Length::U16 => 65535
        }
    }
}

enum FieldAccessPush {
    Struct,
    Enum
}

impl FieldAccessPush {
    fn as_stream(&self, ident: &TokenStream2) -> TokenStream2 {
        match self {
            FieldAccessPush::Enum => {
                let ident = Ident::new(format!("field_{}",
                    ident.to_string()).as_str(),
                    Span::call_site()
                );
                quote! {#ident}
            }
            FieldAccessPush::Struct => quote! {self.#ident}
        }
    }
}

enum FieldAccessPull {
    Struct,
    Enum
}

impl FieldAccessPull {
    fn as_stream(&self, ident: &TokenStream2) -> TokenStream2 {
        match self {
            FieldAccessPull::Enum => {
                let ident = Ident::new(format!("field_{}",
                    ident.to_string()).as_str(),
                    Span::call_site()
                );
                quote! {#ident}
            }
            FieldAccessPull::Struct => quote! {pkg.#ident}
        }
    }
}
