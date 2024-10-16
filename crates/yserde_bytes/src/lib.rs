use as_bytes::{enum_as_bytes, push_fixed_bytes, push_unknown_bytes};
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


#[proc_macro_derive(AsBytes, attributes(yignore))]
pub fn as_bytes_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    println!("\n\nHandling type {}\n\n", ident);
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
    let stream = quote! {
        #max_size_macro
        impl #ident {
            #implementation
        }
    }.into();
    println!("stream: {stream}"); stream
}

fn build_enum_impl(variants: Vec<&Variant>) -> TokenStream2 {
    let size_impl = size_from_variants(&variants);
    let push_bytes = enum_as_bytes(&variants);
    quote! {
        #[allow(unused_comparisons)]
        const MAX_SIZE: usize = #size_impl;
        fn as_bytes(&self) -> Vec<u8> {
            let mut bytes = vec![];
            #push_bytes
            bytes
        }
        fn from_buf(buf: &[u8]) -> Self {Self::default()}
    }
}

fn build_struct_impl(fields: Fields) -> TokenStream2 {
    let (fields, _) = parse_fields(&fields);
    let size_impl = size_from_fields(&fields);
    let (push_fixed_bytes, _) = push_fixed_bytes(&fields, FieldAccess::Struct);
    let push_unknown_bytes = push_unknown_bytes(&fields, FieldAccess::Struct);
    quote! {
        const MAX_SIZE: usize = #size_impl;
        fn as_bytes(&self) -> Vec<u8> {
            let mut bytes = vec![];
            #push_fixed_bytes
            #push_unknown_bytes
            bytes
        }
        fn from_buf(buf: &[u8]) -> Self {Self::default()}
    }
}

#[derive(Debug)]
struct AcceptedField {
    ident: TokenStream2,
    data: DataField,
}

#[derive(Debug)]
enum DataField {
    Type(DataType),
    Vec(DataType),
    Option(DataType)
}
#[derive(Debug)]
enum DataType {
    // u8 is a byte, so it doesn't need conversion like the other ints
    U8,
    Int(Ident, usize),
    Bool,
    String,
    Package(Ident),
}

enum FieldAccess {
    Struct,
    Enum
}

impl FieldAccess {
    fn as_stream(&self, ident: &TokenStream2) -> TokenStream2 {
        match self {
            FieldAccess::Enum => {
                let ident = Ident::new(format!("field_{}",
                    ident.to_string()).as_str(),
                    Span::call_site()
                );
                quote! {#ident}
            }
            FieldAccess::Struct => quote! {self.#ident}
        }
    }
}
