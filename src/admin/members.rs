//! # Organization Members Admin API
//!
//! This module provides a Rust interface to Anthropic's Admin API for managing organization members, which allows you to
//! list, get, update, and remove users from your organization.
//!
//! ## Key Features
//!
//! - List all users with pagination and filtering support
//! - Get detailed information about a specific user
//! - Update user roles within the organization
//! - Remove users from the organization
//!
//! ## Basic Usage
//!
//! ```no_run
//! use anthropic_api::{admin::members::*, Credentials};
//!
//! #[tokio::main]
//! async fn main() {
//!     let credentials = Credentials::from_env();
//!
//!     // List users
//!     let users = UserList::builder()
//!         .credentials(credentials.clone())
//!         .create()
//!         .await
//!         .unwrap();
//!
//!     println!("Organization members: {:?}", users.data);
//!
//!     // Get a specific user
//!     if let Some(user) = users.data.first() {
//!         let user_details = User::builder(&user.id)
//!             .credentials(credentials.clone())
//!             .create()
//!             .await
//!             .unwrap();
//!
//!         println!("User details: {:?}", user_details);
//!     }
//! }
//! ```

use crate::{anthropic_request_json, ApiResponseOrError, Credentials};
use derive_builder::Builder;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// Organization role of a user
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Regular user
    User,
    /// Developer role
    Developer,
    /// Billing administrator
    Billing,
    /// Organization administrator
    Admin,
}

/// A user in the organization
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct User {
    /// Unique user identifier
    pub id: String,
    /// User's email address
    pub email: String,
    /// User's name
    pub name: String,
    /// RFC 3339 datetime string indicating when the user joined the organization
    pub added_at: String,
    /// User's role in the organization
    pub role: UserRole,
    /// Object type (always "user" for Users)
    #[serde(rename = "type")]
    pub user_type: String,
}

/// Response from the List Users API
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct UserList {
    /// List of users in the organization
    pub data: Vec<User>,
    /// First ID in the data list (for pagination)
    pub first_id: Option<String>,
    /// Last ID in the data list (for pagination)
    pub last_id: Option<String>,
    /// Indicates if there are more results in the requested page direction
    pub has_more: bool,
}

/// Response from the Remove User API
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct UserDeleted {
    /// ID of the deleted user
    pub id: String,
    /// Object type (always "user_deleted" for deleted users)
    #[serde(rename = "type")]
    pub deleted_type: String,
}

/// Request parameters for listing users
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "UserListBuilder")]
#[builder(setter(strip_option, into))]
pub struct UserListRequest {
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

    /// Filter by user email
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for getting a specific user
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "UserBuilder")]
#[builder(setter(strip_option, into))]
pub struct UserRequest {
    /// User identifier
    pub user_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for updating a user
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "UserUpdateBuilder")]
#[builder(setter(strip_option, into))]
pub struct UserUpdateRequest {
    /// User identifier (not serialized)
    #[serde(skip_serializing)]
    pub user_id: String,

    /// New role for the user (cannot be "admin")
    pub role: UserRole,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for removing a user
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "UserRemoveBuilder")]
#[builder(setter(strip_option, into))]
pub struct UserRemoveRequest {
    /// User identifier
    pub user_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

impl UserList {
    /// Creates a builder for listing users.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let users = UserList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> UserListBuilder {
        UserListBuilder::create_empty()
    }

    /// Lists users in the organization with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = UserListRequest {
    ///     before_id: None,
    ///     after_id: None,
    ///     limit: Some(20),
    ///     email: None,
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let users = UserList::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: UserListRequest) -> ApiResponseOrError<Self> {
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
        if let Some(email) = &request.email {
            query_params.push(("email", email.clone()));
        }

        anthropic_request_json(
            Method::GET,
            "organizations/users",
            |r| r.query(&query_params),
            credentials_opt,
        )
        .await
    }
}

impl User {
    /// Creates a builder for getting a specific user.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let user = User::builder("user_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(user_id: impl Into<String>) -> UserBuilder {
        UserBuilder::create_empty().user_id(user_id)
    }

    /// Gets information about a specific user.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = UserRequest {
    ///     user_id: "user_123456789".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let user = User::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: UserRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/users/{}", request.user_id);

        anthropic_request_json(Method::GET, &route, |r| r, credentials_opt).await
    }

    /// Creates a builder for updating a user.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let updated_user = User::update_builder("user_123456789")
    ///     .credentials(credentials)
    ///     .role(UserRole::Developer)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_builder(user_id: impl Into<String>) -> UserUpdateBuilder {
        UserUpdateBuilder::create_empty().user_id(user_id)
    }

    /// Updates a user with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = UserUpdateRequest {
    ///     user_id: "user_123456789".to_string(),
    ///     role: UserRole::Developer,
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let updated_user = User::update(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(request: UserUpdateRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/users/{}", request.user_id);

        anthropic_request_json(Method::POST, &route, |r| r.json(&request), credentials_opt).await
    }

    /// Creates a builder for removing a user.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let removed_user = User::remove_builder("user_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_builder(user_id: impl Into<String>) -> UserRemoveBuilder {
        UserRemoveBuilder::create_empty().user_id(user_id)
    }

    /// Removes a user from the organization.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = UserRemoveRequest {
    ///     user_id: "user_123456789".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let removed_user = User::remove(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove(request: UserRemoveRequest) -> ApiResponseOrError<UserDeleted> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/users/{}", request.user_id);

        anthropic_request_json(Method::DELETE, &route, |r| r, credentials_opt).await
    }
}

// Builder convenience methods
impl UserListBuilder {
    /// Creates a new user list request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Users API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let users = UserList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<UserList> {
        let request = self.build().unwrap();
        UserList::create(request).await
    }
}

impl UserBuilder {
    /// Creates a new user request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Users API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let user = User::builder("user_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<User> {
        let request = self.build().unwrap();
        User::create(request).await
    }
}

impl UserUpdateBuilder {
    /// Creates a new user update request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Users API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let updated_user = User::update_builder("user_123456789")
    ///     .credentials(credentials)
    ///     .role(UserRole::Developer)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<User> {
        let request = self.build().unwrap();
        User::update(request).await
    }
}

impl UserRemoveBuilder {
    /// Creates a new user remove request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Users API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::members::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let removed_user = User::remove_builder("user_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<UserDeleted> {
        let request = self.build().unwrap();
        User::remove(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Credentials;

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_list_users() {
        let credentials = Credentials::from_env();

        let users = UserList::builder()
            .credentials(credentials)
            .create()
            .await
            .unwrap();

        assert!(users.data.len() > 0);
    }

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_get_user() {
        let credentials = Credentials::from_env();

        // First get a user ID from the list
        let users = UserList::builder()
            .credentials(credentials.clone())
            .create()
            .await
            .unwrap();

        if let Some(user) = users.data.first() {
            let user_id = &user.id;

            // Then get that specific user
            let user_details = User::builder(user_id)
                .credentials(credentials)
                .create()
                .await
                .unwrap();

            assert_eq!(user_details.id, *user_id);
        }
    }
}
