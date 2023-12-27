use std::net::{IpAddr, SocketAddr};

use async_trait::async_trait;
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
    Resolver,
};

use crate::DnsResolver;

pub struct Dns(Resolver);

impl Dns {
    pub fn new(ips: &[IpAddr]) -> Self {
        Self(
            Resolver::new(
                ResolverConfig::from_parts(
                    None,
                    vec![],
                    NameServerConfigGroup::from_ips_clear(ips, 0, true),
                ),
                ResolverOpts::default(),
            )
            .unwrap(),
        )
    }
}

#[async_trait]
impl DnsResolver for Dns {
    async fn resolve(&self, name: &str) -> Option<SocketAddr> {
        self.0
            .lookup_ip(name)
            .ok()
            .and_then(|lookup_ip| lookup_ip.iter().next())
            .map(|i| SocketAddr::from((i, 0)))
    }
}
