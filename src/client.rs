use crate::CpanelError;
use reqwest::{
    header::{HeaderMap, HeaderName},
    Client,
};
#[cfg(feature = "tracing")]
use std::time::Instant;
use std::{future::Future, sync::Arc, time::Duration};

/// Authentication method used by the cPanel / WHM client.
///
/// - [`Password`] — Standard Basic auth. Works for both cPanel UAPI and WHM JSON API.
/// - [`Key`] — WHM raw access key auth (`WHM user:key`). Requires a raw key generated
///   in WHM > **Manage Remote Access Key**.
/// - [`Token`] — cPanel API Token auth (`Authorization: cpanel <user>:<token>`).
///   Grants granular, scoped permissions without requiring a full account password
///   or root access. Generate tokens in cPanel > **Home > Security Center > API Tokens**.
///
/// [`Password`]: AuthMethod::Password
/// [`Key`]: AuthMethod::Key
/// [`Token`]: AuthMethod::Token
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Password-based authentication.
    Password {
        /// The account username.
        username: String,
        /// The account password.
        password: String,
    },
    /// WHM raw key authentication.
    Key {
        /// The WHM root username (usually `"root"`).
        username: String,
        /// The raw access key string.
        key: String,
    },
    /// cPanel API Token authentication.
    Token {
        /// The cPanel account username.
        username: String,
        /// The API token string (not the hashed/keyified version).
        token: String,
    },
}

/// Configuration for automatic retry on transient failures.
///
/// When enabled, each API call will be retried up to `max_retries` times with
/// exponential backoff when the server returns a 429 (rate limit) or 503
/// (unavailable) status, or when a connection error occurs.
///
/// # Example
///
/// ```
/// use cpanel_rs::RetryPolicy;
///
/// let policy = RetryPolicy::default()
///     .max_retries(3)
///     .base_delay(std::time::Duration::from_millis(100));
/// ```
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (default: 3).
    pub max_retries: u32,
    /// Base delay between retries in milliseconds (default: 250).
    pub base_delay_ms: u64,
    /// Whether to jitter the delay to avoid thundering herd (default: true).
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 250,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Sets the maximum number of retries.
    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    /// Sets the base delay.
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay_ms = delay.as_millis() as u64;
        self
    }

    /// Enables or disables jitter on retry delays.
    pub fn jitter(mut self, enabled: bool) -> Self {
        self.jitter = enabled;
        self
    }

    /// Computes the delay for a given retry attempt using exponential backoff.
    pub(crate) fn delay(&self, attempt: u32) -> Duration {
        let exp = 2u64.checked_pow(attempt).unwrap_or(u64::MAX);
        let mut delay = self.base_delay_ms.saturating_mul(exp);
        if self.jitter {
            use rand::Rng;
            let jitter: u64 = rand::thread_rng().gen_range(0..delay.max(1));
            delay = delay.saturating_add(jitter);
        }
        Duration::from_millis(delay)
    }

    /// Returns `true` if the error indicates a transient condition worth retrying.
    pub(crate) fn is_retryable(&self, err: &CpanelError) -> bool {
        match err {
            CpanelError::Http(e) => matches!(
                e.status(),
                Some(reqwest::StatusCode::TOO_MANY_REQUESTS)
                    | Some(reqwest::StatusCode::SERVICE_UNAVAILABLE)
                    | Some(reqwest::StatusCode::BAD_GATEWAY)
                    | Some(reqwest::StatusCode::GATEWAY_TIMEOUT)
            ),
            _ => false,
        }
    }
}

/// The cPanel & WHM API client.
///
/// This is the main entry point of the crate. It provides typed wrappers around
/// both the **UAPI** (user-level, port 2083) and **WHM JSON API** (admin-level,
/// port 2087) endpoints.
///
/// # Constructing a Client
///
/// ```
/// use cpanel_rs::CpanelClient;
///
/// // Simple: host, port, username, password
/// let client = CpanelClient::simple("server.example.com", 2087, "root", "mypass").unwrap();
///
/// // Or with the builder for timeouts, TLS options, etc.
/// use cpanel_rs::CpanelBuilder;
/// let client = CpanelBuilder::new("server.example.com", 2087, "root")
///     .password("mypass")
///     .timeout(std::time::Duration::from_secs(30))
///     .build()
///     .unwrap();
/// ```
///
/// # Port Conventions
///
/// | Service | Default Port | Protocol |
/// |---------|-------------|----------|
/// | cPanel UAPI | `2083` | HTTPS (auto-detected) |
/// | WHM JSON API | `2087` | HTTPS (auto-detected) |
///
/// Any other port will use plain HTTP unless you explicitly configure it.
#[derive(Debug, Clone)]
pub struct CpanelClient {
    host: String,
    port: u16,
    use_ssl: bool,
    username: String,
    auth: AuthMethod,
    http_client: Client,
    additional_headers: HeaderMap,
    retry_policy: Arc<RetryPolicy>,
}

impl CpanelClient {
    /// Creates a new client with explicit connection parameters.
    ///
    /// Pass **either** `password` or `key` — both cannot be set at the same time.
    ///
    /// # Arguments
    ///
    /// * `host` — Server hostname or IP address.
    /// * `port` — API port (e.g. `2083` for cPanel, `2087` for WHM).
    /// * `username` — The cPanel/WHM account username.
    /// * `password` — The account password, or `None` if using key auth.
    /// * `key` — A WHM raw access key, or `None` if using password auth.
    /// * `use_ssl` — Set to `true` for HTTPS, `false` for HTTP.
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        password: Option<&str>,
        key: Option<&str>,
        use_ssl: bool,
    ) -> Result<Self, CpanelError> {
        let auth = match (password, key) {
            (Some(p), None) => AuthMethod::Password {
                username: username.to_string(),
                password: p.to_string(),
            },
            (None, Some(k)) => AuthMethod::Key {
                username: username.to_string(),
                key: k.to_string(),
            },
            (Some(_), Some(_)) => {
                return Err(CpanelError::Auth(
                    "Cannot specify both password and key; choose one".into(),
                ))
            }
            (None, None) => {
                return Err(CpanelError::Auth(
                    "Must provide either password or key".into(),
                ))
            }
        };

        let http_client = Client::builder().build().map_err(CpanelError::Http)?;

        Ok(CpanelClient {
            host: host.to_string(),
            port,
            use_ssl,
            username: username.to_string(),
            auth,
            http_client,
            additional_headers: HeaderMap::new(),
            retry_policy: Arc::new(RetryPolicy::default()),
        })
    }

    /// Convenience constructor for password-only auth with auto-detected SSL.
    ///
    /// Uses HTTPS for ports 2083/2087, HTTP otherwise.
    pub fn simple(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Result<Self, CpanelError> {
        Self::new(
            host,
            port,
            username,
            Some(password),
            None,
            port == 2083 || port == 2087,
        )
    }

    /// Creates a client using a pre-built [`reqwest::Client`].
    pub fn with_client(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        client: Client,
    ) -> Self {
        Self {
            host: host.to_string(),
            port,
            use_ssl: port == 2083 || port == 2087,
            username: username.to_string(),
            auth: AuthMethod::Password {
                username: username.to_string(),
                password: password.to_string(),
            },
            http_client: client,
            additional_headers: HeaderMap::new(),
            retry_policy: Arc::new(RetryPolicy::default()),
        }
    }

    /// Returns the server hostname or IP address.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the API port number.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the configured username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the full base URL (e.g. `https://server.example.com:2087`).
    pub fn base_url(&self) -> String {
        let scheme = if self.use_ssl { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }

    /// Creates a client from environment variables.
    ///
    /// Requires the `dotenv` feature to be enabled.
    #[cfg(feature = "dotenv")]
    pub fn from_env() -> Result<Self, CpanelError> {
        dotenv::dotenv().ok();
        let host = std::env::var("CPANEL_HOST")
            .map_err(|_| CpanelError::Auth("CPANEL_HOST not set".into()))?;
        let port: u16 = std::env::var("CPANEL_PORT")
            .map_err(|_| CpanelError::Auth("CPANEL_PORT not set".into()))
            .and_then(|s| {
                s.parse()
                    .map_err(|_| CpanelError::Auth("CPANEL_PORT must be a number".into()))
            })?;
        let username = std::env::var("CPANEL_USER")
            .map_err(|_| CpanelError::Auth("CPANEL_USER not set".into()))?;
        let password = std::env::var("CPANEL_PASSWORD")
            .map_err(|_| CpanelError::Auth("CPANEL_PASSWORD not set".into()))?;
        Self::new(&host, port, &username, Some(&password), None, true)
    }

    /// Sets the retry policy for this client.
    ///
    /// When enabled, transient errors (429, 503, connection failures) will be
    /// retried up to [`RetryPolicy::max_retries`] times with exponential backoff.
    ///
    /// # Example
    ///
    /// ```
    /// use cpanel_rs::{CpanelClient, RetryPolicy};
    ///
    /// let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
    /// let client = client.with_retry_policy(
    ///     RetryPolicy::default()
    ///         .max_retries(5)
    ///         .base_delay(std::time::Duration::from_millis(500)),
    /// );
    /// ```
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Arc::new(policy);
        self
    }

    /// Returns the current retry policy.
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    /// Returns an auto-paginating async stream over all WHM accounts.
    ///
    /// Requires the `stream` feature. Each item is a [`AccountInfo`](crate::models::AccountInfo).
    /// The stream fetches pages of 50 accounts at a time until exhaustion.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::CpanelClient;
    /// use futures::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
    ///     let mut stream = client.stream_accounts();
    ///     while let Some(acct) = stream.next().await {
    ///         let acct = acct?;
    ///         println!("{} @ {}", acct.username, acct.domain);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[cfg(feature = "stream")]
    pub fn stream_accounts(&self) -> crate::streams::AccountsStream {
        crate::streams::AccountsStream::new(self)
    }

    /// Returns an async stream over file entries in a directory.
    ///
    /// Requires the `stream` feature. Each item is a
    /// [`FileManagerEntry`](crate::models::FileManagerEntry).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cpanel_rs::CpanelClient;
    /// use futures::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cpanel_rs::CpanelError> {
    ///     let client = CpanelClient::simple("server.example.com", 2083, "user", "pass").unwrap();
    ///     let mut stream = client.stream_files("user", "/public_html");
    ///     while let Some(entry) = stream.next().await {
    ///         let entry = entry?;
    ///         println!("{} ({})", entry.name, entry.type_);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[cfg(feature = "stream")]
    pub fn stream_files(&self, user: &str, path: &str) -> crate::streams::FilesStream {
        crate::streams::FilesStream::new(self, user, path)
    }

    /// Sends a raw UAPI request and returns the deserialized response.
    pub async fn uapi<D: for<'de> serde::Deserialize<'de>>(
        &self,
        module: &str,
        func: &str,
        params: &[(&str, &str)],
    ) -> Result<D, CpanelError> {
        let result = self.send_uapi(module, func, params).await?;
        let call = format!("{}.{}", module, func);

        if let Some(ref errors) = result.errors {
            let failure = crate::types::ApiFailure {
                call: call.clone(),
                errors: errors.clone(),
                messages: result.messages.clone().unwrap_or_default(),
                result_code: result.metadata.as_ref().and_then(|m| m.result),
                reason: result.metadata.as_ref().and_then(|m| m.reason.clone()),
                ..Default::default()
            };
            if !failure.has_errors() {
                return Err(CpanelError::ApiWarning(failure));
            }
            return Err(CpanelError::ApiError(failure));
        }

        if let Some(ref warnings) = result.messages {
            if !warnings.is_empty() {
                let failure = crate::types::ApiFailure {
                    call,
                    warnings: warnings.clone(),
                    result_code: result.metadata.as_ref().and_then(|m| m.result),
                    reason: result.metadata.as_ref().and_then(|m| m.reason.clone()),
                    ..Default::default()
                };
                return Err(CpanelError::ApiWarning(failure));
            }
        }

        result
            .data
            .ok_or_else(|| CpanelError::InvalidResponse("No data in UAPI response".into()))
    }

    /// Sends a raw WHM API request and returns the deserialized response.
    pub async fn whm<D: for<'de> serde::Deserialize<'de>>(
        &self,
        function: &str,
        params: &[(&str, &str)],
    ) -> Result<D, CpanelError> {
        let result = self.send_whm(function, params).await?;
        let call = format!("whm.{}", function);

        if result.status != 1 {
            let failure = crate::types::ApiFailure {
                call: call.clone(),
                message: result
                    .errors
                    .as_ref()
                    .and_then(|e| e.first().cloned())
                    .unwrap_or_else(|| result.metadata.reason.clone().unwrap_or_default()),
                errors: result.errors.unwrap_or_default(),
                messages: result.messages.unwrap_or_default(),
                whm_result: Some(result.metadata.result),
                whm_status: Some(result.metadata.status),
                reason: result.metadata.reason.clone(),
                ..Default::default()
            };
            return Err(CpanelError::ApiError(failure));
        }

        if let Some(ref warnings) = result.messages {
            if !warnings.is_empty() {
                let failure = crate::types::ApiFailure {
                    call,
                    warnings: warnings.clone(),
                    whm_result: Some(result.metadata.result),
                    whm_status: Some(result.metadata.status),
                    reason: result.metadata.reason.clone(),
                    ..Default::default()
                };
                return Err(CpanelError::ApiWarning(failure));
            }
        }

        result
            .data
            .ok_or_else(|| CpanelError::InvalidResponse("No data in WHM response".into()))
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn build_url(&self, path: &str) -> String {
        let scheme = if self.use_ssl { "https" } else { "http" };
        format!("{}://{}:{}{}", scheme, self.host, self.port, path)
    }

    pub(crate) fn auth_headers(&self) -> HeaderMap {
        use base64::engine::{general_purpose::STANDARD, Engine};
        let mut headers = self.additional_headers.clone();

        match &self.auth {
            AuthMethod::Password { username, password } => {
                let cred = STANDARD.encode(format!("{}:{}", username, password));
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("Basic {}", cred).parse().unwrap(),
                );
            }
            AuthMethod::Key { username, key } => {
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("WHM {}:{}", username, key).parse().unwrap(),
                );
            }
            AuthMethod::Token { username, token } => {
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("cpanel {}:{}", username, token).parse().unwrap(),
                );
            }
        }

        headers
    }

    pub(crate) fn http_client(&self) -> &Client {
        &self.http_client
    }

    async fn do_request(&self, url: String) -> Result<(reqwest::StatusCode, String), CpanelError> {
        #[cfg(feature = "tracing")]
        let start = Instant::now();
        #[cfg(feature = "tracing")]
        tracing::debug!(url = %url, "sending request");
        let resp = self
            .http_client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            status = ?status,
            latency_ms = %start.elapsed().as_millis(),
            "request completed"
        );
        Ok((status, body))
    }

    async fn send_with_retry<F, Fut, R>(&self, request_fn: F) -> Result<R, CpanelError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<R, CpanelError>>,
    {
        let policy = self.retry_policy.clone();
        let mut attempt = 0u32;

        loop {
            match request_fn().await {
                Ok(val) => return Ok(val),
                Err(e) if attempt < policy.max_retries && policy.is_retryable(&e) => {
                    #[cfg(feature = "tracing")]
                    tracing::warn!(
                        attempt,
                        max_retries = policy.max_retries,
                        error = ?e,
                        "retrying after transient error"
                    );
                    tokio::time::sleep(policy.delay(attempt)).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn send_uapi<T: for<'de> serde::Deserialize<'de>>(
        &self,
        module: &str,
        func: &str,
        params: &[(&str, &str)],
    ) -> Result<crate::types::UapiResult<T>, CpanelError> {
        let mut url = self.build_url(&format!(
            "/json-api/{}?cpanel_jsonapi_module={}&cpanel_jsonapi_apiversion=2&cpanel_jsonapi_func={}",
            self.username, module, func
        ));
        for (k, v) in params {
            url.push_str(&format!("&{}={}", k, urlencoding::encode(v)));
        }

        let (status, body) = self
            .send_with_retry(|| {
                let url = url.clone();
                async move { self.do_request(url).await }
            })
            .await?;

        if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(CpanelError::ApiError(crate::types::ApiFailure {
                message: format!("HTTP {}", status),
                call: format!("{}.{}", module, func),
                ..Default::default()
            }));
        }

        let api_resp: crate::types::ApiResponse<crate::types::UapiResult<T>> =
            serde_json::from_str(&body).map_err(CpanelError::Serialization)?;

        if !api_resp.is_ok() {
            return Err(CpanelError::ApiError(crate::types::ApiFailure {
                message: api_resp.status_msg,
                call: format!("{}.{}", module, func),
                ..Default::default()
            }));
        }

        api_resp
            .data
            .ok_or_else(|| CpanelError::InvalidResponse("No data".into()))
    }

    async fn send_whm<T: for<'de> serde::Deserialize<'de>>(
        &self,
        function: &str,
        params: &[(&str, &str)],
    ) -> Result<crate::types::WhmApiResponse<T>, CpanelError> {
        let mut url = self.build_url(&format!(
            "/json-api/whmapi1?api.version=1&function={}",
            function
        ));
        for (k, v) in params {
            url.push_str(&format!("&{}={}", k, urlencoding::encode(v)));
        }

        let (status, body) = self
            .send_with_retry(|| {
                let url = url.clone();
                async move { self.do_request(url).await }
            })
            .await?;

        if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(CpanelError::ApiError(crate::types::ApiFailure {
                message: format!("HTTP {}", status),
                call: format!("whm.{}", function),
                ..Default::default()
            }));
        }

        let api_resp: crate::types::ApiResponse<crate::types::WhmApiResponse<T>> =
            serde_json::from_str(&body).map_err(CpanelError::Serialization)?;

        if !api_resp.is_ok() {
            return Err(CpanelError::ApiError(crate::types::ApiFailure {
                message: api_resp.status_msg,
                call: format!("whm.{}", function),
                ..Default::default()
            }));
        }

        api_resp
            .data
            .ok_or_else(|| CpanelError::InvalidResponse("No data".into()))
    }
}

/// Builder for constructing a [`CpanelClient`] with full control over connection settings.
///
/// # Example
///
/// ```no_run
/// use cpanel_rs::CpanelBuilder;
///
/// let client = CpanelBuilder::new("server.example.com", 2087, "root")
///     .password("secret")
///     .timeout(std::time::Duration::from_secs(30))
///     .disable_tls_verification()
///     .header("X-Request-ID", "abc-123")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct CpanelBuilder {
    host: String,
    port: u16,
    username: String,
    auth: AuthMethod,
    timeout: Option<Duration>,
    disable_tls_verify: bool,
    additional_headers: Vec<(String, String)>,
}

impl CpanelBuilder {
    /// Creates a new builder.
    ///
    /// After calling `new()`, chain either [`Self::password`], [`Self::key`], or [`Self::token`]
    /// before calling [`Self::build`].
    pub fn new(host: &str, port: u16, username: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            username: username.to_string(),
            auth: AuthMethod::Password {
                username: username.to_string(),
                password: String::new(),
            },
            timeout: None,
            disable_tls_verify: false,
            additional_headers: Vec::new(),
        }
    }

    /// Sets password-based authentication.
    pub fn password(mut self, password: &str) -> Self {
        self.auth = AuthMethod::Password {
            username: self.username.clone(),
            password: password.to_string(),
        };
        self
    }

    /// Sets WHM raw key authentication.
    pub fn key(mut self, key: &str) -> Self {
        self.auth = AuthMethod::Key {
            username: self.username.clone(),
            key: key.to_string(),
        };
        self
    }

    /// Sets cPanel API Token authentication.
    ///
    /// Generate tokens in cPanel > **Home > Security Center > API Tokens**.
    /// The raw/unhashed token string must be passed here.
    pub fn token(mut self, token: &str) -> Self {
        self.auth = AuthMethod::Token {
            username: self.username.clone(),
            token: token.to_string(),
        };
        self
    }

    /// Sets the HTTP request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Disables TLS certificate and hostname verification.
    ///
    /// **Only** use this when connecting to servers with self-signed certificates.
    pub fn disable_tls_verification(mut self) -> Self {
        self.disable_tls_verify = true;
        self
    }

    /// Adds a custom HTTP header to every request.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.additional_headers
            .push((key.to_string(), value.to_string()));
        self
    }

    /// Builds the [`CpanelClient`].
    pub fn build(self) -> Result<CpanelClient, CpanelError> {
        let use_ssl = self.port == 2083 || self.port == 2087;

        let http_client = {
            let mut builder = Client::builder();
            if let Some(to) = self.timeout {
                builder = builder.timeout(to);
            }
            if self.disable_tls_verify {
                builder = builder.danger_accept_invalid_certs(true);
            }
            builder.build().map_err(CpanelError::Http)?
        };

        let mut additional_headers = HeaderMap::new();
        for (k, v) in self.additional_headers {
            let name: HeaderName = k.parse().unwrap();
            let value: reqwest::header::HeaderValue = v.parse().unwrap();
            additional_headers.insert(name, value);
        }

        Ok(CpanelClient {
            host: self.host,
            port: self.port,
            use_ssl,
            username: self.username,
            auth: self.auth,
            http_client,
            additional_headers,
            retry_policy: Arc::new(RetryPolicy::default()),
        })
    }
}

pub use crate::types::{
    ApiFailure, ApiResponse, CpanelResult, Metadata, StackFrame, UapiResult, WhmApiResponse,
    WhmMetadata,
};
