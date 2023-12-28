use std::str::FromStr;

use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    Data::{self, Struct},
    DataStruct,
    Fields::{Named, Unit},
    FieldsNamed,
};

pub(crate) struct Metadata {
    pub base_url: Literal,
    pub default: bool,
}

impl From<proc_macro::TokenStream> for Metadata {
    fn from(value: proc_macro::TokenStream) -> Self {
        let mut iter = value.into_iter();
        let base_url = iter.next().unwrap().to_string();
        let default = iter.all(|i| i.to_string() != "no_default");
        Self {
            base_url: Literal::from_str(base_url.as_str()).unwrap(),
            default,
        }
    }
}

pub(crate) fn parse_meta(meta: proc_macro::TokenStream) -> Metadata {
    Metadata::from(meta)
}

pub(crate) fn parse_fields(data: Data) -> (TokenStream, TokenStream, TokenStream) {
    let empty = Punctuated::new();
    let fields = match data {
        Struct(DataStruct {
            fields: Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        Struct(DataStruct { fields: Unit, .. }) => &empty,
        _ => unimplemented!("Only works for structs"),
    };

    let fields_decl = fields.iter().map(|f| {
        quote! {
            #f
        }
    });

    let fields_init = fields.iter().map(|f| {
        let fname = f.ident.clone().unwrap();
        quote! {
            #fname: Default::default()
        }
    });

    let fields_clone = fields.iter().map(|f| {
        let fname = f.ident.clone().unwrap();
        quote! {
            #fname: self.#fname.clone()
        }
    });

    (
        quote! {#(#fields_decl,)*},
        quote! {#(#fields_init,)*},
        quote! {#(#fields_clone,)*},
    )
}
