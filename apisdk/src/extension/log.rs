use reqwest_middleware::{RequestBuilder, RequestInitialiser};

/// This struct is used to control how to log
#[derive(Debug, Default, Clone)]
pub struct LogConfig {
    /// Enable logs
    pub enabled: bool,
}

impl LogConfig {
    /// Construct a new instance
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Construct a new instance to enable all logs
    pub fn enabled_all() -> Self {
        Self { enabled: true }
    }
}

impl RequestInitialiser for LogConfig {
    fn init(&self, req: RequestBuilder) -> RequestBuilder {
        let mut req = req;
        if req.extensions().contains::<LogConfig>() {
            req
        } else {
            req.with_extension(self.clone())
        }
    }
}
