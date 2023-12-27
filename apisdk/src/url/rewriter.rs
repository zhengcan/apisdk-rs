use std::{any::type_name, sync::Arc};

use async_trait::async_trait;
use url::Url;

use crate::RouteError;

#[async_trait]
pub trait UrlRewriter: 'static + Send + Sync {
    async fn rewrite(&self, url: Url) -> Result<Url, RouteError>;
}

#[derive(Clone)]
pub(crate) struct ReqwestUrlRewriter {
    type_name: &'static str,
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
    async fn rewrite(&self, url: Url) -> Result<Url, RouteError> {
        self.rewriter.rewrite(url).await
    }
}
