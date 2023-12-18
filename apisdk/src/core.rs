use std::sync::Arc;

use reqwest::header::HOST;

use crate::{
    ApiResult, ApiRouter, ApiSignature, Client, ClientBuilder, Initialiser, IntoUrl, Method,
    Middleware, OriginalEndpoint, RequestBuilder, RequestTraceIdInjector, SignatureMiddleware, Url,
};

/// This struct is used to build an instance of ApiCore
pub struct ApiBuilder {
    /// Reqwest ClientBuilder
    client: ClientBuilder,
    /// Base url for target api
    base_url: Url,
    /// The holder of ApiRouter
    router: Option<Arc<dyn ApiRouter>>,
    /// The holder of ApiSignature
    signature: Option<Arc<dyn ApiSignature>>,
    /// The initialisers for Reqwest
    initialisers: Vec<Arc<dyn Initialiser>>,
    /// The middlewares for Reqwest
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl ApiBuilder {
    /// Create an instance of ApiBuilder
    /// - base_url: base url for target api
    pub fn new(base_url: impl IntoUrl + std::fmt::Debug) -> ApiResult<Self> {
        let request_trace_id_injector = Arc::new(RequestTraceIdInjector {});

        Ok(Self {
            client: ClientBuilder::default(),
            base_url: base_url.into_url()?,
            router: None,
            signature: None,
            initialisers: vec![],
            middlewares: vec![request_trace_id_injector],
        })
    }

    /// Set the ClientBuilder to create Client instance of Reqwest
    /// - client: Reqwest ClientBuilder
    pub fn with_client(self, client: ClientBuilder) -> Self {
        Self { client, ..self }
    }

    /// Set the ApiRouter
    /// - router: ApiRouter
    pub fn with_router(self, router: impl ApiRouter) -> Self {
        Self {
            router: Some(Arc::new(router)),
            ..self
        }
    }

    /// Set the ApiSignature
    /// - signature: ApiSignature
    pub fn with_signature(self, signature: impl ApiSignature) -> Self {
        Self {
            signature: Some(Arc::new(signature)),
            ..self
        }
    }

    /// Add initialiser
    /// - initialiser: Reqwest Initialiser
    pub fn with_initialiser(self, initialiser: impl Initialiser) -> Self {
        let mut s = self;
        s.initialisers.push(Arc::new(initialiser));
        s
    }

    /// Add middleware
    /// - middleware: Reqwest Middleware
    pub fn with_middleware(self, middleware: impl Middleware) -> Self {
        let mut s = self;
        s.middlewares.push(Arc::new(middleware));
        s
    }

    /// Build an instance of ApiCore
    pub fn build(self) -> ApiCore {
        let mut client = reqwest_middleware::ClientBuilder::new(self.client.build().unwrap());
        for initialiser in self.initialisers {
            client = client.with_arc_init(initialiser);
        }
        for middleware in self.middlewares {
            client = client.with_arc(middleware);
        }
        if self.signature.is_some() {
            client = client.with(SignatureMiddleware);
        }

        ApiCore {
            client: client.build(),
            base_url: self.base_url,
            router: self.router,
            signature: self.signature,
        }
    }
}

/// This struct is used to create HTTP request
#[derive(Debug)]
pub struct ApiCore {
    /// Reqwest Client
    client: Client,
    /// Base url for target api
    base_url: Url,
    /// The holder of ApiRouter
    router: Option<Arc<dyn ApiRouter>>,
    /// The holder of ApiSignature
    signature: Option<Arc<dyn ApiSignature>>,
}

impl ApiCore {
    /// Build a new request url
    /// - path: relative path to base_uri
    ///
    /// Return error when failed to retrieve valid endpoint from ApiRouter
    pub async fn build_url(&self, path: impl AsRef<str>) -> ApiResult<Url> {
        let endpoint = match self.router.as_ref() {
            Some(router) => router.next_endpoint().await?,
            None => Box::new(OriginalEndpoint {}),
        };
        endpoint
            .build_url(&self.base_url, path.as_ref())
            .map_err(|e| e.into())
    }

    /// Build a new HTTP request
    /// - method: HTTP method
    /// - path: relative path to base_uri
    pub async fn build_request(
        &self,
        method: Method,
        path: impl AsRef<str>,
    ) -> ApiResult<RequestBuilder> {
        let url = self.build_url(path).await?;
        let mut req = self.client.request(method, url);

        // Keep orginal HOST if required
        if !self
            .router
            .as_ref()
            .map(|r| r.rewrite_host())
            .unwrap_or_default()
        {
            if let Some(host) = self.base_url.host_str() {
                req = req.header(HOST, host);
            }
        }

        match self.signature.clone() {
            Some(signature) => Ok(req.with_extension(signature)),
            None => Ok(req),
        }
    }
}
