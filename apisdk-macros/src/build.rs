use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Visibility};

use crate::parse::Metadata;

/// Generate ApiBuilder
pub(crate) fn build_builder(
    metadata: &Metadata,
    vis: Visibility,
    api_name: Ident,
    fields_init: TokenStream,
) -> (Ident, TokenStream) {
    let Metadata { base_url, default } = metadata;
    let name = Ident::new(format!("{}Builder", api_name).as_str(), Span::call_site());

    let mut builder = quote! {
        /// The build is used to customize the api
        #vis struct #name {
            inner: apisdk::ApiBuilder,
        }

        impl Default for #name {
            fn default() -> Self {
                Self::new(#base_url)
            }
        }

        impl #name {
            /// Construct a new builder with base_url
            fn new(base_url: impl apisdk::IntoUrl + std::fmt::Debug) -> Self {
                Self {
                    inner: apisdk::ApiBuilder::new(base_url).expect("Invalid base_url"),
                }
            }

            // Set ClientBuilder
            pub fn with_client(self, client: apisdk::ClientBuilder) -> Self {
                Self {
                    inner: self.inner.with_client(client)
                }
            }

            /// Set UrlRewriter
            pub fn with_rewriter<T>(self, rewriter: T) -> Self where T: apisdk::UrlRewriter {
                Self {
                    inner: self.inner.with_rewriter(rewriter)
                }
            }

            /// Set DnsResolver
            pub fn with_resolver<T>(self, resolver: T) -> Self where T: apisdk::DnsResolver {
                Self {
                    inner: self.inner.with_resolver(resolver)
                }
            }

            /// Set ApiSignature
            pub fn with_signature<T>(self, signature: T) -> Self where T: apisdk::ApiSignature {
                Self {
                    inner: self.inner.with_signature(signature)
                }
            }

            /// Set initialiser
            pub fn with_initialiser<T>(self, initialiser: T) -> Self where T: apisdk::Initialiser {
                Self {
                    inner: self.inner.with_initialiser(initialiser)
                }
            }

            /// Add middleware
            pub fn with_middleware<T>(self, middleware: T) -> Self where T: apisdk::Middleware {
                Self {
                    inner: self.inner.with_middleware(middleware)
                }
            }

            /// Set log filter
            pub fn with_log<L>(self, level: L) -> Self where L: apisdk::IntoFilter {
                Self {
                    inner: self.inner.with_logger(apisdk::LogConfig::new(level))
                }
            }

            /// Disable log
            pub fn disable_log(self) -> Self {
                Self {
                    inner: self.inner.with_logger(apisdk::LogConfig::new(apisdk::LevelFilter::Off))
                }
            }

            /// Build the api core
            pub fn build_core(self) -> std::sync::Arc<apisdk::ApiCore> {
                std::sync::Arc::new(self.inner.build())
            }
        }
    };

    if *default {
        builder.extend(quote! {
            impl #name {
                /// Build the api instance
                pub fn build(self) -> #api_name {
                    #api_name {
                        core: std::sync::Arc::new(self.inner.build()),
                        #fields_init
                    }
                }
            }
        });
    }

    (name, builder)
}

/// Generate api basic implemations
pub(crate) fn build_api_impl(
    metadata: &Metadata,
    vis: Visibility,
    api_name: Ident,
    api_attrs: Vec<Attribute>,
    fields_decl: TokenStream,
    _fields_clone: TokenStream,
    builder_name: Ident,
) -> TokenStream {
    let Metadata { default, .. } = metadata;

    let mut api = quote! {
        #(#api_attrs)*
        #vis struct #api_name {
            pub core: std::sync::Arc<apisdk::ApiCore>,
            #fields_decl
        }

        impl #api_name {
            thread_local! {
                pub static __REQ_CONFIG: std::cell::RefCell<apisdk::__internal::RequestConfigurator>
                    = std::cell::RefCell::new(apisdk::__internal::RequestConfigurator::default());
            }

            /// Create an ApiBuilder
            pub fn builder() -> #builder_name {
                #builder_name::default()
            }

            /// Build request url
            /// - path: relative path
            pub async fn build_url(
                &self,
                path: impl AsRef<str>,
            ) -> apisdk::ApiResult<apisdk::Url> {
                self.core.build_url(path).await
            }

            /// Build a new HTTP request
            /// - method: HTTP method
            /// - path: relative path
            pub async fn request(
                &self,
                method: apisdk::Method,
                path: impl AsRef<str>,
            ) -> apisdk::ApiResult<apisdk::RequestBuilder> {
                self.core.build_request(method, path).await
            }
        }
    };

    if *default {
        api.extend(quote! {
            impl Default for #api_name {
                fn default() -> Self {
                    Self::builder().build()
                }
            }
        });
    }

    api
}

/// Generate shortcut methods for api
pub(crate) fn build_api_methods(_vis: Visibility) -> Vec<TokenStream> {
    [
        "head", "get", "post", "put", "patch", "delete", "options", "trace",
    ]
    .iter()
    .map(|method| {
        let method_func = Ident::new(method, Span::call_site());
        let method_enum = Ident::new(&method.to_uppercase(), Span::call_site());
        quote! {
            /// Build a new HTTP request
            /// - path: relative path
            pub async fn #method_func(
                &self,
                path: impl AsRef<str>,
            ) -> apisdk::ApiResult<apisdk::RequestBuilder> {
                use std::str::FromStr;
                self.core.build_request(apisdk::Method::#method_enum, path).await
            }
        }
    })
    .collect()
}

pub(crate) fn build_macro_overrides(_fn_name: Ident) -> Vec<TokenStream> {
    // let fn_name = fn_name.to_string();
    [
        "send",
        "send_json",
        "send_xml",
        "send_form",
        "send_multipart",
    ]
    .iter()
    .map(|name| {
        let macro_name = Ident::new(name, Span::call_site());
        let macro_with_name = Ident::new(format!("_{}_with", name).as_str(), Span::call_site());
        quote! {
            #[allow(unused)]
            macro_rules! #macro_name {
                ($req:expr) => {
                    async {
                        apisdk::#macro_with_name!($req, Self::__REQ_CONFIG.take()).await
                    }
                };
                ($req:expr, $arg:tt) => {
                    async {
                        apisdk::#macro_with_name!($req, $arg, Self::__REQ_CONFIG.take()).await
                    }
                };
                ($req:expr, $arg1:expr, $arg2:tt) => {
                    async {
                        apisdk::#macro_with_name!($req, $arg1, $arg2, Self::__REQ_CONFIG.take()).await
                    }
                };
            }
        }
    })
    .collect()
}

// pub(crate) fn build_simple_json_payload(name: Ident) -> TokenStream {
//     quote! {
//         impl apisdk::TryFromJson for #name {
//             fn try_from_json(json: apisdk::serde_json::Value) -> apisdk::ApiResult<Self> {
//                 apisdk::serde_json::from_value(json).map_err(|e| apisdk::ApiError::Other)
//             }
//         }
//         impl apisdk::TryFromString for #name {
//             fn try_from_string(text: String) -> apisdk::ApiResult<Self> {
//                 Err(apisdk::ApiError::Other)
//             }
//         }
//     }
// }

// pub(crate) fn build_generic_json_payload(name: Ident, generics: Generics) -> TokenStream {
//     let idents: Vec<_> = generics
//         .params
//         .iter()
//         .map(|p| match p {
//             GenericParam::Lifetime(param) => {
//                 let ident = param.lifetime.ident.clone();
//                 quote! { #ident }
//             }
//             GenericParam::Type(param) => {
//                 let ident = param.ident.clone();
//                 quote! { #ident }
//             }
//             GenericParam::Const(param) => {
//                 let ident = param.ident.clone();
//                 quote! { #ident }
//             }
//         })
//         .collect();
//     println!("idents = {:?}", idents);

//     quote! {
//         impl<#(#idents)*> apisdk::TryFromJson for #name<#(#idents)*> {
//             fn try_from_json(json: apisdk::serde_json::Value) -> apisdk::ApiResult<Self> {
//                 apisdk::serde_json::from_value(json).map_err(|e| apisdk::ApiError::Other)
//             }
//         }
//         impl<#(#idents)*> apisdk::TryFromString for #name<#(#idents)*> {
//             fn try_from_string(text: String) -> apisdk::ApiResult<Self> {
//                 Err(apisdk::ApiError::Other)
//             }
//         }
//     }
// }
