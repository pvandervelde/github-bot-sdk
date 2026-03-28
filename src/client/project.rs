// GENERATED FROM: docs/spec/interfaces/project-operations.md
// GitHub Projects v2 operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::InstallationClient;
use crate::error::ApiError;

/// GitHub Projects v2 project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectV2 {
    /// Unique project identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Project number (unique within owner)
    pub number: u64,

    /// Project title
    pub title: String,

    /// Project description
    pub description: Option<String>,

    /// Project owner (organisation or user)
    pub owner: ProjectOwner,

    /// Project visibility
    pub public: bool,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Project URL
    pub url: String,
}

/// Project owner (organisation or user).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectOwner {
    /// Owner login name
    pub login: String,

    /// Owner type
    #[serde(rename = "type")]
    pub owner_type: String, // "Organization" or "User"

    /// Owner ID
    pub id: u64,

    /// Owner node ID
    pub node_id: String,
}

/// Item in a GitHub Projects v2 project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectV2Item {
    /// Unique item identifier (project-specific)
    pub id: String,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Content type
    pub content_type: String, // "Issue" or "PullRequest"

    /// Content node ID (issue or PR node ID)
    pub content_node_id: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Request to add an item to a project.
#[derive(Debug, Clone, Serialize)]
pub struct AddProjectV2ItemRequest {
    /// Node ID of the content to add (issue or pull request)
    pub content_node_id: String,
}

// ---------------------------------------------------------------------------
// GraphQL query for get_issue_linked_projects
// ---------------------------------------------------------------------------

const GET_ISSUE_LINKED_PROJECTS_QUERY: &str = r#"
query GetIssueLinkedProjects($owner: String!, $repo: String!, $number: Int!) {
  repository(owner: $owner, name: $repo) {
    issue(number: $number) {
      projectsV2(first: 20) {
        nodes {
          id
          databaseId
          number
          title
          description
          public
          url
          createdAt
          updatedAt
          owner {
            ... on Organization {
              id
              databaseId
              login
              type: __typename
            }
            ... on User {
              id
              databaseId
              login
              type: __typename
            }
          }
        }
      }
    }
  }
}
"#;

/// Map a single GraphQL `projectsV2.nodes` JSON node to a [`ProjectV2`].
///
/// Returns `None` when required fields are absent or have unexpected types,
/// which causes the node to be silently skipped rather than crashing.
fn map_project_node(node: &serde_json::Value) -> Option<ProjectV2> {
    let id = node.get("databaseId")?.as_u64()?;
    let node_id = node.get("id")?.as_str()?.to_string();
    let number = node.get("number")?.as_u64()?;
    let title = node.get("title")?.as_str()?.to_string();
    let description = node
        .get("description")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());
    let public = node.get("public")?.as_bool()?;
    let url = node.get("url")?.as_str()?.to_string();
    let created_at: DateTime<Utc> = node.get("createdAt")?.as_str()?.parse().ok()?;
    let updated_at: DateTime<Utc> = node.get("updatedAt")?.as_str()?.parse().ok()?;

    let owner_node = node.get("owner")?;
    let owner_login = owner_node.get("login")?.as_str()?.to_string();
    let owner_type = owner_node
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("User")
        .to_string();
    let owner_id = owner_node
        .get("databaseId")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let owner_node_id = owner_node
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(ProjectV2 {
        id,
        node_id,
        number,
        title,
        description,
        owner: ProjectOwner {
            login: owner_login,
            owner_type,
            id: owner_id,
            node_id: owner_node_id,
        },
        public,
        created_at,
        updated_at,
        url,
    })
}

impl InstallationClient {
    // ========================================================================
    // Project Operations
    // ========================================================================

    /// List all Projects v2 for an organisation.
    ///
    /// See docs/spec/interfaces/project-operations.md
    pub async fn list_organization_projects(&self, _org: &str) -> Result<Vec<ProjectV2>, ApiError> {
        unimplemented!("See docs/spec/interfaces/project-operations.md")
    }

    /// List all Projects v2 for a user.
    ///
    /// See docs/spec/interfaces/project-operations.md
    pub async fn list_user_projects(&self, _username: &str) -> Result<Vec<ProjectV2>, ApiError> {
        unimplemented!("See docs/spec/interfaces/project-operations.md")
    }

    /// Get details about a specific project.
    ///
    /// See docs/spec/interfaces/project-operations.md
    pub async fn get_project(
        &self,
        _owner: &str,
        _project_number: u64,
    ) -> Result<ProjectV2, ApiError> {
        unimplemented!("See docs/spec/interfaces/project-operations.md")
    }

    /// Add an issue or pull request to a project.
    ///
    /// See docs/spec/interfaces/project-operations.md
    pub async fn add_item_to_project(
        &self,
        _owner: &str,
        _project_number: u64,
        _content_node_id: &str,
    ) -> Result<ProjectV2Item, ApiError> {
        unimplemented!("See docs/spec/interfaces/project-operations.md")
    }

    /// Get all Projects v2 linked to a specific issue.
    ///
    /// Queries the GitHub GraphQL API for all Projects v2 that contain the given issue.
    /// Returns an empty `Vec` when the issue exists but is not linked to any projects.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (organisation or user login)
    /// * `repo`  - Repository name
    /// * `issue_number` - Issue number
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<ProjectV2>)` — projects linked to the issue (may be empty)
    /// - `Err(ApiError::NotFound)` — repository or issue does not exist
    /// - `Err(ApiError::AuthenticationFailed)` — token is invalid
    /// - `Err(ApiError)` — other transport or GraphQL errors
    pub async fn get_issue_linked_projects(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<ProjectV2>, ApiError> {
        let variables = serde_json::json!({
            "owner": owner,
            "repo": repo,
            "number": issue_number,
        });

        let data = self
            .post_graphql(GET_ISSUE_LINKED_PROJECTS_QUERY, variables)
            .await?;

        let nodes = data
            .get("repository")
            .and_then(|r| r.get("issue"))
            .and_then(|i| i.get("projectsV2"))
            .and_then(|p| p.get("nodes"))
            .and_then(|n| n.as_array());

        let projects = match nodes {
            Some(arr) => arr.iter().filter_map(map_project_node).collect(),
            None => Vec::new(),
        };

        Ok(projects)
    }

    /// Remove an item from a project.
    ///
    /// See docs/spec/interfaces/project-operations.md
    pub async fn remove_item_from_project(
        &self,
        _owner: &str,
        _project_number: u64,
        _item_id: &str,
    ) -> Result<(), ApiError> {
        unimplemented!("See docs/spec/interfaces/project-operations.md")
    }
}

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;
