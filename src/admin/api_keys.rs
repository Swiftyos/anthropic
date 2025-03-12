//! # API Keys Admin API
//!
//! This module provides a Rust interface to Anthropic's Admin API for managing API keys, which allows you to
//! list, get, and update API keys.
//!
//! ## Key Features
//!
//! - List all API keys with pagination and filtering support
//! - Get detailed information about a specific API key
//! - Update API key properties like name and status
//!
//! ## Basic Usage
//!
//! ```no_run
//! use anthropic_api::{admin::api_keys::*, Credentials};
//!
//! #[tokio::main]
//! async fn main() {
//!     let credentials = Credentials::from_env();
//!
//!     // List API keys
//!     let api_keys = ApiKeyList::builder()
//!         .credentials(credentials.clone())
//!         .create()
//!         .await
//!         .unwrap();
//!
//!     println!("Available API keys: {:?}", api_keys.data);
//!
//!     // Get a specific API key
//!     if let Some(api_key) = api_keys.data.first() {
//!         let api_key_details = ApiKey::builder(&api_key.id)
//!             .credentials(credentials.clone())
//!             .create()
//!             .await
//!             .unwrap();
//!
//!         println!("API key details: {:?}", api_key_details);
//!     }
//! }
//! ```

use crate::{anthropic_request_json, ApiResponseOrError, Credentials};
use derive_builder::Builder;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// Status of an API key
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyStatus {
    /// API key is active and can be used
    Active,
    /// API key is inactive and cannot be used
    Inactive,
    /// API key is archived and cannot be used
    Archived,
}

/// Information about the creator of an API key
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ApiKeyCreator {
    /// ID of the creator
    pub id: String,
    /// Type of the creator
    #[serde(rename = "type")]
    pub creator_type: String,
}

/// An API key available through the Anthropic Admin API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ApiKey {
    /// Unique API key identifier
    pub id: String,
    /// Name of the API key
    pub name: String,
    /// RFC 3339 datetime string representing the time at which the API key was created
    pub created_at: String,
    /// Information about who created the API key
    pub created_by: ApiKeyCreator,
    /// Partially redacted hint for the API key
    pub partial_key_hint: Option<String>,
    /// Status of the API key
    pub status: ApiKeyStatus,
    /// Object type (always "api_key" for API Keys)
    #[serde(rename = "type")]
    pub key_type: String,
    /// ID of the Workspace associated with the API key, or null if the API key belongs to the default Workspace
    pub workspace_id: Option<String>,
}

/// Response from the List API Keys API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ApiKeyList {
    /// List of available API keys
    pub data: Vec<ApiKey>,
    /// First ID in the data list (for pagination)
    pub first_id: Option<String>,
    /// Last ID in the data list (for pagination)
    pub last_id: Option<String>,
    /// Indicates if there are more results in the requested page direction
    pub has_more: bool,
}

/// Request parameters for listing API keys.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "ApiKeyListBuilder")]
#[builder(setter(strip_option, into))]
pub struct ApiKeyListRequest {
    /// ID of the object to use as a cursor for pagination (previous page)
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,

    /// ID of the object to use as a cursor for pagination (next page)
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,

    /// Number of items to return per page (1-1000)
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Filter by API key status
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ApiKeyStatus>,

    /// Filter by Workspace ID
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,

    /// Filter by the ID of the User who created the object
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by_user_id: Option<String>,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for getting a specific API key.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "ApiKeyBuilder")]
#[builder(setter(strip_option, into))]
pub struct ApiKeyRequest {
    /// API key identifier
    pub api_key_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for updating an API key.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "ApiKeyUpdateBuilder")]
#[builder(setter(strip_option, into))]
pub struct ApiKeyUpdateRequest {
    /// API key identifier (not serialized)
    #[serde(skip_serializing)]
    pub api_key_id: String,

    /// New name for the API key
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// New status for the API key
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ApiKeyStatus>,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

impl ApiKeyList {
    /// Creates a builder for listing API keys.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::api_keys::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let api_keys = ApiKeyList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> ApiKeyListBuilder {
        ApiKeyListBuilder::create_empty()
    }

    /// Lists available API keys with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::api_keys::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = ApiKeyListRequest {
    ///     before_id: None,
    ///     after_id: None,
    ///     limit: Some(20),
    ///     status: None,
    ///     workspace_id: None,
    ///     created_by_user_id: None,
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let api_keys = ApiKeyList::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: ApiKeyListRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();

        // Build query parameters
        let mut query_params = Vec::new();
        if let Some(before_id) = &request.before_id {
            query_params.push(("before_id", before_id.clone()));
        }
        if let Some(after_id) = &request.after_id {
            query_params.push(("after_id", after_id.clone()));
        }
        if let Some(limit) = request.limit {
            query_params.push(("limit", limit.to_string()));
        }
        if let Some(status) = &request.status {
            query_params.push(("status", format!("{:?}", status).to_lowercase()));
        }
        if let Some(workspace_id) = &request.workspace_id {
            query_params.push(("workspace_id", workspace_id.clone()));
        }
        if let Some(created_by_user_id) = &request.created_by_user_id {
            query_params.push(("created_by_user_id", created_by_user_id.clone()));
        }

        anthropic_request_json(
            Method::GET,
            "organizations/api_keys",
            |r| r.query(&query_params),
            credentials_opt,
        )
        .await
    }
}

impl ApiKey {
    /// Creates a builder for getting a specific API key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::api_keys::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let api_key = ApiKey::builder("api_key_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(api_key_id: impl Into<String>) -> ApiKeyBuilder {
        ApiKeyBuilder::create_empty().api_key_id(api_key_id)
    }

    /// Gets information about a specific API key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::api_keys::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = ApiKeyRequest {
    ///     api_key_id: "api_key_123456789".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let api_key = ApiKey::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: ApiKeyRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/api_keys/{}", request.api_key_id);

        anthropic_request_json(Method::GET, &route, |r| r, credentials_opt).await
    }

    /// Creates a builder for updating an API key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::api_keys::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let updated_api_key = ApiKey::update_builder("api_key_123456789")
    ///     .credentials(credentials)
    ///     .name("New API Key Name")
    ///     .status(ApiKeyStatus::Inactive)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_builder(api_key_id: impl Into<String>) -> ApiKeyUpdateBuilder {
        ApiKeyUpdateBuilder::create_empty().api_key_id(api_key_id)
    }

    /// Updates an API key with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::api_keys::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = ApiKeyUpdateRequest {
    ///     api_key_id: "api_key_123456789".to_string(),
    ///     name: Some("New API Key Name".to_string()),
    ///     status: Some(ApiKeyStatus::Inactive),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let updated_api_key = ApiKey::update(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(request: ApiKeyUpdateRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/api_keys/{}", request.api_key_id);

        anthropic_request_json(Method::POST, &route, |r| r.json(&request), credentials_opt).await
    }
}

// Builder convenience methods
impl ApiKeyListBuilder {
    /// Creates a new API key list request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the API Keys API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::api_keys::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let api_keys = ApiKeyList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<ApiKeyList> {
        let request = self.build().unwrap();
        ApiKeyList::create(request).await
    }
}

impl ApiKeyBuilder {
    /// Creates a new API key request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the API Keys API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::api_keys::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let api_key = ApiKey::builder("api_key_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<ApiKey> {
        let request = self.build().unwrap();
        ApiKey::create(request).await
    }
}

impl ApiKeyUpdateBuilder {
    /// Creates a new API key update request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the API Keys API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::api_keys::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let updated_api_key = ApiKey::update_builder("api_key_123456789")
    ///     .credentials(credentials)
    ///     .name("New API Key Name")
    ///     .status(ApiKeyStatus::Inactive)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<ApiKey> {
        let request = self.build().unwrap();
        ApiKey::update(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Credentials;

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_list_api_keys() {
        let credentials = Credentials::from_env();

        let api_keys = ApiKeyList::builder()
            .credentials(credentials)
            .create()
            .await
            .unwrap();

        assert!(api_keys.data.len() > 0);
    }

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_get_api_key() {
        let credentials = Credentials::from_env();

        // First get an API key ID from the list
        let api_keys = ApiKeyList::builder()
            .credentials(credentials.clone())
            .create()
            .await
            .unwrap();

        if let Some(api_key) = api_keys.data.first() {
            let api_key_id = &api_key.id;

            // Then get that specific API key
            let api_key_details = ApiKey::builder(api_key_id)
                .credentials(credentials)
                .create()
                .await
                .unwrap();

            assert_eq!(api_key_details.id, *api_key_id);
        }
    }
}
