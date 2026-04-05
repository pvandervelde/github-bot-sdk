# PullRequestsClient Interface Specification

**Module**: `github-bot-sdk::client::pull_request`
**Struct**: `PullRequestsClient`
**Obtained via**: `InstallationClient::pull_requests()`
**Source file**: `src/client/pull_request.rs`

See **ADR-003** for the sub-client pattern rationale.

## Overview

`PullRequestsClient` provides PR management, review management, inline comments,
label application, and merge operations scoped to pull requests.

## Sub-Client Type

```rust
/// Domain client for pull request operations.
///
/// Obtained via `InstallationClient::pull_requests()`. Cheap to clone (Arc-backed).
#[derive(Debug, Clone)]
pub struct PullRequestsClient {
    // Internal representation chosen by interface designer
}
```

## Permissions

| Operation group | Minimum permission |
|----------------|--------------------|
| `list`, `get` | `pull_requests: read` |
| `create`, `update`, `merge` | `pull_requests: write` |
| Reviews | `pull_requests: write` |
| Comments | `pull_requests: write` (write) / `pull_requests: read` (list) |

## Core Types

### PullRequest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: PullRequestState,
    pub user: User,
    pub head: PullRequestBranch,
    pub base: PullRequestBranch,
    pub draft: bool,
    pub merged: bool,
    pub mergeable: Option<bool>,
    pub labels: Vec<Label>,
    pub html_url: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub merged_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub closed_at: Option<OffsetDateTime>,
}
```

### PullRequestState

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PullRequestState {
    Open,
    Closed,
}
```

### PullRequestBranch

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestBranch {
    pub label: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
    pub repo: Option<PullRequestRepo>,
}
```

**Note**: Uses the shared `Commit` type from repository operations for commit references.

### PullRequestRepo

Repository information for pull request branches.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
}
```

### Review

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: u64,
    pub user: User,
    pub body: Option<String>,
    pub state: ReviewState,
    pub html_url: String,
    #[serde(with = "time::serde::rfc3339")]
    pub submitted_at: OffsetDateTime,
}
```

### ReviewState

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
}
```

## Pull Request Operations

### `get`

```rust
impl PullRequestsClient {
    /// Get a specific pull request by number.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — PR doesn't exist
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<PullRequest, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/pulls/{pull_number}`

### `list`

Returns the first page of pull requests matching the filter criteria (manual pagination).

```rust
impl PullRequestsClient {
    /// List pull requests in a repository.
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
        params: Option<&ListPullRequestsParams>,
    ) -> Result<PagedResponse<PullRequest>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/pulls`

### `create`

```rust
impl PullRequestsClient {
    /// Create a new pull request.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` — Missing `pull_requests: write`
    /// * `ApiError::InvalidRequest` — Invalid branch or no commits (422)
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: &CreatePullRequestRequest,
    ) -> Result<PullRequest, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/pulls`

### `update`

```rust
impl PullRequestsClient {
    /// Update an existing pull request.
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        request: &UpdatePullRequestRequest,
    ) -> Result<PullRequest, ApiError>;
}
```

**Endpoint**: `PATCH /repos/{owner}/{repo}/pulls/{pull_number}`

### `merge`

```rust
impl PullRequestsClient {
    /// Merge a pull request.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` — Missing merge permission
    /// * `ApiError::HttpError { status: 405 }` — Not mergeable
    /// * `ApiError::HttpError { status: 409 }` — Merge conflict
    pub async fn merge(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        request: Option<&MergePullRequestRequest>,
    ) -> Result<MergeResult, ApiError>;
}
```

**Endpoint**: `PUT /repos/{owner}/{repo}/pulls/{pull_number}/merge`

### `set_milestone`

```rust
impl PullRequestsClient {
    /// Set (or clear) the milestone on a pull request.
    ///
    /// Pass `None` to remove the milestone from the PR.
    pub async fn set_milestone(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        milestone_number: Option<u64>,
    ) -> Result<PullRequest, ApiError>;
}
```

**Implementation**: Delegates to `update()` with the `milestone` field set.

## Review Operations

Review methods use the `list_reviews` / `get_review` / etc. naming style (not prefixed
with `pull_request_`) because the sub-client context makes the domain clear.

### `list_reviews`

```rust
impl PullRequestsClient {
    /// List reviews for a pull request in chronological order.
    pub async fn list_reviews(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<Vec<Review>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/pulls/{pull_number}/reviews`

### `get_review`

```rust
impl PullRequestsClient {
    /// Get a single review by ID.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — Review doesn't exist on this PR
    pub async fn get_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
    ) -> Result<Review, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}`

### `create_review`

```rust
impl PullRequestsClient {
    /// Create a review for a pull request.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` — Missing `pull_requests: write`
    /// * `ApiError::InvalidRequest` — Already reviewed (422)
    pub async fn create_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        request: &CreateReviewRequest,
    ) -> Result<Review, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews`

### `update_review`

```rust
impl PullRequestsClient {
    /// Update the body of an existing pending review.
    pub async fn update_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
        body: &str,
    ) -> Result<Review, ApiError>;
}
```

**Endpoint**: `PUT /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}`

### `dismiss_review`

```rust
impl PullRequestsClient {
    /// Dismiss a submitted review.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` — Only maintainers can dismiss reviews
    pub async fn dismiss_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
        message: &str,
    ) -> Result<Review, ApiError>;
}
```

**Endpoint**: `PUT /repos/{owner}/{repo}/pulls/{pull_number}/reviews/{review_id}/dismissals`

## Comment Operations

Pull requests support issue-style comments (on the conversation thread), separate from
review comments (inline code comments attached to a file diff).

### `list_comments`

```rust
impl PullRequestsClient {
    /// List all conversation-thread comments on a pull request.
    ///
    /// These are issue-body-style comments. For review comments (inline code
    /// annotations), use `list_reviews`.
    ///
    /// Auto-paginates (ADR-002).
    pub async fn list_comments(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<Vec<Comment>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/{pull_number}/comments?per_page=100`

*Note*: GitHub routes PR conversation comments through the Issues comments endpoint.

### `create_comment`

```rust
impl PullRequestsClient {
    /// Add a conversation-thread comment to a pull request.
    pub async fn create_comment(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        body: &str,
    ) -> Result<Comment, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/issues/{pull_number}/comments`

### `update_comment`

```rust
impl PullRequestsClient {
    /// Update the body of a conversation-thread comment.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — comment doesn't exist
    /// * `ApiError::AuthorizationFailed` — not the comment author
    pub async fn update_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<Comment, ApiError>;
}
```

**Endpoint**: `PATCH /repos/{owner}/{repo}/issues/comments/{comment_id}`

### `delete_comment`

```rust
impl PullRequestsClient {
    /// Delete a conversation-thread comment.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — comment doesn't exist
    /// * `ApiError::AuthorizationFailed` — not the comment author
    pub async fn delete_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/issues/comments/{comment_id}`
**Success**: 204 No Content

## Label Operations

### `add_labels`

```rust
impl PullRequestsClient {
    /// Add labels to a pull request.
    ///
    /// Labels must already exist in the repository label catalogue (`LabelsClient`).
    ///
    /// # Returns
    ///
    /// Returns the updated set of labels on the PR.
    pub async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        labels: &[String],
    ) -> Result<Vec<Label>, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/issues/{pull_number}/labels`

### `remove_label`

```rust
impl PullRequestsClient {
    /// Remove a single label from a pull request.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — label not applied to this PR
    pub async fn remove_label(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        label_name: &str,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/issues/{pull_number}/labels/{label_name}`

## Request Types

### CreatePullRequestRequest

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CreatePullRequestRequest {
    pub title: String,
    pub head: String,
    pub base: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<u64>,
    /// Whether maintainers of the base repository can push to the head branch.
    /// Defaults to `true` on the GitHub API for fork-sourced pull requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer_can_modify: Option<bool>,
}
```

### UpdatePullRequestRequest

```rust
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdatePullRequestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<PullRequestState>,
}
```

### MergePullRequestRequest

```rust
#[derive(Debug, Clone, Default, Serialize)]
pub struct MergePullRequestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_method: Option<MergeMethod>,
}
```

### MergeMethod

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}
```

### MergeResult

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct MergeResult {
    pub sha: String,
    pub merged: bool,
    pub message: String,
}
```

### CreateReviewRequest

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CreateReviewRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub event: ReviewEvent,
}
```

### ReviewEvent

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewEvent {
    Approve,
    RequestChanges,
    Comment,
}
```

### ListPullRequestsParams

```rust
#[derive(Debug, Clone, Default)]
pub struct ListPullRequestsParams {
    pub state: Option<PullRequestState>,
    pub head: Option<String>,
    pub base: Option<String>,
}
```

## Usage Examples

### Create a Pull Request

```rust
let request = CreatePullRequestRequest {
    title: "Add new feature".to_string(),
    head: "feature-branch".to_string(),
    base: "main".to_string(),
    body: Some("Description of changes".to_string()),
    draft: Some(false),
};

let pr = client.create_pull_request("owner", "repo", &request).await?;
println!("Created PR #{}", pr.number);
```

### Approve a Pull Request

```rust
let review = CreateReviewRequest {
    body: Some("LGTM!".to_string()),
    event: ReviewEvent::Approve,
};

client.create_pull_request_review("owner", "repo", 42, &review).await?;
```

### Merge a Pull Request

```rust
let merge_opts = MergePullRequestRequest {
    commit_title: Some("Merge feature".to_string()),
    merge_method: Some(MergeMethod::Squash),
    ..Default::default()
};

let result = client.merge_pull_request("owner", "repo", 42, Some(&merge_opts)).await?;
println!("Merged: {}", result.sha);
```

## Implementation Notes

### API Paths

- Pull requests: `/repos/{owner}/{repo}/pulls`
- Pull request: `/repos/{owner}/{repo}/pulls/{pull_number}`
- Merge: `/repos/{owner}/{repo}/pulls/{pull_number}/merge`
- Reviews: `/repos/{owner}/{repo}/pulls/{pull_number}/reviews`

### Merge Conflicts

When merge fails due to conflicts:

- Returns `ApiError::HttpError` with status 409
- Message indicates conflicts exist

### Testing Strategy

- Mock all HTTP responses
- Test merge method variations
- Test review state transitions
- Verify error handling for conflicts

## References

- GitHub API: [Pull Requests](https://docs.github.com/en/rest/pulls/pulls)
- GitHub API: [Reviews](https://docs.github.com/en/rest/pulls/reviews)
