use std::sync::Arc;

use crate::{
    ApiError, ApiResult, ApiSignature, Client, ClientBuilder, DnsResolver, Initialiser, IntoUrl,
    LogConfig, LogMiddleware, Method, Middleware, RequestBuilder, RequestTraceIdMiddleware,
    ReqwestDnsResolver, ReqwestUrlRewriter, RouteError, SignatureMiddleware, Url, UrlOps,
    UrlRewriter,
};

/// This struct is used to build an instance of ApiCore
pub struct ApiBuilder {
    /// Reqwest ClientBuilder
    client: ClientBuilder,
    /// Base url for target api
    base_url: Url,
    /// The holder of UrlRewriter
    rewriter: Option<ReqwestUrlRewriter>,
    /// The holder of DnsResolver
    resolver: Option<ReqwestDnsResolver>,
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
            rewriter: None,
            resolver: None,
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

    /// Set the UrlRewriter
    /// - resolver: UrlRewriter
    pub fn with_rewriter<T>(self, rewriter: T) -> Self
    where
        T: UrlRewriter,
    {
        Self {
            rewriter: Some(ReqwestUrlRewriter::new(rewriter)),
            ..self
        }
    }

    /// Set the DnsResolver
    /// - resolver: DnsResolver
    pub fn with_resolver<T>(self, resolver: T) -> Self
    where
        T: DnsResolver,
    {
        Self {
            resolver: Some(ReqwestDnsResolver::new(resolver)),
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
        let client = match self.resolver.clone() {
            Some(r) => self.client.dns_resolver(Arc::new(r)),
            None => self.client,
        };
        let mut client = reqwest_middleware::ClientBuilder::new(client.build().unwrap());

        // Apply middleware in correct order
        client = client.with(RequestTraceIdMiddleware);
        // client = client.with(RewriteHostMiddleware);
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
            rewriter: self.rewriter,
            resolver: self.resolver,
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
    /// The holder of ReqwestUrlRewriter
    rewriter: Option<ReqwestUrlRewriter>,
    /// The holder of ReqwestDnsResolver
    resolver: Option<ReqwestDnsResolver>,
    /// The holder of ApiSignature
    signature: Option<Arc<dyn ApiSignature>>,
}

impl std::fmt::Debug for ApiCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("ApiCore");
        let mut d = d
            .field("client", &self.client)
            .field("base_url", &self.base_url);
        if let Some(r) = self.rewriter.as_ref() {
            d = d.field("rewriter", &r.type_name());
        }
        if let Some(r) = self.resolver.as_ref() {
            d = d.field("resolver", &r.type_name());
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
        Ok(Self {
            client: self.client.clone(),
            base_url,
            rewriter: self.rewriter.clone(),
            resolver: self.resolver.clone(),
            signature: self.signature.clone(),
        })
    }

    // /// Get next ApiEndpoint
    // pub async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError> {
    //     match self.router.as_ref() {
    //         Some(router) => router.next_endpoint().await,
    //         None => Ok(Box::new(OriginalEndpoint)),
    //     }
    // }

    // /// Get next UrlBuilder
    // pub async fn next_url_builder(&self) -> Result<impl UrlBuilder, RouteError> {
    //     let endpoint = self.next_endpoint().await?;
    //     Ok((endpoint, self.base_url.clone()))
    // }

    /// Build a new request url
    /// - path: relative path to base_url
    ///
    /// Return error when failed to retrieve valid endpoint from ApiRouter
    pub async fn build_url(&self, path: impl AsRef<str>) -> ApiResult<Url> {
        let mut base = self.build_base_url().await?;
        base.merge_path(path.as_ref());
        Ok(base)
    }

    async fn build_base_url(&self) -> Result<Url, RouteError> {
        let mut base_url = self.base_url.clone();
        if let Some(router) = self.rewriter.as_ref() {
            base_url = router.rewrite(base_url).await?;
        }
        if let Some(resolver) = self.resolver.as_ref() {
            base_url = resolver.rewrite(base_url).await?;
        }
        Ok(base_url)
    }

    /// Build a new HTTP request
    /// - method: HTTP method
    /// - path: relative path to base_url
    pub async fn build_request(
        &self,
        method: Method,
        path: impl AsRef<str>,
    ) -> ApiResult<RequestBuilder> {
        let url = self.build_url(path.as_ref()).await?;
        let req = self.client.request(method, url);

        // // Keep original HOST if required
        // if endpoint.reserve_original_host() {
        //     if let Some(host) = self.base_url.host_str() {
        //         req = req.with_extension(RewriteHost::new(host));
        //     }
        // }

        match self.signature.clone() {
            Some(signature) => Ok(req.with_extension(signature)),
            None => Ok(req),
        }
    }
}
