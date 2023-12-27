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

pub(crate) struct ApiMeta {
    pub base_url: Literal,
}

impl From<proc_macro::TokenStream> for ApiMeta {
    fn from(value: proc_macro::TokenStream) -> Self {
        let base_url = value.into_iter().next().unwrap().to_string();
        Self {
            base_url: Literal::from_str(base_url.as_str()).unwrap(),
        }
    }
}

pub(crate) fn parse_meta(meta: proc_macro::TokenStream) -> ApiMeta {
    ApiMeta::from(meta)
}

pub(crate) fn parse_fields(data: Data) -> (TokenStream, TokenStream) {
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
    (quote! {#(#fields_decl,)*}, quote! {#(#fields_init,)*})
    // let fields_init = quote! {
    //     #(#fields_init,)*
    // };
}
