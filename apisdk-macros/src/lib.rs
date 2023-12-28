//! A highlevel API client framework for Rust.
//! This crate is an internal used crate, please check `apisdk` crate for more details.

use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, ItemFn, Meta};

mod build;
mod parse;

use crate::build::{build_api_impl, build_api_methods, build_builder, build_macro_overrides};
use crate::parse::parse_fields;

/// Declare a HTTP api with base_url
///
/// # Examples
///
/// ### Declare
///
/// ```
/// use apisdk::http_api;
///
/// #[http_api("https://host.of.service/base/path")]
/// #[derive(Debug, Clone)]
/// pub struct MyApi {
///     any_fields: MustImplDefault
/// }
/// ```
///
/// ### Define APIs
///
/// ```
/// use apisdk::{ApiResult, send_json};
/// use serde_json::{json, Value};
///
/// impl MyApi {
///     async fn do_sth(&self, param: u32) -> ApiResult<Value> {
///         let req = self.post("/relative-path/api").await?;
///         let payload = json!({
///             "param": param,
///         });
///         send_json!(req, payload).await
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn http_api(
    meta: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let vis = ast.vis;
    let api_name = ast.ident;
    let api_attrs = ast.attrs;
    let (fields_decl, fields_init, fields_clone) = parse_fields(ast.data);

    let (builder_name, builder_impl) =
        build_builder(vis.clone(), api_name.clone(), meta, fields_init);
    let api_impl = build_api_impl(
        vis.clone(),
        api_name.clone(),
        api_attrs,
        fields_decl,
        fields_clone,
        builder_name,
    );
    let methods = build_api_methods(vis.clone());

    let output = quote! {
        #api_impl
        #builder_impl
        impl #api_name {
            #(#methods)*
        }
    };

    output.into()
}

/// Refine a method of HTTP api
#[proc_macro_attribute]
pub fn api_method(
    meta: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let meta = syn::parse_macro_input!(meta as Meta);
    let log_enabled = if let Meta::NameValue(name_value) = meta {
        if name_value.path.is_ident("log") {
            name_value.value
        } else {
            syn::parse_str::<Expr>("off").unwrap()
        }
    } else {
        syn::parse_str::<Expr>("off").unwrap()
    };

    let item_fn = syn::parse_macro_input!(input as ItemFn);
    let fn_vis = item_fn.vis;
    let fn_sig = item_fn.sig;
    let fn_block = item_fn.block;

    let macros = build_macro_overrides(fn_sig.ident.clone());

    let output = quote! {
        #[allow(unused)]
        #fn_vis #fn_sig {
            #(#macros)*

            Self::__REQ_CONFIG.set(apisdk::__internal::RequestConfigurator::new(apisdk::_function_path!(), Some(#log_enabled), false));
            #fn_block
        }
    };

    output.into()
}

// #[proc_macro_derive(JsonPayload)]
// pub fn json_payload(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = input.ident;
//     let generics = input.generics;

//     let output = if generics.params.is_empty() {
//         build_simple_json_payload(name)
//     } else {
//         // build_simple_json_payload(name)
//         build_generic_json_payload(name, generics)
//     };

//     println!("out = {:?}", output);
//     output.into()
// }
