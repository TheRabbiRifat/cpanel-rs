//! Core types shared across the crate.
//!
//! This module defines the error type [`CpanelError`], the JSON response
//! structures for UAPI and WHM, and the [`QueryParams`](crate::types::QueryParams) builder used for
//! paginated list operations.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Detailed failure information from a cPanel/WHM API call.
///
/// This struct collects every piece of diagnostic data the API returns on error
/// — messages, warnings, stack traces, and metadata — so callers can inspect
/// the root cause programmatically rather than relying only on display text.
#[derive(Debug, Clone, Default)]
pub struct ApiFailure {
    /// Primary error message from the API.
    pub message: String,
    /// Which API call failed (e.g. `"DNS.list_zone_records"`).
    pub call: String,
    /// All module-level error messages.
    pub errors: Vec<String>,
    /// Warning messages returned alongside the response.
    pub warnings: Vec<String>,
    /// General informational messages.
    pub messages: Vec<String>,
    /// UAPI result code (`None` for WHM calls).
    pub result_code: Option<i32>,
    /// UAPI reason string.
    pub reason: Option<String>,
    /// WHM result code.
    pub whm_result: Option<i32>,
    /// WHM status code.
    pub whm_status: Option<i32>,
    /// Stack trace frames, if any.
    pub stack: Vec<String>,
}

impl ApiFailure {
    /// Returns `true` if this failure contains any error messages.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty() || !self.message.is_empty()
    }

    /// Returns `true` if there are warning messages (but no errors).
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl std::fmt::Display for ApiFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.call, self.message)?;
        if !self.errors.is_empty() {
            write!(f, " (errors: {})", self.errors.join("; "))?;
        }
        if !self.warnings.is_empty() {
            write!(f, " (warnings: {})", self.warnings.join("; "))?;
        }
        if let Some(reason) = &self.reason {
            write!(f, " — {reason}")?;
        }
        Ok(())
    }
}

/// The error type returned by all cPanel/WHM API operations.
///
/// # Variants
///
/// | Variant | Meaning |
/// |---------|---------|
/// | `Http` | Network error, connection failure, or request timeout |
/// | `ApiError` | The API returned a non-success response |
/// | `ApiWarning` | The API returned success but with non-empty warnings |
/// | `Auth` | Missing or invalid credentials |
/// | `InvalidResponse` | API returned success status but no data payload |
/// | `Serialization` | JSON parsing error in the response |
/// | `Io` | Generic I/O error |
///
/// # Example
///
/// ```
/// use cpanel_rs::CpanelError;
///
/// // Pattern-match on error types
/// async fn handle() -> Result<(), CpanelError> {
///     Err(CpanelError::Auth("CPANEL_HOST not set".into()))
/// }
/// ```
#[derive(Error, Debug)]
pub enum CpanelError {
    /// HTTP / network error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// The API returned an error status.
    ///
    /// [`CpanelError::ApiError`] always contains a fully parsed [`ApiFailure`]
    /// that exposes every field the server sent back.
    #[error("{0}")]
    ApiError(ApiFailure),

    /// The API call succeeded but returned non-empty warnings.
    ///
    /// Inspect the [`ApiFailure::warnings`] field for details.
    #[error("{0}")]
    ApiWarning(ApiFailure),

    /// Authentication failed or required credentials are missing.
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// The API response was valid JSON but missing the expected data field.
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// JSON deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// The top-level JSON envelope returned by all cPanel/WHM API responses.
///
/// Every response wraps the actual data inside this struct. The `status` field
/// is `1` on success and `0` on error.
///
/// # Fields
///
/// * `status` — `1` for success, `0` for failure
/// * `status_msg` — Human-readable status message
/// * `data` — The parsed response payload (generic)
/// * `errors` — Optional list of error messages
/// * `warnings` — Optional list of warning messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// `1` on success, `0` on failure.
    pub status: i32,
    /// Human-readable status message from the API.
    pub status_msg: String,
    /// The parsed response payload. `None` if the request failed.
    pub data: Option<T>,
    /// Error messages, if any.
    pub errors: Option<Vec<String>>,
    /// Warning messages, if any.
    pub warnings: Option<Vec<String>>,
}

impl<T> ApiResponse<T> {
    /// Returns `true` if the API call was successful (`status == 1`).
    pub fn is_ok(&self) -> bool {
        self.status == 1
    }

    /// Converts the response into a [`Result`], extracting the data or an error.
    ///
    /// # Example
    ///
    /// ```
    /// use cpanel_rs::types::ApiResponse;
    ///
    /// let resp: ApiResponse<String> = ApiResponse {
    ///     status: 1,
    ///     status_msg: "OK".into(),
    ///     data: Some("hello".into()),
    ///     errors: None,
    ///     warnings: None,
    /// };
    /// assert_eq!(resp.into_result().unwrap(), "hello");
    /// ```
    pub fn into_result(self) -> Result<T, CpanelError> {
        if self.status != 1 {
            let msg = self
                .errors
                .as_ref()
                .and_then(|e| e.first().cloned())
                .unwrap_or_else(|| self.status_msg.clone());
            let mut failure = ApiFailure {
                message: msg,
                ..Default::default()
            };
            failure.errors = self.errors.unwrap_or_default();
            failure.warnings = self.warnings.unwrap_or_default();
            return Err(CpanelError::ApiError(failure));
        }
        self.data
            .ok_or_else(|| CpanelError::InvalidResponse("No data in response".into()))
    }
}

/// Internal struct: the outer wrapper around a cPanel UAPI response.
///
/// The cPanel JSON API wraps every response in a `cpanelresult` object.
/// This struct represents that envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpanelResult<T> {
    /// The action result containing data, errors, and stack traces.
    pub cpanelresult: ActionResult<T>,
}

/// The `cpanelresult` payload from a cPanel UAPI response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult<T> {
    /// The result data, if any.
    pub data: Option<Vec<T>>,
    /// Error message, if the action failed.
    pub error: Option<String>,
    /// Stack trace frames, if an error occurred.
    pub stack: Option<Vec<StackFrame>>,
    /// The result section name (e.g. `"result"`).
    pub section: Option<String>,
}

/// A single frame in a cPanel UAPI stack trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function arguments.
    pub args: Option<Vec<String>>,
    /// Function name.
    pub func: Option<String>,
    /// Module name.
    pub module: Option<String>,
}

/// The UAPI response envelope.
///
/// Unlike the WHM response, UAPI responses include a `metadata` object with
/// result codes and optional messages.
///
/// # Fields
///
/// * `metadata` — Optional metadata with result code and reason
/// * `data` — The parsed response payload
/// * `errors` — Error messages from the module
/// * `messages` — General messages (info/warnings)
/// * `status` — `1` for success, `0` for failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UapiResult<T> {
    /// Optional metadata about the result.
    pub metadata: Option<Metadata>,
    /// The response data.
    pub data: Option<T>,
    /// Module-level error messages.
    pub errors: Option<Vec<String>>,
    /// General messages from the API.
    pub messages: Option<Vec<String>>,
    /// Response status (`1` = success).
    pub status: i32,
}

/// Metadata attached to a UAPI response.
///
/// Contains the numeric result code, a reason string, and an optional status code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// The result code (`1` for success).
    pub result: Option<i32>,
    /// A human-readable reason for the result.
    pub reason: Option<String>,
    /// An optional status code.
    pub status: Option<i32>,
}

/// The WHM API response envelope.
///
/// WHM API responses have a different structure than UAPI responses: they include
/// a `version` field and a `metadata` object at the top level.
///
/// # Fields
///
/// * `version` — API version number
/// * `metadata` — Result metadata
/// * `data` — The parsed response payload
/// * `errors` — Error messages
/// * `messages` — Info/warning messages
/// * `status` — Response status (`1` = success)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhmApiResponse<T> {
    /// The API version (typically `1.0`).
    pub version: f64,
    /// Result metadata.
    pub metadata: WhmMetadata,
    /// The response data.
    pub data: Option<T>,
    /// Error messages.
    pub errors: Option<Vec<String>>,
    /// General messages.
    pub messages: Option<Vec<String>>,
    /// Response status (`1` = success).
    pub status: i32,
}

/// Metadata from a WHM API response.
///
/// Unlike UAPI metadata, WHM metadata always has a `result` and `status` field
/// (they are not optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhmMetadata {
    /// The result code (`1` for success).
    pub result: i32,
    /// A human-readable reason for the result.
    pub reason: Option<String>,
    /// The status code (`1` = success).
    pub status: i32,
}

/// Builder for query parameters passed to cPanel/WHM API list operations.
///
/// Many endpoints support pagination and filtering. Use this builder to construct
/// the parameter set, then pass it to methods like [`CpanelClient::list_accounts_paginated`].
///
/// # Example
///
/// ```
/// use cpanel_rs::types::QueryParams;
///
/// let params = QueryParams::new()
///     .user("john")
///     .page(1)
///     .limit(10);
///
/// let pairs = params.to_pairs();
/// assert_eq!(pairs.len(), 3);
/// ```
///
/// [`CpanelClient::list_accounts_paginated`]: crate::CpanelClient::list_accounts_paginated
#[derive(Debug, Clone, Default, Serialize)]
pub struct QueryParams {
    /// The cPanel/WHM account user.
    pub user: Option<String>,
    /// The domain name.
    pub domain: Option<String>,
    /// The DNS zone name.
    pub zone: Option<String>,
    /// Page number for paginated results (1-indexed).
    pub page: Option<i32>,
    /// Filtering by package/plan name.
    pub plan: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<i32>,
}

impl QueryParams {
    /// Creates an empty parameter set.
    ///
    /// This is equivalent to `Self::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the account user.
    pub fn user(mut self, user: &str) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets the domain name.
    pub fn domain(mut self, domain: &str) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Sets the DNS zone name.
    pub fn zone(mut self, zone: &str) -> Self {
        self.zone = Some(zone.into());
        self
    }

    /// Sets the page number for pagination.
    pub fn page(mut self, page: i32) -> Self {
        self.page = Some(page);
        self
    }

    /// Filters by package/plan name.
    pub fn plan(mut self, plan: &str) -> Self {
        self.plan = Some(plan.into());
        self
    }

    /// Sets the maximum number of results.
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Converts the parameters to a vector of `(key, value)` pairs.
    ///
    /// Only non-`None` fields are included in the output.
    pub fn to_pairs(&self) -> Vec<(&str, String)> {
        let mut pairs = Vec::new();
        if let Some(ref v) = self.user {
            pairs.push(("user", v.clone()));
        }
        if let Some(ref v) = self.domain {
            pairs.push(("domain", v.clone()));
        }
        if let Some(ref v) = self.zone {
            pairs.push(("zone", v.clone()));
        }
        if let Some(v) = self.page {
            pairs.push(("page", v.to_string()));
        }
        if let Some(ref v) = self.plan {
            pairs.push(("plan", v.clone()));
        }
        if let Some(v) = self.limit {
            pairs.push(("limit", v.to_string()));
        }
        pairs
    }
}
