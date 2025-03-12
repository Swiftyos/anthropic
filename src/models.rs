//! # Models API
//!
//! This module provides a Rust interface to Anthropic's Models API, which allows you to
//! [list available models](https://docs.anthropic.com/en/api/models-list) and [get information about specific models](https://docs.anthropic.com/en/api/models-get).
//!
//! ## Key Features
//!
//! - List all available models with pagination support
//! - Get detailed information about a specific model
//! - Resolve model aliases to model IDs
//!
//! ## Basic Usage
//!
//! ```no_run
//! use anthropic_api::{models::*, Credentials};
//!
//! #[tokio::main]
//! async fn main() {
//!     let credentials = Credentials::from_env();
//!
//!     // List available models
//!     let models = ModelList::builder()
//!         .credentials(credentials.clone())
//!         .create()
//!         .await
//!         .unwrap();
//!
//!     println!("Available models: {:?}", models.data);
//!
//!     // Get a specific model
//!     let model = Model::builder("claude-3-7-sonnet-20250219")
//!         .credentials(credentials)
//!         .create()
//!         .await
//!         .unwrap();
//!
//!     println!("Model details: {:?}", model);
//! }
//! ```

use crate::{anthropic_request_json, ApiResponseOrError, Credentials};
use derive_builder::Builder;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// A model available through the Anthropic API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Model {
    /// Unique model identifier
    pub id: String,
    /// A human-readable name for the model
    pub display_name: String,
    /// RFC 3339 datetime string representing the time at which the model was released
    pub created_at: String,
    /// Object type (always "model" for Models)
    #[serde(rename = "type")]
    pub model_type: String,
}

/// Response from the List Models API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ModelList {
    /// List of available models
    pub data: Vec<Model>,
    /// First ID in the data list (for pagination)
    pub first_id: Option<String>,
    /// Last ID in the data list (for pagination)
    pub last_id: Option<String>,
    /// Indicates if there are more results in the requested page direction
    pub has_more: bool,
}

/// Request parameters for listing models.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "ModelListBuilder")]
#[builder(setter(strip_option, into))]
pub struct ModelListRequest {
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

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for getting a specific model.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "ModelBuilder")]
#[builder(setter(strip_option, into))]
pub struct ModelRequest {
    /// Model identifier or alias
    pub model_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

impl ModelList {
    /// Creates a builder for listing models.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{models::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let models = ModelList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> ModelListBuilder {
        ModelListBuilder::create_empty()
    }

    /// Lists available models with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{models::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = ModelListRequest {
    ///     before_id: None,
    ///     after_id: None,
    ///     limit: Some(20),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let models = ModelList::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: ModelListRequest) -> ApiResponseOrError<Self> {
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

        anthropic_request_json(
            Method::GET,
            "models",
            |r| r.query(&query_params),
            credentials_opt,
        )
        .await
    }
}

impl Model {
    /// Creates a builder for getting a specific model.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{models::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let model = Model::builder("claude-3-7-sonnet-20250219")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(model_id: impl Into<String>) -> ModelBuilder {
        ModelBuilder::create_empty().model_id(model_id)
    }

    /// Gets information about a specific model.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{models::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = ModelRequest {
    ///     model_id: "claude-3-7-sonnet-20250219".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let model = Model::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: ModelRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("models/{}", request.model_id);

        anthropic_request_json(Method::GET, &route, |r| r, credentials_opt).await
    }
}

// Builder convenience methods
impl ModelListBuilder {
    /// Creates a new model list request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Models API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{models::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let models = ModelList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<ModelList> {
        let request = self.build().unwrap();
        ModelList::create(request).await
    }
}

impl ModelBuilder {
    /// Creates a new model request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Models API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{models::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let model = Model::builder("claude-3-7-sonnet-20250219")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<Model> {
        let request = self.build().unwrap();
        Model::create(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Credentials;

    #[tokio::test]
    async fn test_list_models() {
        let credentials = Credentials::from_env();

        let models = ModelList::builder()
            .credentials(credentials)
            .create()
            .await
            .unwrap();

        assert!(!models.data.is_empty());
    }

    #[tokio::test]
    async fn test_get_model() {
        let credentials = Credentials::from_env();

        // First get a model ID from the list
        let models = ModelList::builder()
            .credentials(credentials.clone())
            .create()
            .await
            .unwrap();

        let model_id = &models.data[0].id;

        // Then get that specific model
        let model = Model::builder(model_id)
            .credentials(credentials)
            .create()
            .await
            .unwrap();

        assert_eq!(model.id, *model_id);
    }
}
