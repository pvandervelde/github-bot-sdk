# Additional Operations Interface Specification

**Module**: `github-bot-sdk::client::{issue, workflow, release}`
**Files**:

- `src/client/issue.rs` — `MilestonesClient`
- `src/client/workflow.rs` — `WorkflowsClient`
- `src/client/release.rs` — `ReleasesClient`

**Dependencies**: `InstallationClient`, `ApiError`, shared types

**Sub-client pattern**: All three clients follow ADR-003 — they are obtained via factory
methods on `InstallationClient` and are zero-cost to construct (no API call).

```rust
let milestones = client.milestones();   // → MilestonesClient
let workflows  = client.workflows();    // → WorkflowsClient
let releases   = client.releases();     // → ReleasesClient
```

## Overview

This specification covers additional GitHub operations for milestones, workflows, and releases. These are installation-scoped operations requiring appropriate repository permissions.

## Milestone Operations

See the authoritative specification in [milestones-client.md](./milestones-client.md).

### Types

#### Milestone

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

#### MilestoneState

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MilestoneState {
    Open,
    Closed,
}
```

### Operations

#### `list`

```rust
impl MilestonesClient {
    /// List all milestones in a repository (auto-paginated).
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — repository does not exist
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
        query: Option<ListMilestonesQuery>,
    ) -> Result<Vec<Milestone>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/milestones?per_page=100`

#### `get`

```rust
impl MilestonesClient {
    /// Get a single milestone by its repository-scoped number.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — milestone does not exist
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        milestone_number: u64,
    ) -> Result<Milestone, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/milestones/{milestone_number}`

#### `create`

```rust
impl MilestonesClient {
    /// Create a new milestone.
    ///
    /// # Errors
    ///
    /// * `ApiError::InvalidRequest` — title is empty (422)
    /// * `ApiError::AuthorizationFailed` — missing `issues: write`
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: CreateMilestoneRequest,
    ) -> Result<Milestone, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/milestones`

#### `update`

```rust
impl MilestonesClient {
    /// Update an existing milestone.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — milestone does not exist
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

#### `delete`

```rust
impl MilestonesClient {
    /// Delete a milestone.
    ///
    /// Issues assigned to the deleted milestone are unlinked but otherwise unaffected.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — milestone does not exist
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

### Request Types

```rust
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateMilestoneRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<MilestoneState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<DateTime<Utc>>,
}
```

## Workflow Operations

### Types

#### Workflow

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub path: String,
    pub state: WorkflowState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub url: String,
    pub html_url: String,
    pub badge_url: String,
}
```

#### WorkflowState

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    Active,
    DisabledManually,
    DisabledInactivity,
    DisabledFork,
    Deleted,
}
```

#### WorkflowRun

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub run_number: u64,
    pub event: String,
    pub status: WorkflowRunStatus,
    pub conclusion: Option<WorkflowRunConclusion>,
    pub workflow_id: u64,
    pub head_branch: String,
    pub head_sha: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub url: String,
    pub html_url: String,
}
```

#### WorkflowRunStatus

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunStatus {
    Queued,
    InProgress,
    Completed,
    Waiting,
    Requested,
    Pending,
}
```

#### WorkflowRunConclusion

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunConclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
    Stale,
    Neutral,
}
```

### Operations

#### `list`

```rust
impl WorkflowsClient {
    /// List workflows in a repository (auto-paginated).
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Workflow>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/actions/workflows`

#### `get`

```rust
impl WorkflowsClient {
    /// Get a specific workflow by ID.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — workflow doesn't exist
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: u64,
    ) -> Result<Workflow, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}`

#### `trigger`

```rust
impl WorkflowsClient {
    /// Trigger a workflow dispatch event.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` — missing `actions: write` permission
    /// * `ApiError::NotFound` — workflow doesn't exist or has no `workflow_dispatch` trigger
    pub async fn trigger(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: u64,
        request: TriggerWorkflowRequest,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/actions/workflows/{workflow_id}/dispatches`
**Success**: 204 No Content

#### `list_runs`

```rust
impl WorkflowsClient {
    /// List runs for a specific workflow (auto-paginated).
    pub async fn list_runs(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: u64,
    ) -> Result<Vec<WorkflowRun>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs`

#### `get_run`

```rust
impl WorkflowsClient {
    /// Get a specific workflow run by ID.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — run doesn't exist
    pub async fn get_run(
        &self,
        owner: &str,
        repo: &str,
        run_id: u64,
    ) -> Result<WorkflowRun, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/actions/runs/{run_id}`

#### `cancel_run`

```rust
impl WorkflowsClient {
    /// Cancel a workflow run.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — run doesn't exist
    pub async fn cancel_run(
        &self,
        owner: &str,
        repo: &str,
        run_id: u64,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/actions/runs/{run_id}/cancel`
**Success**: 202 Accepted

#### `rerun_run`

```rust
impl WorkflowsClient {
    /// Re-run a workflow run.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — run doesn't exist
    pub async fn rerun_run(
        &self,
        owner: &str,
        repo: &str,
        run_id: u64,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/actions/runs/{run_id}/rerun`
**Success**: 201 Created

### Request Types

```rust
#[derive(Debug, Clone, Serialize)]
pub struct TriggerWorkflowRequest {
    /// Git reference (branch or tag)
    #[serde(rename = "ref")]
    pub git_ref: String,

    /// Workflow inputs (key-value pairs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<std::collections::HashMap<String, String>>,
}
```

## Release Operations

### Types

#### Release

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub id: u64,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub author: IssueUser,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub url: String,
    pub html_url: String,
    pub assets: Vec<ReleaseAsset>,
}
```

### Operations

#### `list`

```rust
impl ReleasesClient {
    /// List releases in a repository (most recent first, auto-paginated).
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Release>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/releases`

#### `get`

```rust
impl ReleasesClient {
    /// Get a specific release by ID.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — release doesn't exist
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        release_id: u64,
    ) -> Result<Release, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/releases/{release_id}`

#### `get_latest`

```rust
impl ReleasesClient {
    /// Get the latest published (non-draft, non-prerelease) release.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — no published release exists
    pub async fn get_latest(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Release, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/releases/latest`

#### `get_by_tag`

```rust
impl ReleasesClient {
    /// Get a release by its tag name.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — no release for this tag
    pub async fn get_by_tag(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
    ) -> Result<Release, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/releases/tags/{tag}`

#### `create`

```rust
impl ReleasesClient {
    /// Create a new release.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` — missing permission
    /// * `ApiError::InvalidRequest` — tag doesn't exist (422)
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        request: CreateReleaseRequest,
    ) -> Result<Release, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/releases`

#### `update`

```rust
impl ReleasesClient {
    /// Update an existing release.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — release doesn't exist
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        release_id: u64,
        request: UpdateReleaseRequest,
    ) -> Result<Release, ApiError>;
}
```

**Endpoint**: `PATCH /repos/{owner}/{repo}/releases/{release_id}`

#### `delete`

```rust
impl ReleasesClient {
    /// Delete a release.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` — missing permission
    /// * `ApiError::NotFound` — release doesn't exist
    pub async fn delete(
        &self,
        owner: &str,
        repo: &str,
        release_id: u64,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/releases/{release_id}`
**Success**: 204 No Content

### Request Types

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CreateReleaseRequest {
    /// Tag name (required)
    pub tag_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_commitish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<bool>,
    /// Auto-generate release name and notes from merged PRs. Create-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_release_notes: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateReleaseRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<bool>,
}
```

## API Paths

### Milestones

- List: `GET /repos/{owner}/{repo}/milestones`
- Get / Update / Delete: `/repos/{owner}/{repo}/milestones/{milestone_number}`
- Create: `POST /repos/{owner}/{repo}/milestones`

### Workflows

- List workflows: `GET /repos/{owner}/{repo}/actions/workflows`
- Get workflow: `GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}`
- Trigger dispatch: `POST /repos/{owner}/{repo}/actions/workflows/{workflow_id}/dispatches`
- List runs: `GET /repos/{owner}/{repo}/actions/workflows/{workflow_id}/runs`
- Get / Cancel / Re-run: `/repos/{owner}/{repo}/actions/runs/{run_id}`

### Releases

- List: `GET /repos/{owner}/{repo}/releases`
- Get: `GET /repos/{owner}/{repo}/releases/{release_id}`
- Get latest: `GET /repos/{owner}/{repo}/releases/latest`
- Get by tag: `GET /repos/{owner}/{repo}/releases/tags/{tag}`
- Create: `POST /repos/{owner}/{repo}/releases`
- Update: `PATCH /repos/{owner}/{repo}/releases/{release_id}`
- Delete: `DELETE /repos/{owner}/{repo}/releases/{release_id}`

## References

- GitHub API: [Milestones](https://docs.github.com/en/rest/issues/milestones)
- GitHub API: [Workflows](https://docs.github.com/en/rest/actions/workflows)
- GitHub API: [Releases](https://docs.github.com/en/rest/releases/releases)
- ADR-003: [Domain Sub-Client Pattern](../adr/ADR-003-sub-client-api-pattern.md)
