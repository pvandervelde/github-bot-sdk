# IssuesClient Interface Specification

**Module**: `github-bot-sdk::client::issue`
**Struct**: `IssuesClient`
**Obtained via**: `InstallationClient::issues()`
**Source file**: `src/client/issue.rs`
**Dependencies**: `InstallationClient`, `ApiError`, shared types

See **ADR-003** for the sub-client pattern rationale.

## Overview

`IssuesClient` groups all operations that act on GitHub issues or objects that are
directly attached to an issue (comments, reactions, labels on an issue, assignees,
locks, timeline, activity events). It does **not** manage the repository-level label
catalogue (that is `LabelsClient`) or milestone definitions (that is `MilestonesClient`).

## Sub-Client Type

```rust
/// Domain client for GitHub issue operations.
///
/// Obtained via `InstallationClient::issues()`. Cheap to clone (Arc-backed).
#[derive(Debug, Clone)]
pub struct IssuesClient {
    // Internal representation chosen by interface designer (e.g. Arc<InstallationClient>)
}
```

## Core Types

### Issue

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub node_id: String,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,           // "open" | "closed"
    pub locked: bool,
    pub user: IssueUser,
    pub assignees: Vec<IssueUser>,
    pub labels: Vec<Label>,
    pub milestone: Option<Milestone>,
    pub comments: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub html_url: String,
}
```

### Comment

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub node_id: String,
    pub body: String,
    pub user: IssueUser,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub html_url: String,
}
```

### IssueUser

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueUser {
    pub login: String,
    pub id: u64,
    pub node_id: String,
    #[serde(rename = "type")]
    pub user_type: String,
}
```

### Label

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub default: bool,
}
```

### Milestone

See `MilestonesClient` spec for the full type definition. `Issue` embeds a
`Milestone` by value as returned by GitHub on issue responses.

### LockReason

```rust
/// Reason used when locking an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockReason {
    OffTopic,
    TooHeated,
    Resolved,
    Spam,
}
```

### IssueActivityEvent

```rust
/// A discrete activity recorded on an issue (labeling, closing, etc.).
///
/// Returned by the issue events REST endpoint — different from webhook events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueActivityEvent {
    pub id: u64,
    pub event: String,            // "labeled", "assigned", "closed", etc.
    pub actor: IssueUser,
    pub created_at: DateTime<Utc>,
    pub label: Option<Label>,
    pub assignee: Option<IssueUser>,
    pub milestone: Option<MilestoneSummary>,
    pub rename: Option<IssueRename>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneSummary {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRename {
    pub from: String,
    pub to: String,
}
```

### TimelineEvent

```rust
/// A single item in an issue's timeline.
///
/// The `event` field drives deserialization. Unknown kinds map to `Unknown`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum TimelineEvent {
    Commented    { id: u64, actor: IssueUser, body: String,
                   created_at: DateTime<Utc>, updated_at: DateTime<Utc>,
                   html_url: String },
    Labeled      { id: u64, actor: IssueUser, label: Label,
                   created_at: DateTime<Utc> },
    Unlabeled    { id: u64, actor: IssueUser, label: Label,
                   created_at: DateTime<Utc> },
    Assigned     { id: u64, actor: IssueUser, assignee: IssueUser,
                   created_at: DateTime<Utc> },
    Unassigned   { id: u64, actor: IssueUser, assignee: IssueUser,
                   created_at: DateTime<Utc> },
    Milestoned   { id: u64, actor: IssueUser, milestone: MilestoneSummary,
                   created_at: DateTime<Utc> },
    Demilestoned { id: u64, actor: IssueUser, milestone: MilestoneSummary,
                   created_at: DateTime<Utc> },
    Closed       { id: u64, actor: IssueUser, created_at: DateTime<Utc> },
    Reopened     { id: u64, actor: IssueUser, created_at: DateTime<Utc> },
    Locked       { id: u64, actor: IssueUser, lock_reason: Option<String>,
                   created_at: DateTime<Utc> },
    Unlocked     { id: u64, actor: IssueUser, created_at: DateTime<Utc> },
    Renamed      { id: u64, actor: IssueUser, rename: IssueRename,
                   created_at: DateTime<Utc> },
    Referenced   { id: u64, actor: IssueUser, created_at: DateTime<Utc> },
    /// Catch-all: unknown event kind. Must not cause a deserialization error.
    #[serde(other)]
    Unknown,
}
```

**Note**: `#[serde(other)]` on a unit variant is valid for `tag`-enums in serde when
the fallback variant is a unit variant. Unknown fields are silently discarded. If the
interface designer determines a different deserialization strategy is needed (e.g. to
preserve the raw JSON), they may use a custom `Deserialize` impl instead — this spec
only requires that unknown kinds produce no `Err`.

---

## Request Types

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CreateIssueRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,     // "open" | "closed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateCommentRequest {
    pub body: String,
}
```

---

## Issue CRUD Operations

### `list`

```rust
impl IssuesClient {
    /// List issues in a repository (manual pagination — callers may stop early).
    ///
    /// # Arguments
    /// * `owner` - Repository owner login
    /// * `repo` - Repository name
    /// * `state` - Filter: `"open"` (default), `"closed"`, or `"all"`
    /// * `page` - Page number (1-indexed); omit for first page
    ///
    /// # Returns
    /// `PagedResponse<Issue>` — use `.has_next()` and `.next_page_number()` to paginate.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — repository doesn't exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: read`
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
        page: Option<u32>,
    ) -> Result<PagedResponse<Issue>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues?state={state}&page={page}`

### `get`

```rust
impl IssuesClient {
    /// Get a single issue by number.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — issue doesn't exist
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Issue, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/{issue_number}`

### `create`

```rust
impl IssuesClient {
    /// Create a new issue.
    ///
    /// # Errors
    /// * `ApiError::InvalidRequest` — validation failed (empty title, etc.)
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: CreateIssueRequest,
    ) -> Result<Issue, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/issues`

### `update`

```rust
impl IssuesClient {
    /// Update an existing issue. All fields are optional (patch semantics).
    ///
    /// # Errors
    /// * `ApiError::NotFound` — issue doesn't exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        request: UpdateIssueRequest,
    ) -> Result<Issue, ApiError>;
}
```

**Endpoint**: `PATCH /repos/{owner}/{repo}/issues/{issue_number}`

### `set_milestone`

```rust
impl IssuesClient {
    /// Set (or clear) the milestone on an issue.
    ///
    /// Implemented as `update` with only the `milestone` field populated.
    ///
    /// # Arguments
    /// * `milestone_number` — milestone number, or `None` to remove the milestone.
    pub async fn set_milestone(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        milestone_number: Option<u64>,
    ) -> Result<Issue, ApiError>;
}
```

---

## Comment Operations

All comments are returned in ascending `created_at` order (oldest first — GitHub default).

### `list_comments` ⚡ *auto-paginated*

```rust
impl IssuesClient {
    /// List all comments on an issue.
    ///
    /// Auto-paginates using `per_page=100` and follows all `Link: rel="next"`
    /// headers (ADR-002). Returns the complete list; callers do not need to
    /// handle pagination themselves.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — issue doesn't exist (returns immediately, no partial list)
    pub async fn list_comments(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<Comment>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/{issue_number}/comments?per_page=100`

### `get_comment`

```rust
impl IssuesClient {
    /// Get a single comment by its ID.
    ///
    /// Note: comment IDs are repository-scoped, not issue-scoped.
    pub async fn get_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<Comment, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/comments/{comment_id}`

### `create_comment`

```rust
impl IssuesClient {
    /// Add a new comment to an issue.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — issue doesn't exist
    /// * `ApiError::InvalidRequest` — empty body
    /// * `ApiError::AuthorizationFailed` — missing `issues: write` or issue is locked
    pub async fn create_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        request: CreateCommentRequest,
    ) -> Result<Comment, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/issues/{issue_number}/comments`

### `update_comment`

```rust
impl IssuesClient {
    /// Update the body of an existing comment.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — comment doesn't exist
    /// * `ApiError::AuthorizationFailed` — not the comment author or missing permission
    pub async fn update_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        request: UpdateCommentRequest,
    ) -> Result<Comment, ApiError>;
}
```

**Endpoint**: `PATCH /repos/{owner}/{repo}/issues/comments/{comment_id}`

### `delete_comment`

```rust
impl IssuesClient {
    /// Delete a comment.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — comment doesn't exist
    /// * `ApiError::AuthorizationFailed` — not the comment author or missing permission
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

---

## Label Application Operations

These act on labels **attached to a specific issue**. For repository-level label catalogue
management (creating/updating label definitions), use `LabelsClient`.

### `list_labels`

```rust
impl IssuesClient {
    /// List labels currently applied to an issue.
    ///
    /// Auto-paginates using `per_page=100` (ADR-002). The label set per issue
    /// is bounded; returning all at once is appropriate.
    pub async fn list_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<Label>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/{issue_number}/labels?per_page=100`

### `add_labels`

```rust
impl IssuesClient {
    /// Add one or more labels to an issue.
    ///
    /// Labels already on the issue are ignored by GitHub (idempotent).
    ///
    /// # Returns
    /// The updated label list on the issue.
    pub async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        labels: Vec<String>,
    ) -> Result<Vec<Label>, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/issues/{issue_number}/labels`
**Body**: `{ "labels": ["bug", "help-wanted"] }`

### `remove_label`

```rust
impl IssuesClient {
    /// Remove a single label from an issue.
    ///
    /// # Returns
    /// The remaining label list.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — issue or label doesn't exist on the issue
    pub async fn remove_label(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        label_name: &str,
    ) -> Result<Vec<Label>, ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/issues/{issue_number}/labels/{label_name}`

### `replace_labels` (atomic)

```rust
impl IssuesClient {
    /// Replace all labels on an issue atomically.
    ///
    /// Any labels not in `labels` are removed. Labels in `labels` that do not exist
    /// in the repository are created automatically by GitHub.
    /// Pass an empty slice to remove all labels.
    ///
    /// # Returns
    /// The new label list as applied by GitHub.
    pub async fn replace_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        labels: Vec<String>,
    ) -> Result<Vec<Label>, ApiError>;
}
```

**Endpoint**: `PUT /repos/{owner}/{repo}/issues/{issue_number}/labels`
**Body**: `{ "labels": ["enhancement"] }`

---

## Reaction Operations

### `list_reactions` ⚡ *auto-paginated*

```rust
impl IssuesClient {
    /// List all reactions on an issue.
    pub async fn list_reactions(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<Reaction>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/{issue_number}/reactions?per_page=100`

### `create_reaction`

```rust
impl IssuesClient {
    /// Add a reaction to an issue.
    ///
    /// If this user has already reacted with this emoji, GitHub returns the
    /// existing reaction (HTTP 200). Both 200 and 201 are mapped to `Ok(Reaction)`.
    pub async fn create_reaction(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        content: ReactionContent,
    ) -> Result<Reaction, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/issues/{issue_number}/reactions`

### `delete_reaction`

```rust
impl IssuesClient {
    /// Remove a reaction from an issue.
    pub async fn delete_reaction(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        reaction_id: u64,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/issues/{issue_number}/reactions/{reaction_id}`

### `list_comment_reactions` ⚡ *auto-paginated*

```rust
impl IssuesClient {
    /// List all reactions on an issue comment.
    pub async fn list_comment_reactions(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<Vec<Reaction>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/comments/{comment_id}/reactions?per_page=100`

### `create_comment_reaction`

```rust
impl IssuesClient {
    /// Add a reaction to an issue comment.
    pub async fn create_comment_reaction(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        content: ReactionContent,
    ) -> Result<Reaction, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/issues/comments/{comment_id}/reactions`

### `delete_comment_reaction`

```rust
impl IssuesClient {
    /// Remove a reaction from an issue comment.
    pub async fn delete_comment_reaction(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        reaction_id: u64,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/issues/comments/{comment_id}/reactions/{reaction_id}`

---

## Assignee Operations

### `list_available_assignees` ⚡ *auto-paginated*

```rust
impl IssuesClient {
    /// List users who can be assigned to issues in a repository.
    ///
    /// Auto-paginates (ADR-002).
    pub async fn list_available_assignees(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<IssueUser>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/assignees?per_page=100`

### `add_assignees`

```rust
impl IssuesClient {
    /// Add assignees to an issue.
    ///
    /// GitHub silently ignores users who are not eligible to be assigned.
    ///
    /// # Returns
    /// Updated `Issue` with the new assignee list.
    pub async fn add_assignees(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        assignees: Vec<String>,
    ) -> Result<Issue, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/issues/{issue_number}/assignees`
**Body**: `{ "assignees": ["alice", "bob"] }`

### `remove_assignees`

```rust
impl IssuesClient {
    /// Remove assignees from an issue.
    ///
    /// GitHub silently ignores users not currently assigned.
    ///
    /// # Returns
    /// Updated `Issue` with the revised assignee list.
    pub async fn remove_assignees(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        assignees: Vec<String>,
    ) -> Result<Issue, ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/issues/{issue_number}/assignees`
**Body**: `{ "assignees": ["alice"] }` — GitHub requires a body on this DELETE.

---

## Lock Operations

### `lock`

```rust
impl IssuesClient {
    /// Lock an issue, preventing non-collaborator comments.
    ///
    /// # Arguments
    /// * `reason` — optional display reason shown to users attempting to comment
    ///
    /// # Errors
    /// * `ApiError::AuthorizationFailed` — requires admin or maintain permission
    pub async fn lock(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        reason: Option<LockReason>,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `PUT /repos/{owner}/{repo}/issues/{issue_number}/lock`
**Body**: `{ "lock_reason": "too-heated" }` (field omitted when `reason` is `None`)
**Success**: 204 No Content

### `unlock`

```rust
impl IssuesClient {
    /// Unlock a previously locked issue.
    ///
    /// # Errors
    /// * `ApiError::AuthorizationFailed` — requires admin or maintain permission
    pub async fn unlock(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/issues/{issue_number}/lock`
**Success**: 204 No Content

---

## Activity & Timeline Operations

### `list_activity_events` ⚡ *auto-paginated*

```rust
impl IssuesClient {
    /// List all discrete activity events recorded on an issue.
    ///
    /// Returns events in ascending chronological order (oldest first).
    /// Auto-paginates (ADR-002).
    ///
    /// # Examples of event kinds
    /// `labeled`, `unlabeled`, `assigned`, `unassigned`, `milestoned`,
    /// `demilestoned`, `closed`, `reopened`, `renamed`, `locked`, `unlocked`.
    pub async fn list_activity_events(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<IssueActivityEvent>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/{issue_number}/events?per_page=100`

### `list_timeline` ⚡ *auto-paginated*

```rust
impl IssuesClient {
    /// List the complete timeline of events for an issue.
    ///
    /// Superset of `list_activity_events`: also includes comments,
    /// cross-references, and review events. Returns a heterogeneous
    /// sequence; unknown event kinds deserialize to `TimelineEvent::Unknown`
    /// without error. Auto-paginates (ADR-002).
    pub async fn list_timeline(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<TimelineEvent>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/{issue_number}/timeline?per_page=100`

---

## Permissions Summary

| Method | Minimum permission |
|--------|--------------------|
| `list`, `get` | `issues: read` |
| `list_comments`, `get_comment` | `issues: read` |
| `list_labels`, `list_reactions`, `list_comment_reactions` | `issues: read` |
| `list_available_assignees` | `issues: read` |
| `list_activity_events`, `list_timeline` | `issues: read` |
| `create`, `update`, `set_milestone` | `issues: write` |
| `create_comment`, `update_comment`, `delete_comment` | `issues: write` |
| `add_labels`, `remove_label`, `replace_labels` | `issues: write` |
| `create_reaction`, `delete_reaction` | `issues: write` |
| `create_comment_reaction`, `delete_comment_reaction` | `issues: write` |
| `add_assignees`, `remove_assignees` | `issues: write` |
| `lock`, `unlock` | admin or maintain |

## Error Mapping (standard across all methods)

| HTTP status | `ApiError` variant |
|-------------|-------------------|
| 401 | `AuthenticationFailed` |
| 403 | `AuthorizationFailed` |
| 404 | `NotFound` |
| 422 | `InvalidRequest { message }` |
| other 4xx/5xx | `HttpError { status, message }` |
| deserialization | `HttpError` (wraps serde error) |

## Testing Requirements

Tests live in `src/client/issue_tests.rs` using `wiremock`. Submodules:

- `construction` — struct construction, serialization of request types
- `issue_operations` — list/get/create/update with mock HTTP responses
- `comment_operations` — full CRUD + auto-pagination (empty, 1-page, 2-page, 404)
- `label_operations` — add, remove, replace, list on issue
- `reaction_operations` — all six reaction methods; duplicate-reaction (200 vs 201)
- `assignee_operations` — add, remove, list-available
- `lock_operations` — lock with/without reason, unlock
- `activity_event_operations` — auto-pagination, empty list
- `timeline_operations` — mixed event types, unknown kind → `Unknown`, empty list
