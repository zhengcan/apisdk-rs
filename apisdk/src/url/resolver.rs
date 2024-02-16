use std::{
    any::type_name,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use async_trait::async_trait;
use futures::FutureExt;
use hyper::{
    client::connect::dns::{GaiResolver, Name},
    service::Service,
};
use reqwest::dns::{Addrs, Resolve, Resolving};
use url::Url;

use crate::{ApiError, UrlRewriter};

pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// This struct is used to provides many SocketAddrs
pub struct SocketAddrs {
    iter: Addrs,
}

impl SocketAddrs {
    /// Construct an instance based on Iterator of SocketAddrs
    pub fn new(addrs: Addrs) -> Self {
        Self { iter: addrs }
    }

    /// Construct an instance based on vector of SocketAddrs
    pub fn new_multi(addrs: Vec<SocketAddr>) -> Self {
        Self {
            iter: Box::new(addrs.into_iter()),
        }
    }

    /// Construct an instance based on single SocketAddr
    pub fn new_single(addr: SocketAddr) -> Self {
        Self {
            iter: Box::new(Some(addr).into_iter()),
        }
    }
}

impl From<IpAddr> for SocketAddrs {
    fn from(value: IpAddr) -> Self {
        SocketAddrs::new_single(SocketAddr::from((value, 0)))
    }
}

impl<I: Into<IpAddr>> From<(I, u16)> for SocketAddrs {
    fn from(value: (I, u16)) -> Self {
        SocketAddrs::new_single(SocketAddr::from(value))
    }
}

impl From<SocketAddr> for SocketAddrs {
    fn from(value: SocketAddr) -> Self {
        SocketAddrs::new_single(value)
    }
}

/// This trait is used to performing DNS queries
#[async_trait]
pub trait DnsResolver: 'static + Send + Sync {
    /// Return `Some` if scheme should be changed
    fn get_scheme(&self) -> Option<&str> {
        None
    }

    /// Return `Some` if port should be changed
    fn get_port(&self) -> Option<u16> {
        None
    }

    /// Do DNS queries
    async fn resolve(&self, name: &str) -> Option<SocketAddrs>;
}

#[async_trait]
impl<F> DnsResolver for F
where
    F: Fn(&str) -> Option<SocketAddrs>,
    F: 'static + Send + Sync,
{
    async fn resolve(&self, name: &str) -> Option<SocketAddrs> {
        self(name)
    }
}

#[async_trait]
impl DnsResolver for IpAddr {
    async fn resolve(&self, _name: &str) -> Option<SocketAddrs> {
        Some(SocketAddrs::from((*self, 0)))
    }
}

#[async_trait]
impl DnsResolver for SocketAddr {
    async fn resolve(&self, _name: &str) -> Option<SocketAddrs> {
        Some(SocketAddrs::from(*self))
    }
}

#[async_trait]
impl<T> DnsResolver for (T, u16)
where
    T: Into<IpAddr>,
    T: 'static + Send + Sync,
    T: Clone,
{
    fn get_port(&self) -> Option<u16> {
        Some(self.1)
    }

    async fn resolve(&self, _name: &str) -> Option<SocketAddrs> {
        Some(SocketAddrs::from((self.0.clone(), self.1)))
    }
}

#[async_trait]
impl DnsResolver for Box<dyn DnsResolver> {
    async fn resolve(&self, name: &str) -> Option<SocketAddrs> {
        self.as_ref().resolve(name).await
    }
}

/// This is default DNS Resolver of reqwest
#[derive(Clone)]
struct FallbackResolver(GaiResolver);

impl Resolve for FallbackResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let this = &mut self.0.clone();
        Box::pin(Service::<Name>::call(this, name).map(|result| {
            result
                .map(|addrs| -> Addrs { Box::new(addrs) })
                .map_err(|err| -> BoxError { Box::new(err) })
        }))
    }
}

/// This struct is used to hold the provided `DnsResolver`, and perform DNS queries
#[derive(Clone)]
pub(crate) struct ReqwestDnsResolver {
    /// The type_name of provided `DnsResolver`
    type_name: &'static str,
    /// The provided `DnsResolver`
    resolver: Arc<dyn DnsResolver>,
    fallback: FallbackResolver,
}

impl ReqwestDnsResolver {
    pub fn new<T>(resolver: T) -> Self
    where
        T: DnsResolver,
    {
        Self {
            type_name: type_name::<T>(),
            resolver: Arc::new(resolver),
            fallback: FallbackResolver(GaiResolver::new()),
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }
}

#[async_trait]
impl UrlRewriter for ReqwestDnsResolver {
    /// Rewrite url if scheme and/or port should be changed
    async fn rewrite(&self, url: Url) -> Result<Url, ApiError> {
        let mut url = url;
        if let Some(scheme) = self.resolver.get_scheme() {
            let _ = url.set_scheme(scheme);
        }
        if let Some(port) = self.resolver.get_port() {
            let _ = url.set_port(Some(port));
        }
        Ok(url)
    }
}

impl Resolve for ReqwestDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let me = self.clone();
        Box::pin(async move {
            if let Some(addrs) = me.resolver.resolve(name.as_str()).await {
                return Ok(addrs.iter);
            }
            me.fallback.resolve(name).await
        })
    }
}
