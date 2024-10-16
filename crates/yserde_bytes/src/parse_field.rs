use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{Fields, GenericArgument, Ident, Index, Meta, PathArguments, Type};

use crate::{AcceptedField, DataField, DataType, INT_BYTE_SIZES, INT_PRIMITIVES};

pub fn parse_fields(fields: &Fields) -> (Vec<AcceptedField>, bool) {
    let (fields, is_named) = match fields {
        Fields::Named(fields) => (&fields.named, true),
        Fields::Unnamed(fields) => (&fields.unnamed, false),
        _ => return (vec![], false)
    };
    (fields.into_iter().filter(|f| {
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
        match &field.ty {
            Type::Path(ty_path) => {
                let path = ty_path.path.segments.first().unwrap_or_else(|| unreachable!("path has no segments?"));
                let field_ident = match is_named {
                    true => {
                        let ident = field.ident.as_ref().unwrap_or_else(|| unreachable!("named field has no identifier?"));
                        quote! {#ident}
                    }
                    false => {
                        let index = Index::from(index);
                        quote! {#index}
                    }
                };
                if let Some(field) = parse_field_type(&field_ident, path.ident.clone(), path.arguments.clone()) {
                    fields.push(field);
                }
                fields
            }
            _ => panic!("Field has an unsupported type: {}", field.ty.to_token_stream())
        }
    }), is_named)
}

fn parse_field_type(field_ident: &TokenStream2, data_ident: Ident, path_args: PathArguments) -> Option<AcceptedField> {
    let field = match path_args {
        PathArguments::None => {
            match parse_data_type(data_ident) {
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
                match parse_data_type(sub_type) {
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


fn parse_data_type(ident: Ident) -> Option<DataType> {
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
