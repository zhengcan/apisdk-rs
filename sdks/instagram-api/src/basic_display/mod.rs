use apisdk::{
    async_trait, http_api, ApiSignature, Extensions, MiddlewareError, Request, TokenProvider,
};

mod media;
mod oauth;
mod user;

pub use media::*;
pub use oauth::*;
pub use user::*;

#[http_api("https://api.instagram.com/", no_default)]
#[derive(Debug)]
pub struct InstagramBasicDisplayApi {
    secret: Secret,
    api_version: String,
}

#[derive(Debug, Clone)]
pub struct Secret {
    app_id: String,
    app_secret: String,
    access_token: Option<String>,
}

impl Secret {
    pub fn new(app_id: impl ToString, app_secret: impl ToString) -> Self {
        Secret {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            access_token: None,
        }
    }

    pub fn get_access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }
}

#[async_trait]
impl TokenProvider for Secret {
    async fn generate_token(&self, req: &Request) -> Result<String, MiddlewareError> {
        if req.url().path().starts_with("/oauth") {
            return Err(MiddlewareError::Middleware(anyhow::format_err!("No")));
        }

        self.access_token
            .clone()
            .ok_or(MiddlewareError::Middleware(anyhow::format_err!("No")))
    }
}

#[async_trait]
impl ApiSignature for Secret {
    async fn sign(
        &self,
        req: Request,
        _extensions: &Extensions,
    ) -> Result<Request, MiddlewareError> {
        match self.generate_token(&req).await {
            Ok(token) => {
                let mut req = req;
                req.url_mut()
                    .query_pairs_mut()
                    .append_pair("access_token", token.as_str());
                Ok(req)
            }
            Err(_) => Ok(req),
        }
    }
}

impl InstagramBasicDisplayApi {
    pub fn new(secret: Secret, api_version: impl ToString) -> Self {
        Self {
            core: Self::builder().with_signature(secret.clone()).build_core(),
            secret,
            api_version: api_version.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;
    use tracing_log::LogTracer;
    use tracing_subscriber::{
        fmt::{writer::MakeWriterExt, Layer},
        layer::SubscriberExt,
        Registry,
    };

    use crate::{InstagramBasicDisplayApi, Secret};

    pub fn init_logger() {
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

    pub fn create_api() -> InstagramBasicDisplayApi {
        init_logger();
        let mut secret = Secret::new("app_id", "app_secret");
        secret.access_token = Some("access_token".to_string());
        InstagramBasicDisplayApi::new(secret, "v18.0")
    }

    #[tokio::test]
    async fn test_ctor() {
        let _api = create_api();
    }
}
