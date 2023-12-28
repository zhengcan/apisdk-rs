use std::{any::type_name, sync::Arc, time::SystemTime};

use async_trait::async_trait;
use reqwest::{
    header::{HeaderName, HeaderValue, AUTHORIZATION},
    Request, Response,
};
use reqwest_middleware::Next;
use serde::{Deserialize, Serialize};

use crate::{digest, Extensions, Middleware};

/// This middleware is used to sign the request
#[derive(Default)]
pub(crate) struct SignatureMiddleware;

#[async_trait]
impl Middleware for SignatureMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        let mut req = req;

        // Sign the request by using ApiSignature
        if let Some(signatue) = extensions.get::<Arc<dyn ApiSignature>>() {
            req = signatue.sign(req, extensions).await?;
        }

        next.run(req, extensions).await
    }
}

/// This trait is used to generate token
#[async_trait]
pub trait TokenProvider: 'static + Send + Sync {
    /// Generate a new token
    async fn generate_token(&self, req: &Request) -> Result<String, reqwest_middleware::Error>;
}

#[async_trait]
impl<F, T> TokenProvider for F
where
    F: 'static + Send + Sync,
    F: Fn() -> Result<T, reqwest_middleware::Error>,
    T: ToString,
{
    async fn generate_token(&self, _req: &Request) -> Result<String, reqwest_middleware::Error> {
        self().map(|t| t.to_string())
    }
}

/// This trait is used to sign request
#[async_trait]
pub trait ApiSignature: TokenProvider {
    /// Get type_name, used in Debug
    fn type_name(&self) -> &str {
        type_name::<Self>()
    }

    /// Get `Carrier`
    fn get_carrier(&self) -> &Carrier {
        &Carrier::BearerAuth
    }

    /// Sign request
    /// - req: HTTP request
    /// - extensions: Extensions
    async fn sign(
        &self,
        req: Request,
        _extensions: &Extensions,
    ) -> Result<Request, reqwest_middleware::Error> {
        let token = self.generate_token(&req).await?;
        Ok(self.get_carrier().apply(req, token))
    }
}

#[async_trait]
impl TokenProvider for Box<dyn ApiSignature> {
    async fn generate_token(&self, req: &Request) -> Result<String, reqwest_middleware::Error> {
        self.as_ref().generate_token(req).await
    }
}

#[async_trait]
impl ApiSignature for Box<dyn ApiSignature> {
    fn get_carrier(&self) -> &Carrier {
        self.as_ref().get_carrier()
    }

    async fn sign(
        &self,
        req: Request,
        extensions: &Extensions,
    ) -> Result<Request, reqwest_middleware::Error> {
        self.as_ref().sign(req, extensions).await
    }
}

/// This trait is used to update signature
pub trait SignatureTrait {
    /// Update signature to use `Carrier`
    fn with_carrier(self, carrier: Carrier) -> Self;

    /// Update signature to use `Header`
    /// - name: the name of header
    fn with_header_name(self, name: impl ToString) -> Self;

    /// Update signature to use `QueryParam`
    /// - name: the name of query param
    fn with_query_param(self, name: impl ToString) -> Self;
}

/// This enum represents the position of request to carry token.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum Carrier {
    /// Standard `Authorization` header, with `Bearer` auth-scheme
    #[default]
    BearerAuth,
    /// Standard `Authorization` header, without any auth-scheme
    SchemelessAuth,
    /// Customized header
    Header(String),
    /// Customized query param
    QueryParam(String),
}

impl Carrier {
    /// Apply the changes to request
    pub fn apply(&self, req: Request, token: impl ToString) -> Request {
        let mut req = req;
        let token = token.to_string();
        match self {
            Carrier::BearerAuth => {
                req.headers_mut().insert(
                    AUTHORIZATION,
                    HeaderValue::try_from(format!("Bearer {}", token)).unwrap(),
                );
            }
            Carrier::SchemelessAuth => {
                req.headers_mut()
                    .insert(AUTHORIZATION, HeaderValue::try_from(token).unwrap());
            }
            Carrier::Header(name) => {
                req.headers_mut().append(
                    HeaderName::try_from(name.as_str()).unwrap(),
                    HeaderValue::try_from(token).unwrap(),
                );
            }
            Carrier::QueryParam(name) => {
                req.url_mut()
                    .query_pairs_mut()
                    .append_pair(name.as_str(), &token);
            }
        }
        req
    }
}

/// This enum holds `access_token`, which used to sign request
pub enum AccessToken {
    /// Immutable token
    Fixed(String),
    /// Dynamic token, which will be retrieved from provider
    Dynamic(Arc<dyn TokenProvider>),
}

impl std::fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(_) => f.debug_tuple("Fixed").finish(),
            Self::Dynamic(_) => f.debug_tuple("Dynamic").finish(),
        }
    }
}

/// This struct is used to sign request by using `access_token`
#[derive(Debug)]
pub struct AccessTokenSignature {
    access_token: AccessToken,
    carrier: Carrier,
}

impl AccessTokenSignature {
    /// Build for immutable token
    pub fn new(access_token: impl ToString) -> Self {
        Self {
            access_token: AccessToken::Fixed(access_token.to_string()),
            carrier: Carrier::default(),
        }
    }

    /// Build for dynamic token
    pub fn new_dynamic(provider: impl TokenProvider) -> Self {
        Self {
            access_token: AccessToken::Dynamic(Arc::new(provider)),
            carrier: Carrier::default(),
        }
    }
}

#[async_trait]
impl ApiSignature for AccessTokenSignature {
    fn get_carrier(&self) -> &Carrier {
        &self.carrier
    }
}

#[async_trait]
impl TokenProvider for AccessTokenSignature {
    async fn generate_token(&self, req: &Request) -> Result<String, reqwest_middleware::Error> {
        match &self.access_token {
            AccessToken::Fixed(token) => Ok(token.clone()),
            AccessToken::Dynamic(provider) => provider.generate_token(req).await,
        }
    }
}

impl SignatureTrait for AccessTokenSignature {
    fn with_carrier(self, carrier: Carrier) -> Self {
        Self { carrier, ..self }
    }

    fn with_header_name(self, name: impl ToString) -> Self {
        Self {
            carrier: Carrier::Header(name.to_string()),
            ..self
        }
    }

    fn with_query_param(self, name: impl ToString) -> Self {
        Self {
            carrier: Carrier::QueryParam(name.to_string()),
            ..self
        }
    }
}

/// Hash algorithm
#[derive(Debug)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
}

impl HashAlgorithm {
    /// Calc hash value
    pub fn apply(&self, input: impl AsRef<[u8]>) -> String {
        match self {
            Self::Md5 => digest::md5(input),
            Self::Sha1 => digest::sha1(input),
            Self::Sha256 => digest::sha256(input),
        }
    }
}

impl From<String> for HashAlgorithm {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<&str> for HashAlgorithm {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sha1" => Self::Sha1,
            "md5" => Self::Md5,
            "sha256" => Self::Sha256,
            _ => Self::Sha1,
        }
    }
}

/// This struct is used to sign request by hashed token.
///
/// # Generate token algorithm
///
/// ```
/// hash = md5 | sha1(default) | sha256
/// timestamp = UNIX_TIMESTAMP (in second)
/// sign = hash($app_id + $app_secret + $timestamp)
/// token = base64($client_id + "," + $app_id + "," + $timestamp + "," + $sign)
///       = (or) base64($app_id + "," + $timestamp + "," + $sign)
/// ```
#[derive(Debug)]
pub struct HashedTokenSignature {
    client_id: Option<String>,
    app_id: String,
    app_secret: String,
    algorithm: HashAlgorithm,
    carrier: Carrier,
}

impl HashedTokenSignature {
    pub fn new<S: ToString>(app_id: S, app_secret: S) -> Self {
        Self::new_with_algorithm(app_id, app_secret, HashAlgorithm::Sha1)
    }

    pub fn new_with_algorithm<S: ToString>(
        app_id: S,
        app_secret: S,
        algorithm: HashAlgorithm,
    ) -> Self {
        Self {
            client_id: None,
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            algorithm,
            carrier: Carrier::default(),
        }
    }

    pub fn new_with_client_id<S: ToString>(
        client_id: S,
        app_id: S,
        app_secret: S,
        algorithm: HashAlgorithm,
    ) -> Self {
        Self {
            client_id: match client_id.to_string() {
                id if id.is_empty() => None,
                id => Some(id),
            },
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            algorithm,
            carrier: Carrier::default(),
        }
    }

    /// Generate token
    fn generate_token_at(&self, timestamp: u64) -> String {
        // Hash
        let plain = format!("{}{}{}", &self.app_id, &self.app_secret, timestamp);
        let sign = self.algorithm.apply(plain);

        // Compose
        let compose = match &self.client_id {
            Some(client_id) => format!("{},{},{},{}", client_id, &self.app_id, timestamp, sign),
            None => format!("{},{},{}", &self.app_id, timestamp, sign),
        };
        digest::encode_base64(compose)
    }
}

#[async_trait]
impl ApiSignature for HashedTokenSignature {
    fn get_carrier(&self) -> &Carrier {
        &self.carrier
    }
}

#[async_trait]
impl TokenProvider for HashedTokenSignature {
    async fn generate_token(&self, _req: &Request) -> Result<String, reqwest_middleware::Error> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(self.generate_token_at(timestamp))
    }
}

impl SignatureTrait for HashedTokenSignature {
    fn with_carrier(self, carrier: Carrier) -> Self {
        Self { carrier, ..self }
    }

    fn with_header_name(self, name: impl ToString) -> Self {
        Self {
            carrier: Carrier::Header(name.to_string()),
            ..self
        }
    }

    fn with_query_param(self, name: impl ToString) -> Self {
        Self {
            carrier: Carrier::QueryParam(name.to_string()),
            ..self
        }
    }
}
