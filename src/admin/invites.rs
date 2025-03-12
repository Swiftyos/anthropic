//! # Organization Invites Admin API
//!
//! This module provides a Rust interface to Anthropic's Admin API for managing organization invites, which allows you to
//! list, get, create, and delete invites to your organization.
//!
//! ## Key Features
//!
//! - List all invites with pagination support
//! - Get detailed information about a specific invite
//! - Create new invites to the organization
//! - Delete pending invites
//!
//! ## Basic Usage
//!
//! ```no_run
//! use anthropic_api::{admin::invites::*, Credentials};
//!
//! #[tokio::main]
//! async fn main() {
//!     let credentials = Credentials::from_env();
//!
//!     // List invites
//!     let invites = InviteList::builder()
//!         .credentials(credentials.clone())
//!         .create()
//!         .await
//!         .unwrap();
//!
//!     println!("Organization invites: {:?}", invites.data);
//!
//!     // Get a specific invite
//!     if let Some(invite) = invites.data.first() {
//!         let invite_details = Invite::builder(&invite.id)
//!             .credentials(credentials.clone())
//!             .create()
//!             .await
//!             .unwrap();
//!
//!         println!("Invite details: {:?}", invite_details);
//!     }
//! }
//! ```

use crate::{anthropic_request_json, ApiResponseOrError, Credentials};
use derive_builder::Builder;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// Organization role of an invited user
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InviteRole {
    /// Regular user
    User,
    /// Developer role
    Developer,
    /// Billing administrator
    Billing,
    /// Organization administrator
    Admin,
}

/// Status of an invite
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InviteStatus {
    /// Invite has been accepted
    Accepted,
    /// Invite has expired
    Expired,
    /// Invite has been deleted
    Deleted,
    /// Invite is pending acceptance
    Pending,
}

/// An invite to the organization
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Invite {
    /// Unique invite identifier
    pub id: String,
    /// Email of the user being invited
    pub email: String,
    /// RFC 3339 datetime string indicating when the invite was created
    pub invited_at: String,
    /// RFC 3339 datetime string indicating when the invite expires
    pub expires_at: String,
    /// Role assigned to the invited user
    pub role: InviteRole,
    /// Current status of the invite
    pub status: InviteStatus,
    /// Object type (always "invite" for Invites)
    #[serde(rename = "type")]
    pub invite_type: String,
}

/// Response from the List Invites API
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct InviteList {
    /// List of invites in the organization
    pub data: Vec<Invite>,
    /// First ID in the data list (for pagination)
    pub first_id: Option<String>,
    /// Last ID in the data list (for pagination)
    pub last_id: Option<String>,
    /// Indicates if there are more results in the requested page direction
    pub has_more: bool,
}

/// Response from the Delete Invite API
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct InviteDeleted {
    /// ID of the deleted invite
    pub id: String,
    /// Object type (always "invite_deleted" for deleted invites)
    #[serde(rename = "type")]
    pub deleted_type: String,
}

/// Request parameters for listing invites
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "InviteListBuilder")]
#[builder(setter(strip_option, into))]
pub struct InviteListRequest {
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

/// Request parameters for getting a specific invite
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "InviteBuilder")]
#[builder(setter(strip_option, into))]
pub struct InviteRequest {
    /// Invite identifier
    pub invite_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for creating an invite
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "InviteCreateBuilder")]
#[builder(setter(strip_option, into))]
pub struct InviteCreateRequest {
    /// Email of the user to invite
    pub email: String,

    /// Role for the invited user (cannot be "admin")
    pub role: InviteRole,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for deleting an invite
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "InviteDeleteBuilder")]
#[builder(setter(strip_option, into))]
pub struct InviteDeleteRequest {
    /// Invite identifier
    pub invite_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

impl InviteList {
    /// Creates a builder for listing invites.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let invites = InviteList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> InviteListBuilder {
        InviteListBuilder::create_empty()
    }

    /// Lists invites in the organization with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = InviteListRequest {
    ///     before_id: None,
    ///     after_id: None,
    ///     limit: Some(20),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let invites = InviteList::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: InviteListRequest) -> ApiResponseOrError<Self> {
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
            "organizations/invites",
            |r| r.query(&query_params),
            credentials_opt,
        )
        .await
    }
}

impl Invite {
    /// Creates a builder for getting a specific invite.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let invite = Invite::builder("invite_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(invite_id: impl Into<String>) -> InviteBuilder {
        InviteBuilder::create_empty().invite_id(invite_id)
    }

    /// Gets information about a specific invite.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = InviteRequest {
    ///     invite_id: "invite_123456789".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let invite = Invite::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: InviteRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/invites/{}", request.invite_id);

        anthropic_request_json(Method::GET, &route, |r| r, credentials_opt).await
    }

    /// Creates a builder for creating a new invite.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let new_invite = Invite::create_builder()
    ///     .credentials(credentials)
    ///     .email("user@example.com")
    ///     .role(InviteRole::Developer)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_builder() -> InviteCreateBuilder {
        InviteCreateBuilder::create_empty()
    }

    /// Creates a new invite with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = InviteCreateRequest {
    ///     email: "user@example.com".to_string(),
    ///     role: InviteRole::Developer,
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let new_invite = Invite::create_new(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_new(request: InviteCreateRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();

        anthropic_request_json(
            Method::POST,
            "organizations/invites",
            |r| r.json(&request),
            credentials_opt,
        )
        .await
    }

    /// Creates a builder for deleting an invite.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let deleted_invite = Invite::delete_builder("invite_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn delete_builder(invite_id: impl Into<String>) -> InviteDeleteBuilder {
        InviteDeleteBuilder::create_empty().invite_id(invite_id)
    }

    /// Deletes an invite from the organization.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = InviteDeleteRequest {
    ///     invite_id: "invite_123456789".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let deleted_invite = Invite::delete(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(request: InviteDeleteRequest) -> ApiResponseOrError<InviteDeleted> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/invites/{}", request.invite_id);

        anthropic_request_json(Method::DELETE, &route, |r| r, credentials_opt).await
    }
}

// Builder convenience methods
impl InviteListBuilder {
    /// Creates a new invite list request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Invites API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let invites = InviteList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<InviteList> {
        let request = self.build().unwrap();
        InviteList::create(request).await
    }
}

impl InviteBuilder {
    /// Creates a new invite request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Invites API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let invite = Invite::builder("invite_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<Invite> {
        let request = self.build().unwrap();
        Invite::create(request).await
    }
}

impl InviteCreateBuilder {
    /// Creates a new invite create request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Invites API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let new_invite = Invite::create_builder()
    ///     .credentials(credentials)
    ///     .email("user@example.com")
    ///     .role(InviteRole::Developer)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<Invite> {
        let request = self.build().unwrap();
        Invite::create_new(request).await
    }
}

impl InviteDeleteBuilder {
    /// Creates a new invite delete request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Invites API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::invites::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let deleted_invite = Invite::delete_builder("invite_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<InviteDeleted> {
        let request = self.build().unwrap();
        Invite::delete(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Credentials;

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_list_invites() {
        let credentials = Credentials::from_env();

        let invites = InviteList::builder()
            .credentials(credentials)
            .create()
            .await
            .unwrap();

        assert!(invites.data.len() > 0);
    }

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_get_invite() {
        let credentials = Credentials::from_env();

        // First get an invite ID from the list
        let invites = InviteList::builder()
            .credentials(credentials.clone())
            .create()
            .await
            .unwrap();

        if let Some(invite) = invites.data.first() {
            let invite_id = &invite.id;

            // Then get that specific invite
            let invite_details = Invite::builder(invite_id)
                .credentials(credentials)
                .create()
                .await
                .unwrap();

            assert_eq!(invite_details.id, *invite_id);
        }
    }
}
