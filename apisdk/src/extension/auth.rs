use std::{any::type_name, num::ParseIntError, string::FromUtf8Error, sync::Arc, time::SystemTime};

use async_trait::async_trait;
use base64::DecodeError;
use reqwest::{
    header::{HeaderName, HeaderValue, AUTHORIZATION},
    Request, Response,
};
use reqwest_middleware::Next;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    digest::{self, decode_base64},
    Extensions, Middleware,
};

/// This middleware is used to authenticate the request
#[derive(Default)]
pub(crate) struct AuthenticateMiddleware;

#[async_trait]
impl Middleware for AuthenticateMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        let mut req = req;

        // Sign the request by using ApiAuthenticator
        if let Some(signatue) = extensions.get::<Arc<dyn ApiAuthenticator>>() {
            req = signatue.authenticate(req, extensions).await?;
        }

        next.run(req, extensions).await
    }
}

/// This trait is used to generate token
#[async_trait]
pub trait TokenGenerator: 'static + Send + Sync {
    /// Generate a new token
    async fn generate_token(&self, req: &Request) -> Result<String, reqwest_middleware::Error>;
}

#[async_trait]
impl<F, T> TokenGenerator for F
where
    F: 'static + Send + Sync,
    F: Fn() -> Result<T, reqwest_middleware::Error>,
    T: ToString,
{
    async fn generate_token(&self, _req: &Request) -> Result<String, reqwest_middleware::Error> {
        self().map(|t| t.to_string())
    }
}

/// This trait is used to authenticate request
#[async_trait]
pub trait ApiAuthenticator: TokenGenerator {
    /// Get type_name, used in Debug
    fn type_name(&self) -> &str {
        type_name::<Self>()
    }

    /// Get `Carrier`
    fn get_carrier(&self) -> &Carrier {
        &Carrier::BearerAuth
    }

    /// Authenticate request
    /// - req: HTTP request
    /// - extensions: Extensions
    async fn authenticate(
        &self,
        req: Request,
        _extensions: &Extensions,
    ) -> Result<Request, reqwest_middleware::Error> {
        let token = self.generate_token(&req).await?;
        Ok(self.get_carrier().apply(req, token))
    }
}

#[async_trait]
impl TokenGenerator for Box<dyn ApiAuthenticator> {
    async fn generate_token(&self, req: &Request) -> Result<String, reqwest_middleware::Error> {
        self.as_ref().generate_token(req).await
    }
}

#[async_trait]
impl ApiAuthenticator for Box<dyn ApiAuthenticator> {
    fn get_carrier(&self) -> &Carrier {
        self.as_ref().get_carrier()
    }

    async fn authenticate(
        &self,
        req: Request,
        extensions: &Extensions,
    ) -> Result<Request, reqwest_middleware::Error> {
        self.as_ref().authenticate(req, extensions).await
    }
}

/// This trait is used to update carrier
pub trait WithCarrier {
    /// Update instance to use `Carrier`
    fn with_carrier(self, carrier: Carrier) -> Self;

    /// Update instance to use `Header`
    /// - name: the name of header
    fn with_header_name(self, name: impl ToString) -> Self;

    /// Update instance to use `QueryParam`
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
    SchemalessAuth,
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
            Carrier::SchemalessAuth => {
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
    Dynamic(Arc<dyn TokenGenerator>),
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
pub struct AccessTokenAuth {
    access_token: AccessToken,
    carrier: Carrier,
}

impl AccessTokenAuth {
    /// Build for immutable token
    pub fn new(access_token: impl ToString) -> Self {
        Self {
            access_token: AccessToken::Fixed(access_token.to_string()),
            carrier: Carrier::default(),
        }
    }

    /// Build for dynamic token
    pub fn new_dynamic(provider: impl TokenGenerator) -> Self {
        Self {
            access_token: AccessToken::Dynamic(Arc::new(provider)),
            carrier: Carrier::default(),
        }
    }
}

#[async_trait]
impl ApiAuthenticator for AccessTokenAuth {
    fn get_carrier(&self) -> &Carrier {
        &self.carrier
    }
}

#[async_trait]
impl TokenGenerator for AccessTokenAuth {
    async fn generate_token(&self, req: &Request) -> Result<String, reqwest_middleware::Error> {
        match &self.access_token {
            AccessToken::Fixed(token) => Ok(token.clone()),
            AccessToken::Dynamic(provider) => provider.generate_token(req).await,
        }
    }
}

impl WithCarrier for AccessTokenAuth {
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
///
/// # Parse token
///
/// ```rust
/// let token = "xxx";
/// let parsed_token = ParsedHashedToken::parse(token)?;
/// if parsed_token.is_expired(60 * 5, None) {
///     // Expired Token
/// } else if parsed_token.is_signed("my-secret-key", HashAlgorithm::MD5) {
///     // Invalid Token
/// }
/// ```
#[derive(Debug)]
pub struct HashedTokenAuth {
    client_id: Option<String>,
    app_id: String,
    app_secret: String,
    algorithm: HashAlgorithm,
    carrier: Carrier,
}

impl HashedTokenAuth {
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
impl ApiAuthenticator for HashedTokenAuth {
    fn get_carrier(&self) -> &Carrier {
        &self.carrier
    }
}

#[async_trait]
impl TokenGenerator for HashedTokenAuth {
    async fn generate_token(&self, _req: &Request) -> Result<String, reqwest_middleware::Error> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(self.generate_token_at(timestamp))
    }
}

impl WithCarrier for HashedTokenAuth {
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

/// Token Error
#[derive(Debug, Error)]
pub enum TokenError {
    /// Base64 decode error
    #[error("{0}")]
    Base64(#[from] DecodeError),
    /// Utf8 decode error
    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),
    /// Invalid format
    #[error("Invalid format")]
    Format,
    /// Invalid timestamp
    #[error("{0}")]
    Timestamp(#[from] ParseIntError),
}

/// This struct is used to parse token
#[derive(Debug)]
pub struct ParsedHashedToken {
    /// client_id
    pub client_id: Option<String>,
    /// app_id
    pub app_id: String,
    /// timestamp, in second
    pub timestamp: u64,
    /// sign
    pub sign: String,
}

impl ParsedHashedToken {
    /// Parse the token
    pub fn parse(token: impl AsRef<[u8]>) -> Result<Self, TokenError> {
        let token = token.as_ref();
        if token.is_empty() {
            return Err(TokenError::Format);
        }

        let composed = decode_base64(token)
            .map_err(|e| TokenError::Base64(e))
            .and_then(|b| String::from_utf8(b).map_err(|e| TokenError::Utf8(e)))?;
        let terms: Vec<&str> = composed.split(',').collect();
        let mut iter = terms.iter();
        match terms.len() {
            4 => Ok(Self {
                client_id: Some(iter.next().unwrap().to_string()),
                app_id: iter.next().unwrap().to_string(),
                timestamp: iter
                    .next()
                    .unwrap()
                    .parse()
                    .map_err(|e| TokenError::Timestamp(e))?,
                sign: iter.next().unwrap().to_string(),
            }),
            3 => Ok(Self {
                client_id: None,
                app_id: iter.next().unwrap().to_string(),
                timestamp: iter
                    .next()
                    .unwrap()
                    .parse()
                    .map_err(|e| TokenError::Timestamp(e))?,
                sign: iter.next().unwrap().to_string(),
            }),
            _ => Err(TokenError::Format),
        }
    }

    /// Check the token is expired or not
    /// - deviation: 1 min as default
    pub fn is_expired(&self, expires_in_secs: u64, deviation: Option<u64>) -> bool {
        let deviation = deviation.unwrap_or(60) as i64;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let diff = now as i64 - self.timestamp as i64;
        diff < -deviation || diff > expires_in_secs as i64 + deviation
    }

    /// Check the token is signed or not
    pub fn is_signed<S, A>(&self, app_secret: S, algorithm: A) -> bool
    where
        S: std::fmt::Display,
        A: Into<HashAlgorithm>,
    {
        // Sign
        let plain = format!("{}{}{}", self.app_id, app_secret, self.timestamp);
        let algorithm: HashAlgorithm = algorithm.into();
        let sign = algorithm.apply(plain);

        sign == self.sign
    }
}

impl TryFrom<&str> for ParsedHashedToken {
    type Error = TokenError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<String> for ParsedHashedToken {
    type Error = TokenError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}
