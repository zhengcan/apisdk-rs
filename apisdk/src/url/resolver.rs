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

use crate::RouteError;

use super::rewriter::UrlRewriter;

pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait DnsResolver: 'static + Send + Sync {
    fn get_scheme(&self) -> Option<&str> {
        None
    }

    fn get_port(&self) -> Option<u16> {
        None
    }

    async fn resolve(&self, name: &str) -> Option<SocketAddr>;
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

    async fn resolve(&self, _name: &str) -> Option<SocketAddr> {
        Some(SocketAddr::from((self.0.clone(), self.1)))
    }
}

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

struct SingleSocketAddr(Option<SocketAddr>);

impl Iterator for SingleSocketAddr {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

#[derive(Clone)]
pub(crate) struct ReqwestDnsResolver {
    type_name: &'static str,
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
    async fn rewrite(&self, url: Url) -> Result<Url, RouteError> {
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
            if let Some(addr) = me.resolver.resolve(name.as_str()).await {
                let addrs: Addrs = Box::new(SingleSocketAddr(Some(addr)));
                return Ok(addrs);
            }
            me.fallback.resolve(name).await
        })
    }
}
