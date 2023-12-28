use apisdk::http_api;

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
}

#[derive(Debug)]
pub struct Secret {
    app_id: String,
    app_secret: String,
}

impl Secret {
    pub fn new(app_id: impl ToString, app_secret: impl ToString) -> Self {
        Secret {
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
        }
    }
}

impl InstagramBasicDisplayApi {
    pub fn new(secret: Secret) -> Self {
        Self {
            core: Self::builder().build_core(),
            secret,
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
        InstagramBasicDisplayApi::new(Secret::new("app_id", "app_secret"))
    }

    #[tokio::test]
    async fn test_ctor() {
        let _api = create_api();
    }
}
