use std::sync::Arc;

use crate::{
    ApiEndpoint, ApiError, ApiResolver, ApiResult, ApiRouter, ApiSignature, Client, ClientBuilder,
    Initialiser, IntoUrl, LogConfig, LogMiddleware, Method, Middleware, OriginalEndpoint,
    RequestBuilder, RequestTraceIdMiddleware, ReqwestApiResolver, RewriteHost,
    RewriteHostMiddleware, RouteError, SignatureMiddleware, Url, UrlBuilder,
};

/// This struct is used to build an instance of ApiCore
pub struct ApiBuilder {
    /// Reqwest ClientBuilder
    client: ClientBuilder,
    /// Base url for target api
    base_url: Url,
    /// The holder of ApiResolver
    resolver: Option<Arc<dyn ApiResolver>>,
    /// The holder of ApiRouter
    router: Option<Arc<dyn ApiRouter>>,
    /// The holder of ApiSignature
    signature: Option<Arc<dyn ApiSignature>>,
    /// The holder of LogConfig
    logger: Option<Arc<LogConfig>>,
    /// The initialisers for Reqwest
    initialisers: Vec<Arc<dyn Initialiser>>,
    /// The middlewares for Reqwest
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl ApiBuilder {
    /// Create an instance of ApiBuilder
    /// - base_url: base url for target api
    pub fn new(base_url: impl IntoUrl + std::fmt::Debug) -> ApiResult<Self> {
        Ok(Self {
            client: ClientBuilder::default(),
            base_url: base_url.into_url().map_err(ApiError::InvalidUrl)?,
            resolver: None,
            router: None,
            signature: None,
            logger: None,
            initialisers: vec![],
            middlewares: vec![],
        })
    }

    /// Set the ClientBuilder to create Client instance of Reqwest
    /// - client: Reqwest ClientBuilder
    pub fn with_client(self, client: ClientBuilder) -> Self {
        Self { client, ..self }
    }

    /// Set the ApiResolver
    /// - resolver: ApiResolver
    pub fn with_resolver<T>(self, resolver: T) -> Self
    where
        T: ApiResolver,
    {
        Self {
            resolver: Some(Arc::new(resolver)),
            ..self
        }
    }

    /// Set the ApiRouter
    /// - router: ApiRouter
    pub fn with_router<T>(self, router: T) -> Self
    where
        T: ApiRouter,
    {
        Self {
            router: Some(Arc::new(router)),
            ..self
        }
    }

    /// Set the ApiSignature
    /// - signature: ApiSignature
    pub fn with_signature<T>(self, signature: T) -> Self
    where
        T: ApiSignature,
    {
        Self {
            signature: Some(Arc::new(signature)),
            ..self
        }
    }

    /// Set the LogConfig
    /// - logger: LogConfig
    pub fn with_logger<T>(self, logger: T) -> Self
    where
        T: Into<LogConfig>,
    {
        Self {
            logger: Some(Arc::new(logger.into())),
            ..self
        }
    }

    /// Add initialiser
    /// - initialiser: Reqwest Initialiser
    pub fn with_initialiser<T>(self, initialiser: T) -> Self
    where
        T: Initialiser,
    {
        let mut s = self;
        s.initialisers.push(Arc::new(initialiser));
        s
    }

    /// Add middleware
    /// - middleware: Reqwest Middleware
    pub fn with_middleware<T>(self, middleware: T) -> Self
    where
        T: Middleware,
    {
        let mut s = self;
        s.middlewares.push(Arc::new(middleware));
        s
    }

    /// Build an instance of ApiCore
    pub fn build(self) -> ApiCore {
        let resolver = self
            .resolver
            .map(|r| Arc::new(ReqwestApiResolver::new(r, &self.base_url)));
        let client = match resolver.as_ref() {
            Some(r) => self.client.dns_resolver(r.clone()),
            None => self.client,
        };
        let mut client = reqwest_middleware::ClientBuilder::new(client.build().unwrap());

        // Apply middleware in correct order
        client = client.with(RequestTraceIdMiddleware);
        client = client.with(RewriteHostMiddleware);
        for middleware in self.middlewares {
            client = client.with_arc(middleware);
        }
        if self.signature.is_some() {
            client = client.with(SignatureMiddleware);
        }
        client = client.with(LogMiddleware);

        // Apply initialisers
        if let Some(logger) = self.logger {
            client = client.with_arc_init(logger);
        };
        for initialiser in self.initialisers {
            client = client.with_arc_init(initialiser);
        }

        ApiCore {
            client: client.build(),
            base_url: self.base_url,
            resolver,
            router: self.router,
            signature: self.signature,
        }
    }
}

/// This struct is used to create HTTP request
pub struct ApiCore {
    /// Reqwest Client
    client: Client,
    /// Base url for target api
    base_url: Url,
    /// The holder of ReqwestApiResolver
    resolver: Option<Arc<ReqwestApiResolver>>,
    /// The holder of ApiRouter
    router: Option<Arc<dyn ApiRouter>>,
    /// The holder of ApiSignature
    signature: Option<Arc<dyn ApiSignature>>,
}

impl std::fmt::Debug for ApiCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("ApiCore");
        let mut d = d
            .field("client", &self.client)
            .field("base_url", &self.base_url);
        if let Some(r) = self.router.as_ref() {
            d = d.field("router", &r.type_name());
        }
        if let Some(s) = self.signature.as_ref() {
            d = d.field("signature", &s.type_name());
        }
        d.finish()
    }
}

impl ApiCore {
    /// Create a new ApiCore with a different base_url
    pub fn rebase(&self, base_url: impl IntoUrl) -> ApiResult<Self> {
        let base_url = base_url.into_url().map_err(ApiError::InvalidUrl)?;
        let resolver = self
            .resolver
            .as_ref()
            .map(|r| Arc::new(r.rebase(&base_url)));
        Ok(Self {
            client: self.client.clone(),
            base_url,
            resolver,
            router: self.router.clone(),
            signature: self.signature.clone(),
        })
    }

    /// Create a new ApiCore with a different router
    pub fn reroute(&self, router: impl ApiRouter) -> Self {
        Self {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            resolver: self.resolver.clone(),
            router: Some(Arc::new(router)),
            signature: self.signature.clone(),
        }
    }

    /// Get next ApiEndpoint
    pub async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError> {
        match self.router.as_ref() {
            Some(router) => router.next_endpoint().await,
            None => Ok(Box::new(OriginalEndpoint)),
        }
    }

    /// Get next UrlBuilder
    pub async fn next_url_builder(&self) -> Result<impl UrlBuilder, RouteError> {
        let endpoint = self.next_endpoint().await?;
        Ok((endpoint, self.base_url.clone()))
    }

    /// Build a new request url
    /// - path: relative path to base_url
    ///
    /// Return error when failed to retrieve valid endpoint from ApiRouter
    pub async fn build_url(&self, path: impl AsRef<str>) -> ApiResult<Url> {
        let endpoint = self.next_endpoint().await?;
        endpoint
            .build_url(&self.base_url, path.as_ref())
            .map_err(|e| e.into())
    }

    /// Build a new HTTP request
    /// - method: HTTP method
    /// - path: relative path to base_url
    pub async fn build_request(
        &self,
        method: Method,
        path: impl AsRef<str>,
    ) -> ApiResult<RequestBuilder> {
        // Rewrite
        // if let Some(router) = self.router.as_ref() {
        //     //
        // }

        // Resolve
        let endpoint = self.next_endpoint().await?;
        let url = endpoint.build_url(&self.base_url, path.as_ref())?;
        let mut req = self.client.request(method, url);

        // Keep original HOST if required
        if endpoint.reserve_original_host() {
            if let Some(host) = self.base_url.host_str() {
                req = req.with_extension(RewriteHost::new(host));
            }
        }

        match self.signature.clone() {
            Some(signature) => Ok(req.with_extension(signature)),
            None => Ok(req),
        }
    }
}
