use std::net::{IpAddr, SocketAddr};

use async_trait::async_trait;
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
    Resolver,
};

use crate::{DnsResolver, SocketAddrs};

/// The NameServer performs DNS queries
pub struct NameServer(Resolver);

impl NameServer {
    /// Create an instance with many NS IPs
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
impl DnsResolver for NameServer {
    async fn resolve(&self, name: &str) -> Option<SocketAddrs> {
        self.0.lookup_ip(name).ok().map(|lookup_ip| {
            SocketAddrs::new(Box::new(
                lookup_ip.into_iter().map(|ip| SocketAddr::from((ip, 0))),
            ))
        })
    }
}
