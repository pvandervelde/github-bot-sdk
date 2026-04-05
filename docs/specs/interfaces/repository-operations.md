# RepositoriesClient Interface Specification

**Module**: `github-bot-sdk::client::repository`
**Struct**: `RepositoriesClient`
**Obtained via**: `InstallationClient::repositories()`
**Source files**: `src/client/repository.rs`, `src/client/commit.rs`

See **ADR-003** for the sub-client pattern rationale.

## Overview

`RepositoriesClient` provides access to repository metadata, branch management,
Git reference operations, tags, and commit history. Commit operations are logically
part of repository operations and live in the same sub-client.

## Sub-Client Type

```rust
/// Domain client for repository, git-ref, and commit operations.
///
/// Obtained via `InstallationClient::repositories()`. Cheap to clone (Arc-backed).
#[derive(Debug, Clone)]
pub struct RepositoriesClient {
    // Internal representation chosen by interface designer
}
```

## Permissions

| Operation group | Minimum permission |
|----------------|--------------------|
| `get`, list branches/tags/commits, `compare` | `contents: read` |
| `create_ref`, `update_ref`, `delete_ref`, `create_branch`, `create_tag` | `contents: write` |

## Core Types

### Repository

Represents a GitHub repository with metadata.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: RepositoryOwner,
    pub description: Option<String>,
    pub private: bool,
    pub default_branch: String,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
```

### RepositoryOwner

Repository owner (user or organization).

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryOwner {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    #[serde(rename = "type")]
    pub owner_type: OwnerType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OwnerType {
    User,
    Organization,
}
```

### Branch

Represents a Git branch.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: Commit,
    pub protected: bool,
}
```

### Commit

Represents a Git commit reference (used in branches, tags, and pull requests).

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub url: String,
}
```

### GitRef

Represents a Git reference (branch, tag, etc.).

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub node_id: String,
    pub url: String,
    pub object: GitRefObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRefObject {
    pub sha: String,
    #[serde(rename = "type")]
    pub object_type: GitObjectType,
    pub url: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}
```

### Tag

Represents a Git tag.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub commit: Commit,
    pub zipball_url: String,
    pub tarball_url: String,
}
```

## Repository Operations

### `get`

```rust
impl RepositoriesClient {
    /// Get repository metadata.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — Repository doesn't exist or is not accessible
    /// * `ApiError::AuthorizationFailed` — Missing `contents: read`
    ///
    /// # Examples
    ///
    /// ```rust
    /// let repo = client.repositories().get("octocat", "Hello-World").await?;
    /// println!("Repository: {}", repo.full_name);
    /// ```
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Repository, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}`

## Branch Operations

### `list_branches`

```rust
impl RepositoriesClient {
    /// List all branches in a repository.
    ///
    /// Auto-paginates using `per_page=100` (ADR-002).
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — Repository not found
    pub async fn list_branches(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Branch>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/branches?per_page=100`

### `get_branch`

```rust
impl RepositoriesClient {
    /// Get a specific branch by name.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` — Branch doesn't exist
    pub async fn get_branch(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Branch, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/branches/{branch}`

## Git Reference Operations

### `get_ref`

```rust
impl RepositoriesClient {
    /// Get a Git reference (branch head or tag).
    ///
    /// `ref_name` examples: `"heads/main"`, `"tags/v1.0.0"`
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Reference doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// let r = client.repositories().get_ref("octocat", "Hello-World", "heads/main").await?;
    /// println!("SHA: {}", r.object.sha);
    /// ```
    pub async fn get_ref(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
    ) -> Result<GitRef, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/git/refs/{ref_name}`

### `create_ref`

```rust
impl RepositoriesClient {
    /// Create a new Git reference (branch or tag).
    ///
    /// `ref_name` must be fully qualified, e.g. `"refs/heads/feature-branch"`.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` - Missing `contents: write`
    /// * `ApiError::InvalidRequest` - Reference already exists (422)
    /// * `ApiError::NotFound` - SHA doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// let r = client.repositories()
    ///     .create_ref("octocat", "Hello-World", "refs/heads/new-feature", "abc123")
    ///     .await?;
    /// ```
    pub async fn create_ref(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
        sha: &str,
    ) -> Result<GitRef, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/git/refs`

### `update_ref`

```rust
impl RepositoriesClient {
    /// Update an existing Git reference.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` - Missing `contents: write`
    /// * `ApiError::InvalidRequest` - Non-fast-forward without `force` (422)
    pub async fn update_ref(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
        sha: &str,
        force: bool,
    ) -> Result<GitRef, ApiError>;
}
```

**Endpoint**: `PATCH /repos/{owner}/{repo}/git/refs/{ref_name}`

### `delete_ref`

```rust
impl RepositoriesClient {
    /// Delete a Git reference.
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` - Missing `contents: write`
    /// * `ApiError::NotFound` - Reference doesn't exist
    pub async fn delete_ref(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/git/refs/{ref_name}`
**Success**: 204 No Content

## Tag Operations

### `list_tags`

```rust
impl RepositoriesClient {
    /// List all tags in a repository.
    ///
    /// Auto-paginates using `per_page=100` (ADR-002).
    pub async fn list_tags(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<Tag>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/tags?per_page=100`

## Convenience Methods

### `create_branch`

Convenience wrapper around `create_ref` for branch creation.

```rust
impl RepositoriesClient {
    /// Create a new branch from a commit SHA.
    ///
    /// `branch_name` is the short name (without `refs/heads/` prefix).
    ///
    /// # Errors
    ///
    /// * `ApiError::AuthorizationFailed` - Missing `contents: write`
    /// * `ApiError::InvalidRequest` - Branch already exists (422)
    /// * `ApiError::NotFound` - SHA doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// let main = client.repositories().get_branch("owner", "repo", "main").await?;
    /// let branch = client.repositories()
    ///     .create_branch("owner", "repo", "feature", &main.commit.sha)
    ///     .await?;
    /// ```
    pub async fn create_branch(
        &self,
        owner: &str,
        repo: &str,
        branch_name: &str,
        from_sha: &str,
    ) -> Result<GitRef, ApiError>;
}
```

**Implementation**: Calls `self.create_ref(owner, repo, &format!("refs/heads/{branch_name}"), from_sha)`

### `create_tag`

Convenience wrapper around `create_ref` for lightweight tag creation.

```rust
impl RepositoriesClient {
    /// Create a new lightweight tag pointing at a commit SHA.
    ///
    /// For annotated tags, use the Git Data API (not currently implemented).
    ///
    /// # Examples
    ///
    /// ```rust
    /// client.repositories().create_tag("owner", "repo", "v1.0.0", sha).await?;
    /// ```
    pub async fn create_tag(
        &self,
        owner: &str,
        repo: &str,
        tag_name: &str,
        from_sha: &str,
    ) -> Result<GitRef, ApiError>;
}
```

**Implementation**: Calls `self.create_ref(owner, repo, &format!("refs/tags/{tag_name}"), from_sha)`

## Commit Operations

### `get_commit`

```rust
impl RepositoriesClient {
    /// Get full commit details by SHA.
    pub async fn get_commit(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
    ) -> Result<FullCommit, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/commits/{sha}`

### `list_commits`

```rust
impl RepositoriesClient {
    /// List commits on a branch, optionally scoped to a file path.
    ///
    /// Returns a single page of results (manual pagination — commit history
    /// can be arbitrarily large, ADR-002).
    pub async fn list_commits(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        path: Option<&str>,
        page: u32,
    ) -> Result<PagedResponse<CommitDetails>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/commits?sha={branch}&path={path}&page={page}&per_page=30`

### `compare`

```rust
impl RepositoriesClient {
    /// Compare two commits or refs to get the diff and divergence info.
    pub async fn compare(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<Comparison, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/compare/{base}...{head}`

## Usage Examples

### Get Repository Metadata

```rust
let repo = client.repositories().get("octocat", "Hello-World").await?;
println!("Default branch: {}", repo.default_branch);
println!("Created: {}", repo.created_at);
```

### Create a New Branch

```rust
let repos = client.repositories();
let main = repos.get_branch("octocat", "Hello-World", "main").await?;
let _branch = repos.create_branch("octocat", "Hello-World", "feature", &main.commit.sha).await?;
```

### List All Tags

```rust
let tags = client.repositories().list_tags("octocat", "Hello-World").await?;
for tag in tags {
    println!("Tag: {} -> {}", tag.name, tag.commit.sha);
}
```

## Implementation Notes

### Path Construction

All operations use GitHub REST API v3 paths:

- Repository: `/repos/{owner}/{repo}`
- Branches: `/repos/{owner}/{repo}/branches`
- Branch: `/repos/{owner}/{repo}/branches/{branch}`
- Git refs: `/repos/{owner}/{repo}/git/refs/{ref}`
- Tags: `/repos/{owner}/{repo}/tags`

### Reference Names

Git references use specific prefixes:

- Branches: `refs/heads/{name}` or `heads/{name}`
- Tags: `refs/tags/{name}` or `tags/{name}`

API returns full `refs/` prefix, but accepts shortened form.

### Testing Strategy

- Mock HTTP responses for all operations
- Test error mapping (404, 403, 422)
- Verify correct path construction
- Test reference name normalization

## Assertions

Supports:

- **Assertion #3a**: Uses installation tokens
- **Assertion #6**: Repository-level operations

## References

- GitHub API: [Repositories](https://docs.github.com/en/rest/repos/repos)
- GitHub API: [Git References](https://docs.github.com/en/rest/git/refs)
- [architecture.md](../architecture.md) - Architecture boundaries and dependencies
- [assertions.md](../assertions.md) - Behavioral requirements
