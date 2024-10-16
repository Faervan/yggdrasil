use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::AcceptedField;

pub fn from_buf(fields: &Vec<&AcceptedField>) -> TokenStream2 {
    quote! {}
}
