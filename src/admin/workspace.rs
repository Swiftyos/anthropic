//! # Workspaces Admin API
//!
//! This module provides a Rust interface to Anthropic's Admin API for managing workspaces and workspace members, which allows you to
//! list, get, create, update, and archive workspaces, as well as manage workspace members.
//!
//! ## Key Features
//!
//! - List all workspaces with pagination and filtering support
//! - Get detailed information about a specific workspace
//! - Create new workspaces
//! - Update workspace properties like name
//! - Archive workspaces
//! - List all members of a workspace with pagination support
//! - Get detailed information about a specific workspace member
//! - Add new members to a workspace
//! - Update workspace member roles
//! - Remove members from a workspace
//!
//! ## Basic Usage
//!
//! ```no_run
//! use anthropic_api::{admin::workspace::*, Credentials};
//!
//! #[tokio::main]
//! async fn main() {
//!     let credentials = Credentials::from_env();
//!
//!     // List workspaces
//!     let workspaces = WorkspaceList::builder()
//!         .credentials(credentials.clone())
//!         .create()
//!         .await
//!         .unwrap();
//!
//!     println!("Available workspaces: {:?}", workspaces.data);
//!
//!     // Get a specific workspace
//!     if let Some(workspace) = workspaces.data.first() {
//!         let workspace_details = Workspace::builder(&workspace.id)
//!             .credentials(credentials.clone())
//!             .create()
//!             .await
//!             .unwrap();
//!
//!         println!("Workspace details: {:?}", workspace_details);
//!         
//!         // List members of the workspace
//!         let members = WorkspaceMemberList::builder(&workspace.id)
//!             .credentials(credentials.clone())
//!             .create()
//!             .await
//!             .unwrap();
//!             
//!         println!("Workspace members: {:?}", members.data);
//!     }
//! }
//! ```

use crate::{anthropic_request_json, ApiResponseOrError, Credentials};
use derive_builder::Builder;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// A workspace available through the Anthropic Admin API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Workspace {
    /// Unique workspace identifier
    pub id: String,
    /// Name of the workspace
    pub name: String,
    /// RFC 3339 datetime string representing the time at which the workspace was created
    pub created_at: String,
    /// RFC 3339 datetime string indicating when the workspace was archived, or null if the workspace is not archived
    pub archived_at: Option<String>,
    /// Hex color code representing the workspace in the Anthropic Console
    pub display_color: String,
    /// Object type (always "workspace" for Workspaces)
    #[serde(rename = "type")]
    pub workspace_type: String,
}

/// Response from the List Workspaces API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceList {
    /// List of available workspaces
    pub data: Vec<Workspace>,
    /// First ID in the data list (for pagination)
    pub first_id: Option<String>,
    /// Last ID in the data list (for pagination)
    pub last_id: Option<String>,
    /// Indicates if there are more results in the requested page direction
    pub has_more: bool,
}

/// Request parameters for listing workspaces.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceListBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceListRequest {
    /// Whether to include workspaces that have been archived in the response
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_archived: Option<bool>,

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

/// Request parameters for getting a specific workspace.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceRequest {
    /// Workspace identifier
    pub workspace_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for creating a workspace.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceCreateBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceCreateRequest {
    /// Name of the workspace
    pub name: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for updating a workspace.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceUpdateBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceUpdateRequest {
    /// Workspace identifier (not serialized)
    #[serde(skip_serializing)]
    pub workspace_id: String,

    /// New name for the workspace
    pub name: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for archiving a workspace.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceArchiveBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceArchiveRequest {
    /// Workspace identifier
    pub workspace_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

impl WorkspaceList {
    /// Creates a builder for listing workspaces.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let workspaces = WorkspaceList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> WorkspaceListBuilder {
        WorkspaceListBuilder::create_empty()
    }

    /// Lists available workspaces with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceListRequest {
    ///     include_archived: Some(true),
    ///     before_id: None,
    ///     after_id: None,
    ///     limit: Some(20),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let workspaces = WorkspaceList::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: WorkspaceListRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();

        // Build query parameters
        let mut query_params = Vec::new();
        if let Some(include_archived) = request.include_archived {
            query_params.push(("include_archived", include_archived.to_string()));
        }
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
            "organizations/workspaces",
            |r| r.query(&query_params),
            credentials_opt,
        )
        .await
    }
}

impl Workspace {
    /// Creates a builder for getting a specific workspace.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let workspace = Workspace::builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(workspace_id: impl Into<String>) -> WorkspaceBuilder {
        WorkspaceBuilder::create_empty().workspace_id(workspace_id)
    }

    /// Gets information about a specific workspace.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceRequest {
    ///     workspace_id: "workspace_123456789".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let workspace = Workspace::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: WorkspaceRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/workspaces/{}", request.workspace_id);

        anthropic_request_json(Method::GET, &route, |r| r, credentials_opt).await
    }

    /// Creates a builder for creating a new workspace.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let new_workspace = Workspace::create_builder()
    ///     .credentials(credentials)
    ///     .name("My New Workspace")
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_builder() -> WorkspaceCreateBuilder {
        WorkspaceCreateBuilder::create_empty()
    }

    /// Creates a new workspace with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceCreateRequest {
    ///     name: "My New Workspace".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let new_workspace = Workspace::create_new(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_new(request: WorkspaceCreateRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();

        anthropic_request_json(
            Method::POST,
            "organizations/workspaces",
            |r| r.json(&request),
            credentials_opt,
        )
        .await
    }

    /// Creates a builder for updating a workspace.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let updated_workspace = Workspace::update_builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .name("Updated Workspace Name")
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_builder(workspace_id: impl Into<String>) -> WorkspaceUpdateBuilder {
        WorkspaceUpdateBuilder::create_empty().workspace_id(workspace_id)
    }

    /// Updates a workspace with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceUpdateRequest {
    ///     workspace_id: "workspace_123456789".to_string(),
    ///     name: "Updated Workspace Name".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let updated_workspace = Workspace::update(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(request: WorkspaceUpdateRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/workspaces/{}", request.workspace_id);

        anthropic_request_json(Method::POST, &route, |r| r.json(&request), credentials_opt).await
    }

    /// Creates a builder for archiving a workspace.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let archived_workspace = Workspace::archive_builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn archive_builder(workspace_id: impl Into<String>) -> WorkspaceArchiveBuilder {
        WorkspaceArchiveBuilder::create_empty().workspace_id(workspace_id)
    }

    /// Archives a workspace with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceArchiveRequest {
    ///     workspace_id: "workspace_123456789".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let archived_workspace = Workspace::archive(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn archive(request: WorkspaceArchiveRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/workspaces/{}/archive", request.workspace_id);

        anthropic_request_json(Method::POST, &route, |r| r, credentials_opt).await
    }
}

// Builder convenience methods
impl WorkspaceListBuilder {
    /// Creates a new workspace list request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspaces API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let workspaces = WorkspaceList::builder()
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<WorkspaceList> {
        let request = self.build().unwrap();
        WorkspaceList::create(request).await
    }
}

impl WorkspaceBuilder {
    /// Creates a new workspace request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspaces API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let workspace = Workspace::builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<Workspace> {
        let request = self.build().unwrap();
        Workspace::create(request).await
    }
}

impl WorkspaceCreateBuilder {
    /// Creates a new workspace create request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspaces API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let new_workspace = Workspace::create_builder()
    ///     .credentials(credentials)
    ///     .name("My New Workspace")
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<Workspace> {
        let request = self.build().unwrap();
        Workspace::create_new(request).await
    }
}

impl WorkspaceUpdateBuilder {
    /// Creates a new workspace update request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspaces API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let updated_workspace = Workspace::update_builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .name("Updated Workspace Name")
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<Workspace> {
        let request = self.build().unwrap();
        Workspace::update(request).await
    }
}

impl WorkspaceArchiveBuilder {
    /// Creates a new workspace archive request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspaces API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let archived_workspace = Workspace::archive_builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<Workspace> {
        let request = self.build().unwrap();
        Workspace::archive(request).await
    }
}

/// A workspace member available through the Anthropic Admin API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceMember {
    /// Object type (always "workspace_member" for Workspace Members)
    #[serde(rename = "type")]
    pub member_type: String,
    /// User identifier
    pub user_id: String,
    /// Workspace identifier
    pub workspace_id: String,
    /// Role of the workspace member
    pub workspace_role: WorkspaceMemberRole,
}

/// Response from the List Workspace Members API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceMemberList {
    /// List of workspace members
    pub data: Vec<WorkspaceMember>,
    /// First ID in the data list (for pagination)
    pub first_id: Option<String>,
    /// Last ID in the data list (for pagination)
    pub last_id: Option<String>,
    /// Indicates if there are more results in the requested page direction
    pub has_more: bool,
}

/// Response from the Delete Workspace Member API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct WorkspaceMemberDeleted {
    /// Object type (always "workspace_member_deleted" for deleted Workspace Members)
    #[serde(rename = "type")]
    pub member_type: String,
    /// User identifier
    pub user_id: String,
    /// Workspace identifier
    pub workspace_id: String,
}

/// Role of a workspace member.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceMemberRole {
    /// Regular workspace user
    WorkspaceUser,
    /// Workspace developer with additional permissions
    WorkspaceDeveloper,
    /// Workspace administrator with full control
    WorkspaceAdmin,
    /// Workspace billing administrator
    WorkspaceBilling,
}

/// Request parameters for listing workspace members.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceMemberListBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceMemberListRequest {
    /// Workspace identifier
    pub workspace_id: String,

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

/// Request parameters for getting a specific workspace member.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceMemberBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceMemberRequest {
    /// Workspace identifier
    pub workspace_id: String,

    /// User identifier
    pub user_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for adding a workspace member.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceMemberAddBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceMemberAddRequest {
    /// Workspace identifier (not serialized)
    #[serde(skip_serializing)]
    pub workspace_id: String,

    /// User identifier
    pub user_id: String,

    /// Role of the new workspace member
    pub workspace_role: WorkspaceMemberRole,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for updating a workspace member.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceMemberUpdateBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceMemberUpdateRequest {
    /// Workspace identifier (not serialized)
    #[serde(skip_serializing)]
    pub workspace_id: String,

    /// User identifier (not serialized)
    #[serde(skip_serializing)]
    pub user_id: String,

    /// New role for the workspace member
    pub workspace_role: WorkspaceMemberRole,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Request parameters for deleting a workspace member.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "WorkspaceMemberDeleteBuilder")]
#[builder(setter(strip_option, into))]
pub struct WorkspaceMemberDeleteRequest {
    /// Workspace identifier
    pub workspace_id: String,

    /// User identifier
    pub user_id: String,

    /// Credentials for authentication (not serialized)
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

impl WorkspaceMemberList {
    /// Creates a builder for listing workspace members.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let members = WorkspaceMemberList::builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(workspace_id: impl Into<String>) -> WorkspaceMemberListBuilder {
        WorkspaceMemberListBuilder::create_empty().workspace_id(workspace_id)
    }

    /// Lists workspace members with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceMemberListRequest {
    ///     workspace_id: "workspace_123456789".to_string(),
    ///     before_id: None,
    ///     after_id: None,
    ///     limit: Some(20),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let members = WorkspaceMemberList::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: WorkspaceMemberListRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/workspaces/{}/members", request.workspace_id);

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
            &route,
            |r| r.query(&query_params),
            credentials_opt,
        )
        .await
    }
}

impl WorkspaceMember {
    /// Creates a builder for getting a specific workspace member.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let member = WorkspaceMember::builder("workspace_123456789", "user_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(
        workspace_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> WorkspaceMemberBuilder {
        WorkspaceMemberBuilder::create_empty()
            .workspace_id(workspace_id)
            .user_id(user_id)
    }

    /// Gets information about a specific workspace member.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceMemberRequest {
    ///     workspace_id: "workspace_123456789".to_string(),
    ///     user_id: "user_123456789".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let member = WorkspaceMember::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: WorkspaceMemberRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!(
            "organizations/workspaces/{}/members/{}",
            request.workspace_id, request.user_id
        );

        anthropic_request_json(Method::GET, &route, |r| r, credentials_opt).await
    }

    /// Creates a builder for adding a new workspace member.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let new_member = WorkspaceMember::add_builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .user_id("user_123456789")
    ///     .workspace_role(WorkspaceMemberRole::WorkspaceDeveloper)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_builder(workspace_id: impl Into<String>) -> WorkspaceMemberAddBuilder {
        WorkspaceMemberAddBuilder::create_empty().workspace_id(workspace_id)
    }

    /// Adds a new workspace member with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceMemberAddRequest {
    ///     workspace_id: "workspace_123456789".to_string(),
    ///     user_id: "user_123456789".to_string(),
    ///     workspace_role: WorkspaceMemberRole::WorkspaceDeveloper,
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let new_member = WorkspaceMember::add(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add(request: WorkspaceMemberAddRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!("organizations/workspaces/{}/members", request.workspace_id);

        // Create the request body
        let body = serde_json::json!({
            "user_id": request.user_id,
            "workspace_role": request.workspace_role,
        });

        anthropic_request_json(Method::POST, &route, |r| r.json(&body), credentials_opt).await
    }

    /// Creates a builder for updating a workspace member.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let updated_member = WorkspaceMember::update_builder("workspace_123456789", "user_123456789")
    ///     .credentials(credentials)
    ///     .workspace_role(WorkspaceMemberRole::WorkspaceAdmin)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_builder(
        workspace_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> WorkspaceMemberUpdateBuilder {
        WorkspaceMemberUpdateBuilder::create_empty()
            .workspace_id(workspace_id)
            .user_id(user_id)
    }

    /// Updates a workspace member with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceMemberUpdateRequest {
    ///     workspace_id: "workspace_123456789".to_string(),
    ///     user_id: "user_123456789".to_string(),
    ///     workspace_role: WorkspaceMemberRole::WorkspaceAdmin,
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let updated_member = WorkspaceMember::update(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(request: WorkspaceMemberUpdateRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        let route = format!(
            "organizations/workspaces/{}/members/{}",
            request.workspace_id, request.user_id
        );

        // Create the request body
        let body = serde_json::json!({
            "workspace_role": request.workspace_role,
        });

        anthropic_request_json(Method::POST, &route, |r| r.json(&body), credentials_opt).await
    }

    /// Creates a builder for deleting a workspace member.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let deleted_member = WorkspaceMember::delete_builder("workspace_123456789", "user_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn delete_builder(
        workspace_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> WorkspaceMemberDeleteBuilder {
        WorkspaceMemberDeleteBuilder::create_empty()
            .workspace_id(workspace_id)
            .user_id(user_id)
    }

    /// Deletes a workspace member with the given request parameters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = WorkspaceMemberDeleteRequest {
    ///     workspace_id: "workspace_123456789".to_string(),
    ///     user_id: "user_123456789".to_string(),
    ///     credentials: Some(credentials),
    /// };
    ///
    /// let deleted_member = WorkspaceMember::delete(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(
        request: WorkspaceMemberDeleteRequest,
    ) -> ApiResponseOrError<WorkspaceMemberDeleted> {
        let credentials_opt = request.credentials.clone();
        let route = format!(
            "organizations/workspaces/{}/members/{}",
            request.workspace_id, request.user_id
        );

        anthropic_request_json(Method::DELETE, &route, |r| r, credentials_opt).await
    }
}

// Builder convenience methods
impl WorkspaceMemberListBuilder {
    /// Creates a new workspace member list request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspace Members API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let members = WorkspaceMemberList::builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .limit(10u32)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<WorkspaceMemberList> {
        let request = self.build().unwrap();
        WorkspaceMemberList::create(request).await
    }
}

impl WorkspaceMemberBuilder {
    /// Creates a new workspace member request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspace Members API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let member = WorkspaceMember::builder("workspace_123456789", "user_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<WorkspaceMember> {
        let request = self.build().unwrap();
        WorkspaceMember::create(request).await
    }
}

impl WorkspaceMemberAddBuilder {
    /// Creates a new workspace member add request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspace Members API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let new_member = WorkspaceMember::add_builder("workspace_123456789")
    ///     .credentials(credentials)
    ///     .user_id("user_123456789")
    ///     .workspace_role(WorkspaceMemberRole::WorkspaceDeveloper)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<WorkspaceMember> {
        let request = self.build().unwrap();
        WorkspaceMember::add(request).await
    }
}

impl WorkspaceMemberUpdateBuilder {
    /// Creates a new workspace member update request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspace Members API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let updated_member = WorkspaceMember::update_builder("workspace_123456789", "user_123456789")
    ///     .credentials(credentials)
    ///     .workspace_role(WorkspaceMemberRole::WorkspaceAdmin)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<WorkspaceMember> {
        let request = self.build().unwrap();
        WorkspaceMember::update(request).await
    }
}

impl WorkspaceMemberDeleteBuilder {
    /// Creates a new workspace member delete request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Workspace Members API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{admin::workspace::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let deleted_member = WorkspaceMember::delete_builder("workspace_123456789", "user_123456789")
    ///     .credentials(credentials)
    ///     .create()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<WorkspaceMemberDeleted> {
        let request = self.build().unwrap();
        WorkspaceMember::delete(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Credentials;

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_list_workspaces() {
        let credentials = Credentials::from_env();

        let workspaces = WorkspaceList::builder()
            .credentials(credentials)
            .create()
            .await
            .unwrap();

        assert!(workspaces.data.len() > 0);
    }

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_get_workspace() {
        let credentials = Credentials::from_env();

        // First get a workspace ID from the list
        let workspaces = WorkspaceList::builder()
            .credentials(credentials.clone())
            .create()
            .await
            .unwrap();

        if let Some(workspace) = workspaces.data.first() {
            let workspace_id = &workspace.id;

            // Then get that specific workspace
            let workspace_details = Workspace::builder(workspace_id)
                .credentials(credentials)
                .create()
                .await
                .unwrap();

            assert_eq!(workspace_details.id, *workspace_id);
        }
    }

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_list_workspace_members() {
        let credentials = Credentials::from_env();

        // First get a workspace ID from the list
        let workspaces = crate::admin::workspace::WorkspaceList::builder()
            .credentials(credentials.clone())
            .create()
            .await
            .unwrap();

        if let Some(workspace) = workspaces.data.first() {
            let workspace_id = &workspace.id;

            // Then list members for that workspace
            let members = WorkspaceMemberList::builder(workspace_id)
                .credentials(credentials)
                .create()
                .await
                .unwrap();

            // Just verify we got a response, may be empty if no members
            assert!(!members.data.is_empty() || (members.data.is_empty() && !members.has_more));
        }
    }

    #[tokio::test]
    #[ignore] // Requires admin API key
    async fn test_get_workspace_member() {
        let credentials = Credentials::from_env();

        // First get a workspace ID from the list
        let workspaces = crate::admin::workspace::WorkspaceList::builder()
            .credentials(credentials.clone())
            .create()
            .await
            .unwrap();

        if let Some(workspace) = workspaces.data.first() {
            let workspace_id = &workspace.id;

            // Then list members for that workspace
            let members = WorkspaceMemberList::builder(workspace_id)
                .credentials(credentials.clone())
                .create()
                .await
                .unwrap();

            // If there are members, get details for the first one
            if let Some(member) = members.data.first() {
                let user_id = &member.user_id;

                let member_details = WorkspaceMember::builder(workspace_id, user_id)
                    .credentials(credentials)
                    .create()
                    .await
                    .unwrap();

                assert_eq!(member_details.user_id, *user_id);
                assert_eq!(member_details.workspace_id, *workspace_id);
            }
        }
    }
}
