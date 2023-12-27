use async_trait::async_trait;
use url::Url;

mod resolver;
mod router;

pub use resolver::*;
pub use router::*;

#[async_trait]
pub trait UrlRewrite {
    async fn rewrite(&self, url: Url) -> Url;
}
