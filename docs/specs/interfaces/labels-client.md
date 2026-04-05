# LabelsClient Interface Specification

**Module**: `github-bot-sdk::client::issue`
**Struct**: `LabelsClient`
**Obtained via**: `InstallationClient::labels()`
**Source file**: `src/client/issue.rs`

See **ADR-003** for the sub-client pattern rationale.

## Overview

`LabelsClient` manages the **repository-level label catalogue** — the set of label
definitions that exist in a repository. These are the labels that can be applied to
issues and pull requests.

**Scope**: `LabelsClient` operates only on label *definitions*. Applying labels to a
specific issue is done via `IssuesClient`. Applying labels to a PR is done via
`PullRequestsClient`.

This split reflects GitHub's own API structure:

| Concern | Client | GitHub endpoint |
|---------|--------|----------------|
| Label definition CRUD | `LabelsClient` | `/repos/{owner}/{repo}/labels` |
| Apply label to issue | `IssuesClient` | `/repos/{owner}/{repo}/issues/{number}/labels` |
| Apply label to PR | `PullRequestsClient` | `/repos/{owner}/{repo}/pulls/{number}/labels` |

## Sub-Client Type

```rust
/// Domain client for repository-level label catalogue operations.
///
/// Obtained via `InstallationClient::labels()`. Cheap to clone (Arc-backed).
#[derive(Debug, Clone)]
pub struct LabelsClient {
    // Internal representation chosen by interface designer
}
```

## Types

`LabelsClient` uses the shared `Label` type defined in `issue.rs` (also used by
`IssuesClient`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,        // 6-digit hex without '#'
    pub default: bool,
}
```

### Request Types

```rust
/// Request to create a new label definition.
#[derive(Debug, Clone, Serialize)]
pub struct CreateLabelRequest {
    /// Label name (required)
    pub name: String,
    /// 6-digit hex color without '#' (required)
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request to update an existing label definition.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateLabelRequest {
    /// New name (renames the label — all existing references update automatically).
    ///
    /// The field is called `new_name` in Rust to distinguish it from the `name`
    /// path parameter on the `update` method. The GitHub API expects the JSON key
    /// to be `"name"`, so the serde rename is required.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub new_name: Option<String>,
    /// New color (6-digit hex without '#')
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
```

## Operations

### `list` ⚡ *auto-paginated*

```rust
impl LabelsClient {
    /// List all label definitions in a repository.
    ///
    /// Auto-paginates using `per_page=100` (ADR-002). The label catalogue
    /// is bounded in size; returning all at once is appropriate.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — repository doesn't exist
    pub async fn list(&self, owner: &str, repo: &str) -> Result<Vec<Label>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/labels?per_page=100`

### `get`

```rust
impl LabelsClient {
    /// Get a single label definition by name.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — label doesn't exist
    pub async fn get(&self, owner: &str, repo: &str, name: &str) -> Result<Label, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/labels/{name}`

### `create`

```rust
impl LabelsClient {
    /// Create a new label definition.
    ///
    /// # Errors
    /// * `ApiError::InvalidRequest` — label with this name already exists (422)
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: CreateLabelRequest,
    ) -> Result<Label, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/labels`

### `update`

```rust
impl LabelsClient {
    /// Update an existing label definition.
    ///
    /// Renaming a label via `new_name` updates all existing issue/PR references.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — label doesn't exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        request: UpdateLabelRequest,
    ) -> Result<Label, ApiError>;
}
```

**Endpoint**: `PATCH /repos/{owner}/{repo}/labels/{name}`

### `delete`

```rust
impl LabelsClient {
    /// Delete a label definition.
    ///
    /// Deleting a label removes it from all issues and PRs it was applied to.
    ///
    /// # Errors
    /// * `ApiError::NotFound` — label doesn't exist
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn delete(&self, owner: &str, repo: &str, name: &str) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/labels/{name}`
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

## Testing Requirements

Tests live in `src/client/issue_tests.rs` in a `label_catalogue_operations` submodule.

| Scenario | Expected |
|----------|---------|
| `list` — empty repository | `Ok(vec![])` |
| `list` — multi-page | Auto-paginates, returns all |
| `get` — existing label | `Ok(Label)` with correct fields |
| `get` — not found | `Err(ApiError::NotFound)` |
| `create` — success | `Ok(Label)` with id assigned |
| `create` — duplicate name | `Err(ApiError::InvalidRequest)` |
| `update` — rename | `Ok(Label)` with new name |
| `update` — not found | `Err(ApiError::NotFound)` |
| `delete` — success | `Ok(())` |
| `delete` — not found | `Err(ApiError::NotFound)` |
