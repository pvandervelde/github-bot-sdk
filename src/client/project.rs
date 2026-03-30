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
query GetIssueLinkedProjects($owner: String!, $repo: String!, $number: Int!, $cursor: String) {
  repository(owner: $owner, name: $repo) {
    issue(number: $number) {
      projectsV2(first: 20, after: $cursor) {
        pageInfo {
          hasNextPage
          endCursor
        }
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
    // The query always selects `databaseId` on both Organization and User owner fragments;
    // `unwrap_or(0)` is a defensive default for a query-guaranteed-present field.
    let owner_id = owner_node
        .get("databaseId")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    // Similarly, `id` (the owner's global node ID) is always selected by the query;
    // `unwrap_or("")` is a defensive default that is not reached in practice.
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

// ---------------------------------------------------------------------------
// GraphQL queries and mutations for project item operations
// ---------------------------------------------------------------------------

const GET_PROJECT_NODE_ID_ORG_QUERY: &str = r#"
query GetProjectNodeIdOrg($owner: String!, $number: Int!) {
  organization(login: $owner) {
    projectV2(number: $number) {
      id
    }
  }
}
"#;

const GET_PROJECT_NODE_ID_USER_QUERY: &str = r#"
query GetProjectNodeIdUser($owner: String!, $number: Int!) {
  user(login: $owner) {
    projectV2(number: $number) {
      id
    }
  }
}
"#;

const ADD_PROJECT_ITEM_MUTATION: &str = r#"
mutation AddProjectV2Item($projectId: ID!, $contentId: ID!) {
  addProjectV2ItemById(input: { projectId: $projectId, contentId: $contentId }) {
    item {
      id
      type
      createdAt
      updatedAt
      content {
        ... on Issue { id }
        ... on PullRequest { id }
      }
    }
  }
}
"#;

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
    /// Resolves the project node ID from `owner` + `project_number` (trying organisation
    /// first, then falling back to user), then calls the `addProjectV2ItemById` GraphQL
    /// mutation to attach the content.
    ///
    /// # Arguments
    ///
    /// * `owner`          - Organisation or user login name
    /// * `project_number` - Project number (unique within owner)
    /// * `content_node_id` - Node ID of the issue or pull request to add
    ///
    /// # Returns
    ///
    /// - `Ok(ProjectV2Item)` — the newly created project item
    /// - `Err(ApiError::NotFound)` — project not found for this owner
    /// - `Err(ApiError::AuthorizationFailed)` — no write access to the project
    /// - `Err(ApiError)` — other transport or GraphQL errors
    pub async fn add_item_to_project(
        &self,
        owner: &str,
        project_number: u64,
        content_node_id: &str,
    ) -> Result<ProjectV2Item, ApiError> {
        let project_node_id = self.get_project_node_id(owner, project_number).await?;

        let variables = serde_json::json!({
            "projectId": project_node_id,
            "contentId": content_node_id,
        });

        let data = self
            .post_graphql(ADD_PROJECT_ITEM_MUTATION, variables)
            .await?;

        let item = data
            .get("addProjectV2ItemById")
            .and_then(|a| a.get("item"))
            .ok_or_else(|| ApiError::GraphQlError {
                message: "addProjectV2ItemById returned no item".to_string(),
            })?;

        let item_id = item
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::GraphQlError {
                message: "project item missing id field".to_string(),
            })?
            .to_string();

        let content_type = item
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::GraphQlError {
                message: "project item missing type field".to_string(),
            })?
            .to_string();

        let created_at: DateTime<Utc> = item
            .get("createdAt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::GraphQlError {
                message: "project item missing createdAt field".to_string(),
            })?
            .parse()
            .map_err(|_| ApiError::GraphQlError {
                message: "project item createdAt is not a valid timestamp".to_string(),
            })?;

        let updated_at: DateTime<Utc> = item
            .get("updatedAt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::GraphQlError {
                message: "project item missing updatedAt field".to_string(),
            })?
            .parse()
            .map_err(|_| ApiError::GraphQlError {
                message: "project item updatedAt is not a valid timestamp".to_string(),
            })?;

        // content.id is the node ID of the linked issue or PR.
        let linked_content_node_id = item
            .get("content")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or(content_node_id)
            .to_string();

        Ok(ProjectV2Item {
            id: item_id.clone(),
            // GitHub Projects v2 exposes only a single `id` (the global node ID) for
            // ProjectV2Item objects — there is no separate integer `databaseId`. Both
            // `id` and `node_id` therefore carry the same value.
            node_id: item_id,
            content_type,
            content_node_id: linked_content_node_id,
            created_at,
            updated_at,
        })
    }

    /// Resolve an owner + project number to the project's GraphQL node ID.
    ///
    /// Attempts an organisation query first. If the response carries a
    /// `NOT_FOUND` error the query is retried against the user namespace.
    /// Returns `ApiError::NotFound` when neither lookup succeeds.
    async fn get_project_node_id(
        &self,
        owner: &str,
        project_number: u64,
    ) -> Result<String, ApiError> {
        let variables = serde_json::json!({
            "owner": owner,
            // Cast to i64: GraphQL Int! is 32-bit signed; realistic project numbers
            // are well within that range.
            "number": project_number as i64,
        });

        // Try organisation first.
        match self
            .post_graphql(GET_PROJECT_NODE_ID_ORG_QUERY, variables.clone())
            .await
        {
            Ok(data) => {
                if let Some(id) = data
                    .get("organization")
                    .and_then(|o| o.get("projectV2"))
                    .and_then(|p| p.get("id"))
                    .and_then(|v| v.as_str())
                {
                    return Ok(id.to_string());
                }
                // data.organization.projectV2 was null — fall through to user lookup.
            }
            Err(ApiError::NotFound) => {
                // org not found — fall through to user lookup.
            }
            Err(other) => return Err(other),
        }

        // Fall back to user lookup.
        let data = self
            .post_graphql(GET_PROJECT_NODE_ID_USER_QUERY, variables)
            .await?;

        data.get("user")
            .and_then(|u| u.get("projectV2"))
            .and_then(|p| p.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or(ApiError::NotFound)
    }

    /// Get all Projects v2 linked to a specific issue.
    ///
    /// Queries the GitHub GraphQL API for all Projects v2 that contain the given issue.
    /// Returns an empty `Vec` when the issue exists but is not linked to any projects.
    /// Results are fetched in pages of 20; all pages are retrieved automatically and
    /// returned as a single combined `Vec`.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner (organisation or user login)
    /// * `repo`  - Repository name
    /// * `issue_number` - Issue number
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<ProjectV2>)` — all projects linked to the issue (may be empty)
    /// - `Err(ApiError::NotFound)` — repository or issue does not exist
    /// - `Err(ApiError::AuthenticationFailed)` — token is invalid
    /// - `Err(ApiError)` — other transport or GraphQL errors
    pub async fn get_issue_linked_projects(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<ProjectV2>, ApiError> {
        let mut all_projects = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let variables = serde_json::json!({
                "owner": owner,
                "repo": repo,
                // Cast to i64: GraphQL Int! is 32-bit signed; realistic issue numbers
                // are well within that range.
                "number": issue_number as i64,
                "cursor": cursor,
            });

            let data = self
                .post_graphql(GET_ISSUE_LINKED_PROJECTS_QUERY, variables)
                .await?;

            let projects_v2 = match data
                .get("repository")
                .and_then(|r| r.get("issue"))
                .and_then(|i| i.get("projectsV2"))
            {
                Some(pv2) => pv2,
                None => break,
            };

            if let Some(nodes) = projects_v2.get("nodes").and_then(|n| n.as_array()) {
                all_projects.extend(nodes.iter().filter_map(map_project_node));
            }

            let has_next_page = projects_v2
                .get("pageInfo")
                .and_then(|p| p.get("hasNextPage"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !has_next_page {
                break;
            }

            cursor = projects_v2
                .get("pageInfo")
                .and_then(|p| p.get("endCursor"))
                .and_then(|v| v.as_str())
                .map(String::from);
        }

        Ok(all_projects)
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
