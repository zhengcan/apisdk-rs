use std::{net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use futures::FutureExt;
use hyper::{
    client::connect::dns::{GaiResolver, Name},
    service::Service,
};
use reqwest::dns::{Addrs, Resolve, Resolving};
use url::Url;

pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait ApiResolver: 'static + Send + Sync {
    fn get_scheme(&self) -> Option<&str> {
        None
    }

    fn get_port(&self) -> Option<u16> {
        None
    }

    async fn resolve(&self, name: &str) -> Option<SocketAddr>;
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

#[derive(Clone)]
pub struct ReqwestApiResolver {
    resolver: Arc<dyn ApiResolver>,
    fallback: FallbackResolver,
    base_url: Url,
}

impl ReqwestApiResolver {
    pub fn new(resolver: Arc<dyn ApiResolver>, base_url: &Url) -> Self {
        let mut base_url = base_url.clone();
        if let Some(scheme) = resolver.get_scheme() {
            let _ = base_url.set_scheme(scheme);
        }
        if let Some(port) = resolver.get_port() {
            let _ = base_url.set_port(Some(port));
        }
        Self {
            resolver,
            fallback: FallbackResolver(GaiResolver::new()),
            base_url,
        }
    }

    pub fn rebase(&self, base_url: &Url) -> Self {
        Self::new(self.resolver.clone(), base_url)
    }
}

struct SingleSocketAddr(Option<SocketAddr>);

impl Iterator for SingleSocketAddr {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.take()
    }
}

impl Resolve for ReqwestApiResolver {
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

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, str::FromStr, sync::Arc};

    use async_trait::async_trait;
    use rand::random;
    use reqwest::ClientBuilder;
    use tracing::Level;
    use tracing_log::LogTracer;
    use tracing_subscriber::{
        fmt::{writer::MakeWriterExt, Layer},
        layer::SubscriberExt,
        Registry,
    };
    use url::Url;

    use crate::{ApiResolver, ReqwestApiResolver};

    fn init_logger() {
        let registry = Registry::default().with(
            Layer::default()
                .with_ansi(true)
                .without_time()
                .with_writer(std::io::stdout.with_max_level(Level::TRACE)),
        );
        if tracing::subscriber::set_global_default(registry).is_ok() {
            let _ = LogTracer::init();
        }
    }

    #[tokio::test]
    async fn test_resolve() {
        init_logger();

        struct TheResolver;

        #[async_trait]
        impl ApiResolver for TheResolver {
            fn get_scheme(&self) -> Option<&str> {
                Some("http")
            }

            fn get_port(&self) -> Option<u16> {
                Some(8888)
            }

            async fn resolve(&self, name: &str) -> Option<SocketAddr> {
                if name == "hit" {
                    let ip = random::<u8>();
                    let port = 30000 + (10000f32 * random::<f32>()) as u16;
                    Some(SocketAddr::from(([127, 0, 0, ip], port)))
                } else {
                    None
                }
            }
        }

        let base_url = Url::from_str("https://hit/path").unwrap();
        let resolver = Arc::new(ReqwestApiResolver::new(Arc::new(TheResolver), &base_url));
        let client = ClientBuilder::new()
            .dns_resolver(resolver.clone())
            .build()
            .unwrap();
        for _ in 0..10 {
            let result = client.get(resolver.base_url.clone()).send().await;
            assert!(result.is_err());
            // println!("result = {:?}", result);
        }
    }
}
