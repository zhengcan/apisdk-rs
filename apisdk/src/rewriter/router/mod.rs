mod multi;
mod simple;

use std::{any::type_name, str::FromStr};

use multi::*;
use simple::*;

use crate::{async_trait, RouteError, Url, UrlRewrite};

/// This trait is used to generate an endpoint for each request
///
/// # Examples
///
/// ```
/// #[derive(Debug)]
/// struct EnvBasedRouter;
///
/// #[async_trait]
/// impl ApiRouter for EnvBasedRouter {
///     async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError> {
///         let env = std::env::var_os("ENV")
///             .map(|v| v.to_string_lossy().to_uppercase())
///             .unwrap_or_default();
///         if env == "PROD" {
///             Ok(Box::new(DefaultApiEndpoint::new_default("1.1.1.1", 80)))
///         } else {
///             Ok(Box::new(DefaultApiEndpoint::new_default("2.2.2.2", 80)))
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait ApiRouter: 'static + Send + Sync {
    /// Get type_name, used in Debug
    fn type_name(&self) -> &str {
        type_name::<Self>()
    }

    /// Indicate whether the HOST header should be rewritten.
    /// As default, we keep the original HOST from original url.
    fn rewrite_host(&self) -> bool {
        false
    }

    /// Generate endpoint
    async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError>;
}

#[async_trait]
impl UrlRewrite for Box<dyn ApiRouter> {
    async fn rewrite(&self, url: Url) -> Url {
        url
    }
}

#[async_trait]
impl ApiRouter for Box<dyn ApiRouter> {
    fn rewrite_host(&self) -> bool {
        self.as_ref().rewrite_host()
    }

    async fn next_endpoint(&self) -> Result<Box<dyn ApiEndpoint>, RouteError> {
        self.as_ref().next_endpoint().await
    }
}

/// This struct provides several built-in implements of `ApiRouter`
pub struct ApiRouters;

impl ApiRouters {
    /// Initiate a fixed ApiRouter
    pub fn fixed(endpoint: impl Into<DefaultApiEndpoint>) -> impl ApiRouter {
        SimpleApiRouter::new(endpoint.into())
    }

    /// Initiate a round robin ApiRouter for multiply endpoints
    pub fn round_robin(endpoints: &[DefaultApiEndpoint]) -> impl ApiRouter {
        MultiApiRouter::new_round_robin(endpoints)
    }

    /// Initiate a random ApiRouter for multiply endpoints
    pub fn random(endpoints: &[DefaultApiEndpoint]) -> impl ApiRouter {
        MultiApiRouter::new_random(endpoints)
    }
}

pub trait UrlBuilder {
    fn build_url(&self, path: &str) -> Result<Url, RouteError>;
}

impl UrlBuilder for (Box<dyn ApiEndpoint>, Url) {
    fn build_url(&self, path: &str) -> Result<Url, RouteError> {
        self.0.build_url(&self.1, path)
    }
}

/// This trait is used to build urls
pub trait ApiEndpoint: Send + Sync {
    fn reserve_original_host(&self) -> bool {
        false
    }

    /// Build request url
    /// - base: original base url
    /// - path: relative path
    fn build_url(&self, base: &Url, path: &str) -> Result<Url, RouteError>;

    /// Merge base url and path
    /// - base: base url
    /// - path: relative path
    fn merge_path(&self, base: &mut Url, path: &str) {
        let base_path = base.path();
        let new_path = match (base_path.ends_with('/'), path.starts_with('/')) {
            (true, true) => format!("{}{}", base_path, &path[1..]),
            (true, false) | (false, true) => format!("{}{}", base_path, path),
            (false, false) => format!("{}/{}", base_path, path),
        };
        base.set_path(&new_path);
    }
}

/// This endpoint keep original base url from ApiCore
#[derive(Debug, Default)]
pub struct OriginalEndpoint;

impl ApiEndpoint for OriginalEndpoint {
    fn build_url(&self, base: &Url, path: &str) -> Result<Url, RouteError> {
        let mut url = base.clone();
        self.merge_path(&mut url, path);
        Ok(url)
    }
}

/// This struct is a default implementation of `ApiEndpoint`
#[derive(Debug, Clone)]
pub struct DefaultApiEndpoint {
    scheme: Option<String>,
    host: String,
    port: u16,
}

impl DefaultApiEndpoint {
    /// Create an instance without scheme
    pub fn new_default(host: impl ToString, port: u16) -> Self {
        Self::new(None::<&str>, host, port)
    }

    /// Create an instance with http
    pub fn new_http(host: impl ToString, port: u16) -> Self {
        Self::new(Some("http"), host, port)
    }

    /// Create an instance with https
    pub fn new_https(host: impl ToString, port: u16) -> Self {
        Self::new(Some("https"), host, port)
    }

    /// Create an instance
    pub fn new(scheme: Option<impl ToString>, host: impl ToString, port: u16) -> Self {
        Self {
            scheme: scheme.map(|s| s.to_string()),
            host: host.to_string(),
            port,
        }
    }
}

impl<T> From<(T, u16)> for DefaultApiEndpoint
where
    T: ToString,
{
    fn from((host, port): (T, u16)) -> Self {
        Self::new_default(host, port)
    }
}

impl FromStr for DefaultApiEndpoint {
    type Err = RouteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((host, port)) = s.split_once(':') {
            if host.is_empty() {
                return Err(RouteError::Custom("Invalid host".to_string()));
            }
            let port = match port.parse::<u16>() {
                Ok(port) => port,
                Err(e) => {
                    return Err(RouteError::Custom(format!("Invalid port: {}", e)));
                }
            };
            return Ok(DefaultApiEndpoint::new_default(host, port));
        }
        Err(RouteError::Custom(format!("Invalid endpoint: {}", s)))
    }
}

impl ApiEndpoint for DefaultApiEndpoint {
    fn build_url(&self, base: &Url, path: &str) -> Result<Url, RouteError> {
        let mut url = base.clone();
        if let Some(scheme) = self.scheme.as_ref() {
            url.set_scheme(scheme.as_str())
                .map_err(|_| RouteError::UpdateScheme(base.clone(), scheme.clone()))?;
        }
        url.set_host(Some(self.host.as_str()))
            .map_err(|e| RouteError::UpdateHost(base.clone(), self.host.clone(), e))?;
        url.set_port(Some(self.port))
            .map_err(|_| RouteError::UpdatePort(base.clone(), self.port))?;
        self.merge_path(&mut url, path);
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ApiRouter, ApiRouters};

    fn dump_router<T>(router: T)
    where
        T: ApiRouter,
    {
        println!("router = {}", router.type_name());
    }

    #[test]
    fn test_box_router() {
        let boxed: Box<dyn ApiRouter> = Box::new(ApiRouters::fixed(("127.0.0.1", 80)));
        dump_router(boxed);
    }
}
