// Spec: docs/specs/interfaces/pull-request-operations.md
// Pull request, review, comment, and label operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::issue::{Comment, IssueUser, Label, LabelsRequest, Milestone};
use crate::client::{parse_link_header, InstallationClient, PagedResponse};
use crate::error::ApiError;

/// GitHub pull request.
///
/// Represents a pull request with all its metadata.
///
/// See docs/spec/interfaces/pull-request-operations.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// Unique pull request identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Pull request number (repository-specific)
    pub number: u64,

    /// Pull request title
    pub title: String,

    /// Pull request body content (Markdown)
    pub body: Option<String>,

    /// Pull request state
    pub state: String, // "open", "closed"

    /// User who created the pull request
    pub user: IssueUser,

    /// Head branch information
    pub head: PullRequestBranch,

    /// Base branch information
    pub base: PullRequestBranch,

    /// Whether the pull request is a draft
    pub draft: bool,

    /// Whether the pull request is merged
    pub merged: bool,

    /// Whether the pull request is mergeable
    pub mergeable: Option<bool>,

    /// Merge commit SHA (if merged)
    pub merge_commit_sha: Option<String>,

    /// Assigned users
    pub assignees: Vec<IssueUser>,

    /// Requested reviewers
    pub requested_reviewers: Vec<IssueUser>,

    /// Applied labels
    pub labels: Vec<Label>,

    /// Milestone
    pub milestone: Option<Milestone>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Close timestamp
    pub closed_at: Option<DateTime<Utc>>,

    /// Merge timestamp
    pub merged_at: Option<DateTime<Utc>>,

    /// Pull request URL
    pub html_url: String,
}

/// Branch information in a pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestBranch {
    /// Branch name
    #[serde(rename = "ref")]
    pub branch_ref: String,

    /// Commit SHA
    pub sha: String,

    /// Repository information
    pub repo: PullRequestRepo,
}

/// Repository information in a pull request branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestRepo {
    /// Repository ID
    pub id: u64,

    /// Repository name
    pub name: String,

    /// Full repository name (owner/repo)
    pub full_name: String,
}

/// Pull request review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Unique review identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// User who submitted the review
    pub user: IssueUser,

    /// Review body content (Markdown)
    pub body: Option<String>,

    /// Review state
    pub state: String, // "APPROVED", "CHANGES_REQUESTED", "COMMENTED", "DISMISSED", "PENDING"

    /// Commit SHA that was reviewed
    pub commit_id: String,

    /// Creation timestamp
    pub submitted_at: Option<DateTime<Utc>>,

    /// Review URL
    pub html_url: String,
}

/// Comment on a pull request (review comment on code).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestComment {
    /// Unique comment identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Comment body content (Markdown)
    pub body: String,

    /// User who created the comment
    pub user: IssueUser,

    /// File path
    pub path: String,

    /// Line number (if single-line comment)
    pub line: Option<u64>,

    /// Commit SHA
    pub commit_id: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Comment URL
    pub html_url: String,
}

/// Request to create a new pull request.
#[derive(Debug, Clone, Serialize)]
pub struct CreatePullRequestRequest {
    /// Pull request title (required)
    pub title: String,

    /// Head branch (required) - format: "username:branch" for forks
    pub head: String,

    /// Base branch (required)
    pub base: String,

    /// Pull request body content (Markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Whether to create as draft
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,

    /// Milestone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<u64>,

    /// Whether maintainers of the base repository can push to the head branch.
    ///
    /// When `true`, maintainers of the base repository (contributors with push
    /// access) can push commits to the head branch of this pull request, even
    /// when the head branch lives in a fork. Defaults to `true` on the GitHub
    /// API for fork-sourced pull requests when not provided.
    ///
    /// # Example
    ///
    /// ```
    /// use github_bot_sdk::client::CreatePullRequestRequest;
    ///
    /// let request = CreatePullRequestRequest {
    ///     title: "My feature".to_string(),
    ///     head: "contributor:feature-branch".to_string(),
    ///     base: "main".to_string(),
    ///     body: None,
    ///     draft: None,
    ///     milestone: None,
    ///     maintainer_can_modify: Some(true),
    /// };
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer_can_modify: Option<bool>,
}

/// Request to update an existing pull request.
///
/// Note: milestone assignment is not supported here — the GitHub Pulls API
/// silently ignores the `milestone` field. Use `PullRequestsClient::set_milestone`
/// which delegates to the Issues API endpoint that actually applies the milestone.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdatePullRequestRequest {
    /// Pull request title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Pull request body content (Markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Pull request state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>, // "open" or "closed"

    /// Base branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
}

/// Request to merge a pull request.
#[derive(Debug, Clone, Serialize, Default)]
pub struct MergePullRequestRequest {
    /// Merge commit message title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_title: Option<String>,

    /// Merge commit message body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_message: Option<String>,

    /// SHA that pull request head must match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,

    /// Merge method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_method: Option<String>, // "merge", "squash", "rebase"
}

/// Result of merging a pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Whether the merge was successful
    pub merged: bool,

    /// Merge commit SHA
    pub sha: String,

    /// Message describing the result
    pub message: String,
}

/// Request to create a review.
#[derive(Debug, Clone, Serialize)]
pub struct CreateReviewRequest {
    /// Commit SHA to review (optional, defaults to PR head)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_id: Option<String>,

    /// Review body content (Markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Review event
    pub event: String, // "APPROVE", "REQUEST_CHANGES", "COMMENT"
}

/// Request to update a review.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateReviewRequest {
    /// Review body content (Markdown, required)
    pub body: String,
}

/// Request to dismiss a review.
#[derive(Debug, Clone, Serialize)]
pub struct DismissReviewRequest {
    /// Dismissal message (required)
    pub message: String,
}

/// Request to create a pull request comment.
#[derive(Debug, Clone, Serialize)]
pub struct CreatePullRequestCommentRequest {
    /// Comment body content (Markdown, required)
    pub body: String,
}

/// Request to update a pull request comment.
#[derive(Debug, Clone, Serialize)]
pub struct UpdatePullRequestCommentRequest {
    /// Comment body content (Markdown, required)
    pub body: String,
}

// ============================================================================
// PullRequestsClient
// ============================================================================

/// Domain client for pull request operations.
///
/// Obtained via [`InstallationClient::pull_requests()`]. Cheap to clone (Arc-backed).
///
/// See docs/specs/interfaces/pull-request-operations.md
#[derive(Debug, Clone)]
pub struct PullRequestsClient {
    client: InstallationClient,
}

impl PullRequestsClient {
    pub(crate) fn new(client: InstallationClient) -> Self {
        Self { client }
    }

    // --- Pull Request CRUD ---

    /// List pull requests in a repository.
    ///
    /// Returns a paginated response with pull requests and pagination metadata.
    /// Use the pagination information to fetch subsequent pages if needed.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `state` - Filter by state (`"open"`, `"closed"`, or `"all"`)
    /// * `page` - Page number (1-indexed, omit for first page)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::PullRequestsClient;
    /// # async fn example(client: &PullRequestsClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get first page
    /// let response = client.list("owner", "repo", None, None).await?;
    /// println!("Got {} pull requests", response.items.len());
    ///
    /// // Check if more pages exist
    /// if response.has_next() {
    ///     if let Some(next_page) = response.next_page_number() {
    ///         let next_response = client.list("owner", "repo", None, Some(next_page)).await?;
    ///         println!("Got {} more PRs", next_response.items.len());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
        page: Option<u32>,
    ) -> Result<PagedResponse<PullRequest>, ApiError> {
        let mut path = format!("/repos/{}/{}/pulls", owner, repo);
        let mut query_params = Vec::new();

        if let Some(state_value) = state {
            query_params.push(format!("state={}", state_value));
        }
        if let Some(page_num) = page {
            query_params.push(format!("page={}", page_num));
        }

        if !query_params.is_empty() {
            path = format!("{}?{}", path, query_params.join("&"));
        }

        let response = self.client.get(&path).await?;
        let status = response.status();

        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }

        // Parse Link header for pagination
        let pagination = response
            .headers()
            .get("Link")
            .and_then(|h| h.to_str().ok())
            .map(|h| parse_link_header(Some(h)))
            .unwrap_or_default();

        // Parse response body
        let items: Vec<PullRequest> = response.json().await.map_err(ApiError::from)?;

        Ok(PagedResponse {
            items,
            total_count: None, // GitHub doesn't provide total count in list responses
            pagination,
        })
    }

    /// Get a specific pull request by number.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<PullRequest, ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}", owner, repo, pull_number);
        let response = self.client.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Create a new pull request.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: CreatePullRequestRequest,
    ) -> Result<PullRequest, ApiError> {
        let path = format!("/repos/{}/{}/pulls", owner, repo);
        let response = self.client.post(&path, &request).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                422 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Validation failed".to_string());
                    ApiError::InvalidRequest { message }
                }
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Update an existing pull request.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        request: UpdatePullRequestRequest,
    ) -> Result<PullRequest, ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}", owner, repo, pull_number);
        let response = self.client.patch(&path, &request).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                422 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Validation failed".to_string());
                    ApiError::InvalidRequest { message }
                }
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Merge a pull request.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn merge(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        request: MergePullRequestRequest,
    ) -> Result<MergeResult, ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}/merge", owner, repo, pull_number);
        let response = self.client.put(&path, &request).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                405 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Pull request not mergeable".to_string());
                    ApiError::HttpError {
                        status: 405,
                        message,
                    }
                }
                409 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Merge conflict".to_string());
                    ApiError::HttpError {
                        status: 409,
                        message,
                    }
                }
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Set the milestone on a pull request.
    ///
    /// The GitHub Pulls API silently ignores the milestone field, so this method
    /// delegates to the Issues API (PATCH /repos/{owner}/{repo}/issues/{number})
    /// which correctly applies the milestone, then re-fetches the PR to return
    /// the updated state.
    ///
    /// # Partial-failure note
    ///
    /// If the Issues API call succeeds but the subsequent `get()` call fails
    /// (e.g. due to a transient network error), this method returns an error even
    /// though the milestone **was** actually set on the PR. Callers should treat
    /// an error response as "milestone state unknown" rather than "milestone not
    /// set" and may wish to re-fetch the PR to confirm the current state.
    ///
    /// See docs/specs/interfaces/pull-request-operations.md
    pub async fn set_milestone(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        milestone_number: Option<u64>,
    ) -> Result<PullRequest, ApiError> {
        self.client
            .issues()
            .set_milestone(owner, repo, pull_number, milestone_number)
            .await?;
        self.get(owner, repo, pull_number).await
    }

    // ========================================================================
    // Pull Request Review Operations
    // ========================================================================

    /// List reviews on a pull request.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn list_reviews(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<Vec<Review>, ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}/reviews", owner, repo, pull_number);
        let response = self.client.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Get a specific review by ID.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn get_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
    ) -> Result<Review, ApiError> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/reviews/{}",
            owner, repo, pull_number, review_id
        );
        let response = self.client.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Create a review on a pull request.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn create_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        request: CreateReviewRequest,
    ) -> Result<Review, ApiError> {
        let path = format!("/repos/{}/{}/pulls/{}/reviews", owner, repo, pull_number);
        let response = self.client.post(&path, &request).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                422 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Validation failed".to_string());
                    ApiError::InvalidRequest { message }
                }
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Update a pending review.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn update_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
        request: UpdateReviewRequest,
    ) -> Result<Review, ApiError> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/reviews/{}",
            owner, repo, pull_number, review_id
        );
        let response = self.client.put(&path, &request).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                422 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Validation failed".to_string());
                    ApiError::InvalidRequest { message }
                }
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Dismiss a review.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    pub async fn dismiss_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
        request: DismissReviewRequest,
    ) -> Result<Review, ApiError> {
        let path = format!(
            "/repos/{}/{}/pulls/{}/reviews/{}/dismissals",
            owner, repo, pull_number, review_id
        );
        let response = self.client.put(&path, &request).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                422 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Validation failed".to_string());
                    ApiError::InvalidRequest { message }
                }
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
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
            });
        }
        response.json().await.map_err(ApiError::from)
    }

    // ========================================================================
    // Pull Request Comment Operations
    // ========================================================================

    /// List all conversation-thread comments on a pull request (auto-paginated).
    ///
    /// Uses the Issues comments endpoint per GitHub API design.
    ///
    /// See docs/specs/interfaces/pull-request-operations.md
    pub async fn list_comments(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<Vec<Comment>, ApiError> {
        let first_page = format!(
            "/repos/{}/{}/issues/{}/comments?per_page=100",
            owner, repo, pull_number
        );
        self.client.fetch_all_pages(&first_page).await
    }

    /// Add a conversation-thread comment to a pull request.
    ///
    /// Uses the Issues comments endpoint per GitHub API design.
    ///
    /// See docs/specs/interfaces/pull-request-operations.md
    pub async fn create_comment(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        request: CreatePullRequestCommentRequest,
    ) -> Result<Comment, ApiError> {
        let path = format!("/repos/{}/{}/issues/{}/comments", owner, repo, pull_number);
        let response = self.client.post(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Update an existing pull request conversation-thread comment.
    ///
    /// See docs/specs/interfaces/pull-request-operations.md
    pub async fn update_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        request: UpdatePullRequestCommentRequest,
    ) -> Result<Comment, ApiError> {
        let path = format!("/repos/{}/{}/issues/comments/{}", owner, repo, comment_id);
        let response = self.client.patch(&path, &request).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Delete a pull request conversation-thread comment.
    ///
    /// See docs/specs/interfaces/pull-request-operations.md
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
            return Err(super::map_http_error(status, response).await);
        }
        Ok(())
    }

    // ========================================================================
    // Pull Request Label Operations
    // ========================================================================

    /// Add labels to a pull request.
    ///
    /// See docs/specs/interfaces/pull-request-operations.md
    pub async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        labels: Vec<String>,
    ) -> Result<Vec<Label>, ApiError> {
        // PRs use the same label endpoint as issues
        let path = format!("/repos/{}/{}/issues/{}/labels", owner, repo, pull_number);
        let body = LabelsRequest { labels };
        let response = self.client.post(&path, &body).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Replace all labels on a pull request.
    ///
    /// Replaces the entire set of labels. Pass an empty vec to clear all labels.
    ///
    /// See docs/specs/interfaces/pull-request-operations.md
    pub async fn replace_labels(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        labels: Vec<String>,
    ) -> Result<Vec<Label>, ApiError> {
        // PRs use the same label endpoint as issues
        let path = format!("/repos/{}/{}/issues/{}/labels", owner, repo, pull_number);
        let body = LabelsRequest { labels };
        let response = self.client.put(&path, &body).await?;
        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }

    /// Remove a label from a pull request.
    ///
    /// See docs/spec/interfaces/pull-request-operations.md
    ///
    /// # Error mapping
    ///
    /// GitHub returns HTTP 422 when the label name is unprocessable (e.g. does
    /// not exist on the repository).  This method maps that to
    /// [`ApiError::InvalidRequest`], which is the correct semantic mapping and
    /// is consistent with how other label methods in this file behave.  Callers
    /// that previously matched on `ApiError::HttpError { status: 422, .. }`
    /// must be updated to match `ApiError::InvalidRequest { .. }` instead.
    pub async fn remove_label(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        name: &str,
    ) -> Result<Vec<Label>, ApiError> {
        // PRs use the same label endpoint as issues
        let path = format!(
            "/repos/{}/{}/issues/{}/labels/{}",
            owner,
            repo,
            pull_number,
            urlencoding::encode(name)
        );
        let response = self.client.delete(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }
        response.json().await.map_err(ApiError::from)
    }
}

#[cfg(test)]
#[path = "pull_request_tests.rs"]
mod tests;
