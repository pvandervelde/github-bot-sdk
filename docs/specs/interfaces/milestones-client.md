# MilestonesClient Interface Specification

**Module**: `github-bot-sdk::client::issue`
**Struct**: `MilestonesClient`
**Obtained via**: `InstallationClient::milestones()`
**Source file**: `src/client/issue.rs`

See **ADR-003** for the sub-client pattern rationale.

## Overview

`MilestonesClient` manages repository milestones — named progress markers that group
issues and pull requests. Milestones are repository-level entities with their own
CRUD lifecycle.

**Scope**: `MilestonesClient` owns milestone definitions. Assigning a milestone to a
specific issue is done via `IssuesClient::set_milestone`. Assigning to a PR is done via
`PullRequestsClient::set_milestone`.

## Sub-Client Type

```rust
/// Domain client for milestone lifecycle operations.
///
/// Obtained via `InstallationClient::milestones()`. Cheap to clone (Arc-backed).
#[derive(Debug, Clone)]
pub struct MilestonesClient {
    // Internal representation chosen by interface designer
}
```

## Types

### `Milestone`

Reused from the shared type in `issue.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: u64,
    pub node_id: String,
    pub number: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: MilestoneState,
    pub due_on: Option<DateTime<Utc>>,
    pub open_issues: u32,
    pub closed_issues: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}
```

### `MilestoneState`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MilestoneState {
    Open,
    Closed,
}
```

### Request Types

```rust
/// Request to create a new milestone.
#[derive(Debug, Clone, Serialize)]
pub struct CreateMilestoneRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<MilestoneState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// ISO 8601 format (GitHub expects this)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<DateTime<Utc>>,
}

/// Request to update an existing milestone.
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

/// Filter and sort options for listing milestones.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListMilestonesQuery {
    /// Filter by state; default `open`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<MilestoneState>,
    /// Sort by `due_on` or `completeness`; default `due_on`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<MilestoneSortField>,
    /// Sort direction; default `asc`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<SortDirection>,
}
```

### `MilestoneSortField`

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MilestoneSortField {
    DueOn,
    Completeness,
}
```

### `SortDirection`

```rust
/// Sort direction for list queries.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}
```

This type is shared across domain clients. The interface designer may hoist it to a
common module (e.g. `src/client/mod.rs`) so it can be reused by `list_commits`,
`list_milestones`, and any future sorted-list operations without duplication.

## Operations

### `list` ⚡ *auto-paginated*

```rust
impl MilestonesClient {
    /// List all milestones in a repository.
    ///
    /// Auto-paginates using `per_page=100` (ADR-002).
    ///
    /// # Errors
    /// * `ApiError::NotFound` — repository doesn't exist
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
        query: Option<ListMilestonesQuery>,
    ) -> Result<Vec<Milestone>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/milestones?per_page=100[&state=..][&sort=..][&direction=..]`

### `get`

```rust
impl MilestonesClient {
    /// Get a single milestone by its number.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — milestone doesn't exist
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        milestone_number: u64,
    ) -> Result<Milestone, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/milestones/{milestone_number}`

### `create`

```rust
impl MilestonesClient {
    /// Create a new milestone.
    ///
    /// # Errors
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    /// * `ApiError::InvalidRequest` — title is empty
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: CreateMilestoneRequest,
    ) -> Result<Milestone, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/milestones`

### `update`

```rust
impl MilestonesClient {
    /// Update an existing milestone.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — milestone doesn't exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        milestone_number: u64,
        request: UpdateMilestoneRequest,
    ) -> Result<Milestone, ApiError>;
}
```

**Endpoint**: `PATCH /repos/{owner}/{repo}/milestones/{milestone_number}`

### `delete`

```rust
impl MilestonesClient {
    /// Delete a milestone.
    ///
    /// Issues assigned to the deleted milestone are unlinked but otherwise
    /// unaffected. This operation cannot be undone.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — milestone doesn't exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn delete(
        &self,
        owner: &str,
        repo: &str,
        milestone_number: u64,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/milestones/{milestone_number}`
**Success**: 204 No Content

## Permissions

| Method | Minimum permission |
|--------|--------------------|
| `list`, `get` | `issues: read` |
| `create`, `update`, `delete` | `issues: write` |

## Error Mapping

| HTTP status | `ApiError` variant |
|-------------|-------------------|
| 401 | `AuthenticationFailed` |
| 403 | `AuthorizationFailed` |
| 404 | `NotFound` |
| 422 | `InvalidRequest { message }` |
| other | `HttpError { status, message }` |

## Edge Cases

- **Deleting a milestone with open issues**: Issues are unlinked (milestone set to `null`)
  but remain open. Callers that rely on milestone membership for triage should query
  issues before deleting.
- **Past-due milestones**: `due_on` in the past is accepted; GitHub does not reject it.
  The `state` field must be set to `closed` explicitly to close a past-due milestone.
- **Closed milestone listing**: `list` with `state: closed` returns milestones that no
  longer actively track work; useful for audit/history use cases.

## Testing Requirements

Tests live in `src/client/issue_tests.rs` in a `milestone_operations` submodule.

| Scenario | Expected |
|----------|---------|
| `list` — empty milestones | `Ok(vec![])` |
| `list` — with `state: closed` | Returns only closed milestones |
| `list` — multi-page | Auto-paginates, returns all |
| `get` — existing milestone | `Ok(Milestone)` with correct fields |
| `get` — not found | `Err(ApiError::NotFound)` |
| `create` — success | `Ok(Milestone)` with number assigned |
| `create` — with `due_on` | `Ok(Milestone)` with `due_on` set |
| `update` — close milestone | `Ok(Milestone)` with `state: closed` |
| `update` — not found | `Err(ApiError::NotFound)` |
| `delete` — success | `Ok(())` |
| `delete` — not found | `Err(ApiError::NotFound)` |
