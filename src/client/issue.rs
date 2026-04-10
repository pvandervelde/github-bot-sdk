// Spec: docs/specs/interfaces/issue-operations.md, labels-client.md,
//       milestones-client.md, reactions.md
// Issue, labels-on-issue, comments, reactions, assignees, lock, timeline,
// and milestone-definition operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::{extract_page_number, parse_link_header, InstallationClient, PagedResponse};
use crate::error::ApiError;

/// Milestone state.
///
/// See docs/specs/interfaces/milestones-client.md
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MilestoneState {
    Open,
    Closed,
}

/// GitHub issue.
///
/// Represents a GitHub issue with all its metadata.
///
/// See docs/specs/interfaces/issue-operations.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Unique issue identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Issue number (repository-specific)
    pub number: u64,

    /// Issue title
    pub title: String,

    /// Issue body content (Markdown)
    pub body: Option<String>,

    /// Issue state
    pub state: String, // "open" | "closed"

    /// Whether the issue is locked
    #[serde(default)]
    pub locked: bool,

    /// User who created the issue
    pub user: IssueUser,

    /// Assigned users
    pub assignees: Vec<IssueUser>,

    /// Applied labels
    pub labels: Vec<Label>,

    /// Milestone
    pub milestone: Option<Milestone>,

    /// Number of comments
    pub comments: u64,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Close timestamp
    pub closed_at: Option<DateTime<Utc>>,

    /// Issue URL
    pub html_url: String,
}

/// User associated with an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueUser {
    /// User login name
    pub login: String,

    /// User ID
    pub id: u64,

    /// User node ID
    pub node_id: String,

    /// User type
    #[serde(rename = "type")]
    pub user_type: String,
}

/// Milestone associated with an issue or pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    /// Unique milestone identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Milestone number (repository-specific)
    pub number: u64,

    /// Milestone title
    pub title: String,

    /// Milestone description
    pub description: Option<String>,

    /// Milestone state
    pub state: MilestoneState,

    /// Number of open issues
    pub open_issues: u32,

    /// Number of closed issues
    pub closed_issues: u32,

    /// Due date
    pub due_on: Option<DateTime<Utc>>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Close timestamp
    pub closed_at: Option<DateTime<Utc>>,
}

/// GitHub label.
///
/// Labels are used to categorize issues and pull requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// Unique label identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Label name
    pub name: String,

    /// Label description
    pub description: Option<String>,

    /// Label color (6-digit hex code without #)
    pub color: String,

    /// Whether this is a default label
    pub default: bool,
}

/// Comment on an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Unique comment identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Comment body content (Markdown)
    pub body: String,

    /// User who created the comment
    pub user: IssueUser,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Comment URL
    pub html_url: String,
}

/// Request to create a new issue.
#[derive(Debug, Clone, Serialize)]
pub struct CreateIssueRequest {
    /// Issue title (required)
    pub title: String,

    /// Issue body content (Markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Usernames to assign
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,

    /// Milestone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<u64>,

    /// Label names to apply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

/// Request to update an existing issue.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateIssueRequest {
    /// Issue title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Issue body content (Markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Issue state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>, // "open" or "closed"

    /// Usernames to assign (replaces existing assignees)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,

    /// Milestone number (None to clear milestone)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<u64>,

    /// Label names (replaces existing labels)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

/// Request to create a label.
#[derive(Debug, Clone, Serialize)]
pub struct CreateLabelRequest {
    /// Label name (required)
    pub name: String,

    /// Label color (6-digit hex code without #)
    pub color: String,

    /// Label description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request to update a label.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateLabelRequest {
    /// New label name. The JSON key sent to GitHub is `"name"`.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub new_name: Option<String>,

    /// Label color (6-digit hex code without #)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Label description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request to create a comment.
#[derive(Debug, Clone, Serialize)]
pub struct CreateCommentRequest {
    /// Comment body content (Markdown, required)
    pub body: String,
}

/// Request to update a comment.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateCommentRequest {
    /// Comment body content (Markdown, required)
    pub body: String,
}

// ============================================================================
// New types introduced by ADR-003 spec
// ============================================================================

/// Reason used when locking an issue.
///
/// See docs/specs/interfaces/issue-operations.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockReason {
    OffTopic,
    TooHeated,
    Resolved,
    Spam,
}

/// A discrete activity event recorded on an issue.
///
/// Returned by the issue events REST endpoint — different from webhook events.
///
/// See docs/specs/interfaces/issue-operations.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueActivityEvent {
    pub id: u64,
    pub event: String, // "labeled", "assigned", "closed", etc.
    pub actor: IssueUser,
    pub created_at: DateTime<Utc>,
    pub label: Option<Label>,
    pub assignee: Option<IssueUser>,
    pub milestone: Option<MilestoneSummary>,
    pub rename: Option<IssueRename>,
}

/// Brief milestone reference used inside activity events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneSummary {
    pub title: String,
}

/// Issue rename payload inside activity events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRename {
    pub from: String,
    pub to: String,
}

/// A single item in an issue's full timeline.
///
/// The `event` field drives deserialization.  Unknown event kinds map to
/// `TimelineEvent::Unknown` without error (via `#[serde(other)]`).
///
/// See docs/specs/interfaces/issue-operations.md
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum TimelineEvent {
    /// A comment posted on the issue. Note: GitHub returns comment objects
    /// here (with `user`, not `actor`) so this variant is structurally
    /// different from the other event kinds.
    #[serde(rename = "commented")]
    Commented {
        id: u64,
        user: IssueUser,
        body: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        html_url: String,
    },
    Labeled {
        id: u64,
        actor: IssueUser,
        label: Label,
        created_at: DateTime<Utc>,
    },
    Unlabeled {
        id: u64,
        actor: IssueUser,
        label: Label,
        created_at: DateTime<Utc>,
    },
    Assigned {
        id: u64,
        actor: IssueUser,
        assignee: IssueUser,
        created_at: DateTime<Utc>,
    },
    Unassigned {
        id: u64,
        actor: IssueUser,
        assignee: IssueUser,
        created_at: DateTime<Utc>,
    },
    Milestoned {
        id: u64,
        actor: IssueUser,
        milestone: MilestoneSummary,
        created_at: DateTime<Utc>,
    },
    Demilestoned {
        id: u64,
        actor: IssueUser,
        milestone: MilestoneSummary,
        created_at: DateTime<Utc>,
    },
    Closed {
        id: u64,
        actor: IssueUser,
        created_at: DateTime<Utc>,
    },
    Reopened {
        id: u64,
        actor: IssueUser,
        created_at: DateTime<Utc>,
    },
    Locked {
        id: u64,
        actor: IssueUser,
        lock_reason: Option<String>,
        created_at: DateTime<Utc>,
    },
    Unlocked {
        id: u64,
        actor: IssueUser,
        created_at: DateTime<Utc>,
    },
    Renamed {
        id: u64,
        actor: IssueUser,
        rename: IssueRename,
        created_at: DateTime<Utc>,
    },
    Referenced {
        id: u64,
        actor: IssueUser,
        created_at: DateTime<Utc>,
    },
    /// Catch-all: unknown event kind — must not cause a deserialization error.
    #[serde(other)]
    Unknown,
}

/// Emoji content for a GitHub reaction.
///
/// The `+1` and `-1` variants require explicit `#[serde(rename)]` because
/// their GitHub API names start with symbols.
///
/// See docs/specs/interfaces/reactions.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactionContent {
    /// 👍
    #[serde(rename = "+1")]
    PlusOne,
    /// 👎
    #[serde(rename = "-1")]
    MinusOne,
    /// 😄
    Laugh,
    /// 😕
    Confused,
    /// ❤️
    Heart,
    /// 🎉
    Hooray,
    /// 🚀
    Rocket,
    /// 👀
    Eyes,
}

/// A single reaction on an issue or issue comment.
///
/// See docs/specs/interfaces/reactions.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub id: u64,
    pub node_id: String,
    pub user: IssueUser,
    pub content: ReactionContent,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Milestone-management types (MilestonesClient)
// ============================================================================

/// Sort direction for list queries.
///
/// See docs/specs/interfaces/milestones-client.md
#[derive(Debug, Clone)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Sort field for milestone list queries.
///
/// See docs/specs/interfaces/milestones-client.md
#[derive(Debug, Clone)]
pub enum MilestoneSortField {
    DueOn,
    Completeness,
}

/// Filter and sort options for listing milestones.
///
/// See docs/specs/interfaces/milestones-client.md
#[derive(Debug, Clone, Default)]
pub struct ListMilestonesQuery {
    pub state: Option<MilestoneState>,
    pub sort: Option<MilestoneSortField>,
    pub direction: Option<SortDirection>,
}

/// Request to create a new milestone.
///
/// See docs/specs/interfaces/milestones-client.md
#[derive(Debug, Clone, Serialize)]
pub struct CreateMilestoneRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<MilestoneState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<DateTime<Utc>>,
}

/// Request to update an existing milestone.
///
/// See docs/specs/interfaces/milestones-client.md
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateMilestoneRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<MilestoneState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<DateTime<Utc>>,
}

// ============================================================================
// Private request body helpers (not part of the public API)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LabelsRequest {
    pub(crate) labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LockIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    lock_reason: Option<LockReason>,
}

#[derive(Debug, Clone, Serialize)]
struct AssigneesRequest {
    assignees: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CreateReactionRequest {
    content: ReactionContent,
}

// ============================================================================
// Shared error-mapping helper
// ============================================================================

async fn map_error(status: reqwest::StatusCode, response: reqwest::Response) -> ApiError {
    match status.as_u16() {
        401 => ApiError::AuthenticationFailed,
        403 => ApiError::AuthorizationFailed,
        404 => ApiError::NotFound,
        422 => {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Validation failed".to_string());
            ApiError::InvalidRequest { message }
        }
        _ => {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            ApiError::HttpError {
                status: status.as_u16(),
                message,
            }
        }
    }
}

// ============================================================================
// IssuesClient
// ============================================================================

/// Domain client for GitHub issue operations.
///
/// Obtained via [`InstallationClient::issues()`].  Cheap to clone (Arc-backed).
///
/// Covers issue CRUD, comment CRUD, reactions, label application, assignees,
/// lock/unlock, and timeline queries.
///
/// See docs/specs/interfaces/issue-operations.md
#[derive(Debug, Clone)]
pub struct IssuesClient {
    client: InstallationClient,
}

impl IssuesClient {
    pub(crate) fn new(client: InstallationClient) -> Self {
        Self { client }
    }

    // --- Issue CRUD ---

    /// List issues in a repository (manual pagination).
    ///
    /// # Arguments
    /// * `state` — `"open"` (default), `"closed"`, or `"all"`
    /// * `page` — 1-indexed page number; omit for first page
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
        page: Option<u32>,
    ) -> Result<PagedResponse<Issue>, ApiError> {
        let mut path = format!("/repos/{}/{}/issues", owner, repo);
        let mut params: Vec<String> = Vec::new();
        if let Some(s) = state {
            params.push(format!("state={}", s));
        }
        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if !params.is_empty() {
            path = format!("{}?{}", path, params.join("&"));
        }

        let response = self.client.get(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }

        let pagination = response
            .headers()
            .get("Link")
            .and_then(|h| h.to_str().ok())
            .map(|h| parse_link_header(Some(h)))
            .unwrap_or_default();

        let items: Vec<Issue> = response.json().await.map_err(ApiError::from)?;
        Ok(PagedResponse {
            items,
            total_count: None,
            pagination,
        })
    }

    /// Get a single issue by number.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — issue does not exist
    pub async fn get(&self, owner: &str, repo: &str, issue_number: u64) -> Result<Issue, ApiError> {
        let path = format!("/repos/{}/{}/issues/{}", owner, repo, issue_number);
        let response = self.client.get(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Create a new issue.
    ///
    /// # Errors
    /// * `ApiError::InvalidRequest` — validation failed (empty title, etc.)
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: CreateIssueRequest,
    ) -> Result<Issue, ApiError> {
        let path = format!("/repos/{}/{}/issues", owner, repo);
        let response = self.client.post(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Update an existing issue (patch semantics — only set fields are changed).
    ///
    /// # Errors
    /// * `ApiError::NotFound` — issue does not exist
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        request: UpdateIssueRequest,
    ) -> Result<Issue, ApiError> {
        let path = format!("/repos/{}/{}/issues/{}", owner, repo, issue_number);
        let response = self.client.patch(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Set (or clear) the milestone on an issue.
    ///
    /// Pass `None` to remove the milestone.
    pub async fn set_milestone(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        milestone_number: Option<u64>,
    ) -> Result<Issue, ApiError> {
        self.update(
            owner,
            repo,
            issue_number,
            UpdateIssueRequest {
                milestone: milestone_number,
                ..Default::default()
            },
        )
        .await
    }

    // --- Comment operations ---

    /// List all comments on an issue.
    ///
    /// Auto-paginates with `per_page=100` (ADR-002).  Oldest comments first.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — issue does not exist (no partial list returned)
    pub async fn list_comments(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<Comment>, ApiError> {
        let base = format!("/repos/{}/{}/issues/{}/comments", owner, repo, issue_number);
        self.fetch_all(&base).await
    }

    /// Get a single comment by its repository-scoped ID.
    pub async fn get_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<Comment, ApiError> {
        let path = format!("/repos/{}/{}/issues/comments/{}", owner, repo, comment_id);
        let response = self.client.get(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Create a comment on an issue.
    pub async fn create_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        request: CreateCommentRequest,
    ) -> Result<Comment, ApiError> {
        let path = format!("/repos/{}/{}/issues/{}/comments", owner, repo, issue_number);
        let response = self.client.post(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Update an existing comment.
    pub async fn update_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        request: UpdateCommentRequest,
    ) -> Result<Comment, ApiError> {
        let path = format!("/repos/{}/{}/issues/comments/{}", owner, repo, comment_id);
        let response = self.client.patch(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Delete a comment.
    pub async fn delete_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/comments/{}", owner, repo, comment_id);
        let response = self.client.delete(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        Ok(())
    }

    // --- Label application on issue ---

    /// List all labels applied to an issue (auto-paginated).
    pub async fn list_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<Label>, ApiError> {
        let base = format!("/repos/{}/{}/issues/{}/labels", owner, repo, issue_number);
        self.fetch_all(&base).await
    }

    /// Add one or more labels to an issue (idempotent — duplicates ignored).
    ///
    /// Returns the updated label list on the issue.
    pub async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        labels: Vec<String>,
    ) -> Result<Vec<Label>, ApiError> {
        let path = format!("/repos/{}/{}/issues/{}/labels", owner, repo, issue_number);
        let body = LabelsRequest { labels };
        let response = self.client.post(&path, &body).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Remove a single label from an issue.
    ///
    /// Returns the remaining label list.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — label is not applied to this issue
    pub async fn remove_label(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        label_name: &str,
    ) -> Result<Vec<Label>, ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/{}/labels/{}",
            owner, repo, issue_number, urlencoding::encode(label_name)
        );
        let response = self.client.delete(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Replace all labels on an issue atomically.
    ///
    /// Any labels not in `labels` are removed.  Pass an empty vec to remove all labels.
    /// Returns the new label list as applied by GitHub.
    pub async fn replace_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        labels: Vec<String>,
    ) -> Result<Vec<Label>, ApiError> {
        let path = format!("/repos/{}/{}/issues/{}/labels", owner, repo, issue_number);
        let body = LabelsRequest { labels };
        let response = self.client.put(&path, &body).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    // --- Reactions ---

    /// List all reactions on an issue (auto-paginated).
    pub async fn list_reactions(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<Reaction>, ApiError> {
        let base = format!(
            "/repos/{}/{}/issues/{}/reactions",
            owner, repo, issue_number
        );
        self.fetch_all(&base).await
    }

    /// Add a reaction to an issue.
    ///
    /// If the same user has already reacted with this emoji, GitHub returns the
    /// existing reaction (HTTP 200). Both 200 and 201 map to `Ok(Reaction)`.
    pub async fn create_reaction(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        content: ReactionContent,
    ) -> Result<Reaction, ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/{}/reactions",
            owner, repo, issue_number
        );
        let body = CreateReactionRequest { content };
        let response = self.client.post(&path, &body).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Remove a reaction from an issue.
    pub async fn delete_reaction(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        reaction_id: u64,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/{}/reactions/{}",
            owner, repo, issue_number, reaction_id
        );
        let response = self.client.delete(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        Ok(())
    }

    /// List all reactions on an issue comment (auto-paginated).
    pub async fn list_comment_reactions(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<Vec<Reaction>, ApiError> {
        let base = format!(
            "/repos/{}/{}/issues/comments/{}/reactions",
            owner, repo, comment_id
        );
        self.fetch_all(&base).await
    }

    /// Add a reaction to an issue comment.
    pub async fn create_comment_reaction(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        content: ReactionContent,
    ) -> Result<Reaction, ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/comments/{}/reactions",
            owner, repo, comment_id
        );
        let body = CreateReactionRequest { content };
        let response = self.client.post(&path, &body).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Remove a reaction from an issue comment.
    pub async fn delete_comment_reaction(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        reaction_id: u64,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/comments/{}/reactions/{}",
            owner, repo, comment_id, reaction_id
        );
        let response = self.client.delete(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        Ok(())
    }

    // --- Assignees ---

    /// List all users eligible to be assigned in this repository (auto-paginated).
    pub async fn list_available_assignees(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<IssueUser>, ApiError> {
        let base = format!("/repos/{}/{}/assignees", owner, repo);
        self.fetch_all(&base).await
    }

    /// Add assignees to an issue.
    ///
    /// GitHub silently ignores users who are not eligible.
    /// Returns the updated issue with the new assignee list.
    pub async fn add_assignees(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        assignees: Vec<String>,
    ) -> Result<Issue, ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/{}/assignees",
            owner, repo, issue_number
        );
        let body = AssigneesRequest { assignees };
        let response = self.client.post(&path, &body).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Remove assignees from an issue.
    ///
    /// GitHub silently ignores users not currently assigned.
    /// Returns the updated issue with the revised assignee list.
    pub async fn remove_assignees(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        assignees: Vec<String>,
    ) -> Result<Issue, ApiError> {
        let path = format!(
            "/repos/{}/{}/issues/{}/assignees",
            owner, repo, issue_number
        );
        let body = AssigneesRequest { assignees };
        let response = self.client.delete_with_body(&path, &body).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    // --- Lock / Unlock ---

    /// Lock an issue, preventing non-collaborator comments.
    ///
    /// # Arguments
    /// * `reason` — optional lock reason shown to users attempting to comment
    ///
    /// # Errors
    /// * `ApiError::AuthorizationFailed` — requires admin or maintain permission
    pub async fn lock(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        reason: Option<LockReason>,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}/lock", owner, repo, issue_number);
        let body = LockIssueRequest {
            lock_reason: reason,
        };
        let response = self.client.put(&path, &body).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        Ok(())
    }

    /// Unlock a previously locked issue.
    ///
    /// # Errors
    /// * `ApiError::AuthorizationFailed` — requires admin or maintain permission
    pub async fn unlock(&self, owner: &str, repo: &str, issue_number: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/issues/{}/lock", owner, repo, issue_number);
        let response = self.client.delete(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        Ok(())
    }

    // --- Activity events & timeline ---

    /// List all discrete activity events on an issue (auto-paginated).
    ///
    /// Returns events in ascending chronological order (oldest first).
    pub async fn list_activity_events(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<IssueActivityEvent>, ApiError> {
        let base = format!("/repos/{}/{}/issues/{}/events", owner, repo, issue_number);
        self.fetch_all(&base).await
    }

    /// List the complete timeline of an issue (auto-paginated).
    ///
    /// Superset of [`list_activity_events`]: also includes comments and
    /// cross-references.  Unknown event kinds yield `TimelineEvent::Unknown`.
    pub async fn list_timeline(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<TimelineEvent>, ApiError> {
        let base = format!("/repos/{}/{}/issues/{}/timeline", owner, repo, issue_number);
        self.fetch_all(&base).await
    }

    // --- Private: auto-pagination ---

    /// Fetch all pages of a GET endpoint that returns `Vec<T>`.
    ///
    /// Uses `per_page=100` and follows `Link: rel="next"` headers (ADR-002).
    async fn fetch_all<T: serde::de::DeserializeOwned>(
        &self,
        base_path: &str,
    ) -> Result<Vec<T>, ApiError> {
        let mut all_items: Vec<T> = Vec::new();
        let mut path = format!("{}?per_page=100", base_path);

        loop {
            let response = self.client.get(&path).await?;
            let status = response.status();
            if !status.is_success() {
                return Err(map_error(status, response).await);
            }

            let next_page: Option<u32> = response
                .headers()
                .get("Link")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| parse_link_header(Some(h)).next)
                .as_deref()
                .and_then(extract_page_number);

            let items: Vec<T> = response.json().await.map_err(ApiError::from)?;
            all_items.extend(items);

            match next_page {
                Some(page) => {
                    path = format!("{}?per_page=100&page={}", base_path, page);
                }
                None => break,
            }
        }

        Ok(all_items)
    }
}

// ============================================================================
// LabelsClient
// ============================================================================

/// Domain client for repository-level label catalogue operations.
///
/// Obtained via [`InstallationClient::labels()`].  Cheap to clone (Arc-backed).
///
/// Manages label *definitions*.  To apply labels to a specific issue use
/// [`IssuesClient`]; to a PR use `PullRequestsClient`.
///
/// See docs/specs/interfaces/labels-client.md
#[derive(Debug, Clone)]
pub struct LabelsClient {
    client: InstallationClient,
}

impl LabelsClient {
    pub(crate) fn new(client: InstallationClient) -> Self {
        Self { client }
    }

    /// List all label definitions in a repository (auto-paginated).
    ///
    /// # Errors
    /// * `ApiError::NotFound` — repository does not exist
    pub async fn list(&self, owner: &str, repo: &str) -> Result<Vec<Label>, ApiError> {
        let first_page = format!("/repos/{}/{}/labels?per_page=100", owner, repo);
        self.client.fetch_all_pages(&first_page).await
    }

    /// Get a single label definition by name.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — label does not exist
    pub async fn get(&self, owner: &str, repo: &str, name: &str) -> Result<Label, ApiError> {
        let path = format!("/repos/{}/{}/labels/{}", owner, repo, urlencoding::encode(name));
        let response = self.client.get(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Create a new label definition.
    ///
    /// # Errors
    /// * `ApiError::InvalidRequest` — name already exists (422)
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: CreateLabelRequest,
    ) -> Result<Label, ApiError> {
        let path = format!("/repos/{}/{}/labels", owner, repo);
        let response = self.client.post(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Update an existing label definition.
    ///
    /// Renaming via `new_name` updates all existing issue/PR references.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — label does not exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        request: UpdateLabelRequest,
    ) -> Result<Label, ApiError> {
        let path = format!("/repos/{}/{}/labels/{}", owner, repo, urlencoding::encode(name));
        let response = self.client.patch(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Delete a label definition.
    ///
    /// Deleting a label removes it from all issues and PRs it was applied to.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — label does not exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn delete(&self, owner: &str, repo: &str, name: &str) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/labels/{}", owner, repo, urlencoding::encode(name));
        let response = self.client.delete(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        Ok(())
    }
}

// ============================================================================
// MilestonesClient
// ============================================================================

/// Domain client for milestone lifecycle operations.
///
/// Obtained via [`InstallationClient::milestones()`].  Cheap to clone (Arc-backed).
///
/// Manages milestone *definitions*.  To assign a milestone to a specific issue
/// use [`IssuesClient::set_milestone`]; for a PR use `PullRequestsClient::set_milestone`.
///
/// See docs/specs/interfaces/milestones-client.md
#[derive(Debug, Clone)]
pub struct MilestonesClient {
    client: InstallationClient,
}

impl MilestonesClient {
    pub(crate) fn new(client: InstallationClient) -> Self {
        Self { client }
    }

    /// List all milestones in a repository (auto-paginated).
    ///
    /// # Errors
    /// * `ApiError::NotFound` — repository does not exist
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
        query: Option<ListMilestonesQuery>,
    ) -> Result<Vec<Milestone>, ApiError> {
        let base = format!("/repos/{}/{}/milestones", owner, repo);
        let mut params: Vec<String> = vec!["per_page=100".to_string()];

        if let Some(q) = query {
            if let Some(state) = q.state {
                let s = match state {
                    MilestoneState::Open => "open",
                    MilestoneState::Closed => "closed",
                };
                params.push(format!("state={}", s));
            }
            if let Some(sort) = q.sort {
                let s = match sort {
                    MilestoneSortField::DueOn => "due_on",
                    MilestoneSortField::Completeness => "completeness",
                };
                params.push(format!("sort={}", s));
            }
            if let Some(dir) = q.direction {
                let d = match dir {
                    SortDirection::Asc => "asc",
                    SortDirection::Desc => "desc",
                };
                params.push(format!("direction={}", d));
            }
        }

        let first_page = format!("{}?{}", base, params.join("&"));
        self.client.fetch_all_pages(&first_page).await
    }

    /// Get a single milestone by its repository-scoped number.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — milestone does not exist
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        milestone_number: u64,
    ) -> Result<Milestone, ApiError> {
        let path = format!("/repos/{}/{}/milestones/{}", owner, repo, milestone_number);
        let response = self.client.get(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Create a new milestone.
    ///
    /// # Errors
    /// * `ApiError::InvalidRequest` — title is empty
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: CreateMilestoneRequest,
    ) -> Result<Milestone, ApiError> {
        let path = format!("/repos/{}/{}/milestones", owner, repo);
        let response = self.client.post(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Update an existing milestone.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — milestone does not exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        milestone_number: u64,
        request: UpdateMilestoneRequest,
    ) -> Result<Milestone, ApiError> {
        let path = format!("/repos/{}/{}/milestones/{}", owner, repo, milestone_number);
        let response = self.client.patch(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Delete a milestone.
    ///
    /// Issues assigned to the deleted milestone are unlinked but otherwise unaffected.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — milestone does not exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn delete(
        &self,
        owner: &str,
        repo: &str,
        milestone_number: u64,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/milestones/{}", owner, repo, milestone_number);
        let response = self.client.delete(&path).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(map_error(status, response).await);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "issue_tests.rs"]
mod tests;
