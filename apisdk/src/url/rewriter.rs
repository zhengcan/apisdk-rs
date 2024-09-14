use std::{
    any::type_name,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use async_trait::async_trait;
use url::Url;

use crate::ApiError;

/// This trait is used to rewrite base_url
#[async_trait]
pub trait UrlRewriter: 'static + Send + Sync {
    /// Rewrite url if possible
    async fn rewrite(&self, url: Url) -> Result<Url, ApiError>;
}

#[async_trait]
impl<F> UrlRewriter for F
where
    F: Fn(Url) -> Result<Url, ApiError>,
    F: 'static + Send + Sync,
{
    async fn rewrite(&self, url: Url) -> Result<Url, ApiError> {
        self(url)
    }
}

#[async_trait]
impl UrlRewriter for IpAddr {
    async fn rewrite(&self, url: Url) -> Result<Url, ApiError> {
        let mut url = url;
        let _ = url.set_ip_host(*self);
        Ok(url)
    }
}

#[async_trait]
impl UrlRewriter for SocketAddr {
    async fn rewrite(&self, url: Url) -> Result<Url, ApiError> {
        let mut url = url;
        let _ = url.set_ip_host(self.ip());
        let _ = url.set_port(Some(self.port()));
        Ok(url)
    }
}

#[async_trait]
impl UrlRewriter for Box<dyn UrlRewriter> {
    async fn rewrite(&self, url: Url) -> Result<Url, ApiError> {
        self.as_ref().rewrite(url).await
    }
}

/// This struct is used to hold the provided `UrlRewriter`, and perform url rewrites
#[derive(Clone)]
pub(crate) struct ReqwestUrlRewriter {
    /// The type_name of provided `UrlRewriter`
    type_name: &'static str,
    /// The provided `UrlRewriter`
    rewriter: Arc<dyn UrlRewriter>,
}

impl ReqwestUrlRewriter {
    pub fn new<T>(rewriter: T) -> Self
    where
        T: UrlRewriter,
    {
        Self {
            type_name: type_name::<T>(),
            rewriter: Arc::new(rewriter),
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }
}

#[async_trait]
impl UrlRewriter for ReqwestUrlRewriter {
    async fn rewrite(&self, url: Url) -> Result<Url, ApiError> {
        self.rewriter.rewrite(url).await
    }
}
