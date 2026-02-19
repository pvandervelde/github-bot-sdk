# Commit Operations Interface Specification

**Module**: `github-bot-sdk::client::commit`
**File**: `src/client/commit.rs`
**Dependencies**: `InstallationClient`, `ApiError`, `IssueUser`, `chrono::DateTime`

## Overview

Commit operations provide access to repository commit history, enabling retrieval of individual commits, listing commits with filters, and comparing refs to determine changes. These operations are essential for release automation, changelog generation, and version analysis workflows.

## Architectural Location

**Layer**: Infrastructure adapter (GitHub API operations)
**Purpose**: Commit retrieval, history listing, and comparison
**Required Permissions**: `contents:read` (minimum)

## Core Types

### FullCommit

Represents a complete GitHub commit with all metadata.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullCommit {
    /// Commit SHA (40-character hexadecimal hash)
    pub sha: String,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Git-level commit details
    pub commit: CommitDetails,

    /// GitHub user who authored (if email matches GitHub account)
    pub author: Option<IssueUser>,

    /// GitHub user who committed (if email matches GitHub account)
    pub committer: Option<IssueUser>,

    /// Parent commits
    pub parents: Vec<CommitReference>,

    /// Commit URL (API)
    pub url: String,

    /// Commit URL (web interface)
    pub html_url: String,

    /// Comment count
    pub comment_count: u32,
}
```

**Note**: Named `FullCommit` to distinguish from the minimal `Commit` struct in `repository.rs` which only contains `sha` and `url`.

### CommitDetails

Git-level commit metadata.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetails {
    /// Commit author (from Git config)
    pub author: GitSignature,

    /// Commit committer (from Git config)
    pub committer: GitSignature,

    /// Full commit message
    pub message: String,

    /// Tree SHA
    pub tree: CommitReference,

    /// GPG signature verification (if signed)
    pub verification: Option<Verification>,

    /// Comment count
    pub comment_count: u32,
}
```

### GitSignature

Identity record from Git.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSignature {
    /// Name from Git config (user.name)
    pub name: String,

    /// Email from Git config (user.email)
    pub email: String,

    /// Timestamp of action
    pub date: DateTime<Utc>,
}
```

### CommitReference

Minimal commit reference (SHA and URL only).

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitReference {
    /// Git object SHA
    pub sha: String,

    /// API URL for the object
    pub url: String,
}
```

### Verification

GPG signature verification status.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    /// Whether signature is valid
    pub verified: bool,

    /// Reason for verification status
    pub reason: String, // "valid", "invalid", "expired_key", "unknown_key", "unsigned", etc.

    /// GPG signature payload
    pub signature: Option<String>,

    /// Signed content
    pub payload: Option<String>,
}
```

### Comparison

Result of comparing two refs.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    /// Base commit (starting point)
    pub base_commit: FullCommit,

    /// Merge base commit (common ancestor)
    pub merge_base_commit: FullCommit,

    /// Head commit (ending point)
    pub head_commit: FullCommit,

    /// Comparison status
    pub status: String, // "ahead", "behind", "identical", "diverged"

    /// Number of commits ahead
    pub ahead_by: u32,

    /// Number of commits behind
    pub behind_by: u32,

    /// Total commits in comparison
    pub total_commits: u32,

    /// All commits from base to head
    pub commits: Vec<FullCommit>,

    /// All file changes
    pub files: Vec<FileChange>,

    /// Comparison URL (web interface)
    pub html_url: String,

    /// Permalink URL
    pub permalink_url: String,

    /// Diff URL
    pub diff_url: String,

    /// Patch URL
    pub patch_url: String,
}
```

### FileChange

Description of a file change in a comparison.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Current filename (path in repository)
    pub filename: String,

    /// Change status
    pub status: String, // "added", "removed", "modified", "renamed", "copied"

    /// Lines added
    pub additions: u32,

    /// Lines deleted
    pub deletions: u32,

    /// Total lines changed
    pub changes: u32,

    /// Blob URL
    pub blob_url: String,

    /// Raw URL
    pub raw_url: String,

    /// Contents URL
    pub contents_url: String,

    /// Unified diff patch (may be truncated or absent for large diffs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,

    /// Previous filename (for renamed files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_filename: Option<String>,
}
```

## Commit Operations

### Get Commit

Retrieve a single commit by SHA or ref.

```rust
impl InstallationClient {
    /// Get a single commit by SHA or ref.
    ///
    /// Retrieves complete commit details including author, committer, message,
    /// parents, and verification status.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner login
    /// * `repo` - Repository name
    /// * `ref_name` - Commit SHA, branch name, or tag name
    ///
    /// # Returns
    ///
    /// Returns `FullCommit` with complete metadata.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Commit/ref doesn't exist or repository not found
    /// * `ApiError::AuthorizationFailed` - Missing `contents:read` permission
    /// * `ApiError::AuthenticationFailed` - Token expired or invalid
    /// * `ApiError::HttpError` - Other API errors
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // Get by SHA
    /// let commit = client.get_commit("owner", "repo", "abc123def456").await?;
    /// println!("Message: {}", commit.commit.message);
    ///
    /// // Get by branch name
    /// let head = client.get_commit("owner", "repo", "main").await?;
    /// println!("HEAD: {}", head.sha);
    ///
    /// // Get by tag
    /// let release = client.get_commit("owner", "repo", "v1.0.0").await?;
    /// println!("Release commit: {}", release.sha);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # GitHub API
    ///
    /// `GET /repos/{owner}/{repo}/commits/{ref}`
    ///
    /// See: https://docs.github.com/en/rest/commits/commits#get-a-commit
    pub async fn get_commit(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
    ) -> Result<FullCommit, ApiError>;
}
```

**Implementation Notes**:

1. Build path: `/repos/{owner}/{repo}/commits/{ref_name}`
2. Call `self.get(path)`
3. Parse JSON response to `FullCommit`
4. Map errors: 404 → NotFound, 403 → AuthorizationFailed, 401 → AuthenticationFailed

### List Commits

List commits in a repository with optional filters.

```rust
impl InstallationClient {
    /// List commits in a repository with optional filtering.
    ///
    /// Returns commits in reverse chronological order (newest first) with
    /// support for filtering by ref, path, author, and date range.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner login
    /// * `repo` - Repository name
    /// * `sha` - SHA or ref to list commits from (default: default branch)
    /// * `path` - Only commits modifying this file or directory
    /// * `author` - GitHub username or email address
    /// * `since` - Only commits after this date
    /// * `until` - Only commits before this date
    /// * `per_page` - Results per page (max 100, default 30)
    /// * `page` - Page number for pagination (default 1)
    ///
    /// # Returns
    ///
    /// Returns vector of `FullCommit` in reverse chronological order.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Repository not found
    /// * `ApiError::InvalidRequest` - Empty repository (422 status)
    /// * `ApiError::AuthorizationFailed` - Missing permissions
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # use chrono::{DateTime, Utc};
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // List recent commits on default branch
    /// let commits = client.list_commits(
    ///     "owner", "repo",
    ///     None, None, None, None, None, None, None
    /// ).await?;
    ///
    /// // List commits on specific branch
    /// let feature_commits = client.list_commits(
    ///     "owner", "repo",
    ///     Some("feature-branch"), None, None, None, None, None, None
    /// ).await?;
    ///
    /// // List commits affecting specific file
    /// let readme_commits = client.list_commits(
    ///     "owner", "repo",
    ///     None, Some("README.md"), None, None, None, Some(50), None
    /// ).await?;
    ///
    /// // List commits by author in date range
    /// let since = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")?.with_timezone(&Utc);
    /// let until = DateTime::parse_from_rfc3339("2026-01-31T23:59:59Z")?.with_timezone(&Utc);
    /// let author_commits = client.list_commits(
    ///     "owner", "repo",
    ///     None, None, Some("alice"), Some(since), Some(until), None, None
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// - All filters are combined with AND logic
    /// - Maximum 100 items per page (GitHub API limit)
    /// - Empty repositories return 422 InvalidRequest
    /// - Path filter includes commits that modified file or directory
    /// - Author matches GitHub username OR email address
    ///
    /// # GitHub API
    ///
    /// `GET /repos/{owner}/{repo}/commits`
    ///
    /// See: https://docs.github.com/en/rest/commits/commits#list-commits
    pub async fn list_commits(
        &self,
        owner: &str,
        repo: &str,
        sha: Option<&str>,
        path: Option<&str>,
        author: Option<&str>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<Vec<FullCommit>, ApiError>;
}
```

**Implementation Notes**:

1. Build path: `/repos/{owner}/{repo}/commits`
2. Add query parameters:
   - `sha={sha}` if provided
   - `path={path}` if provided (URL encode)
   - `author={author}` if provided (URL encode)
   - `since={since}` if provided (ISO 8601 format)
   - `until={until}` if provided (ISO 8601 format)
   - `per_page={per_page}` if provided (clamp to 100 max)
   - `page={page}` if provided
3. Call `self.get(path)`
4. Parse JSON array response to `Vec<FullCommit>`
5. Map errors: 404 → NotFound, 422 → InvalidRequest, 403 → AuthorizationFailed

### Compare Commits

Compare two refs to determine changes.

```rust
impl InstallationClient {
    /// Compare two commits, branches, or tags.
    ///
    /// Returns complete comparison including commits between refs and all
    /// file changes with statistics. Useful for changelog generation and
    /// release notes.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner login
    /// * `repo` - Repository name
    /// * `base` - Base ref (SHA, branch, or tag) - starting point
    /// * `head` - Head ref (SHA, branch, or tag) - ending point
    ///
    /// # Returns
    ///
    /// Returns `Comparison` with commits, file changes, and statistics.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Base or head ref not found, or repository not found
    /// * `ApiError::AuthorizationFailed` - Missing permissions
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // Compare two tags (for release notes)
    /// let comparison = client.compare_commits(
    ///     "owner", "repo",
    ///     "v1.0.0", "v1.1.0"
    /// ).await?;
    ///
    /// println!("Status: {}", comparison.status);
    /// println!("Commits: {}", comparison.total_commits);
    /// println!("Files changed: {}", comparison.files.len());
    ///
    /// // Generate changelog from commits
    /// for commit in comparison.commits {
    ///     println!("- {}", commit.commit.message.lines().next().unwrap());
    /// }
    ///
    /// // Analyze file changes
    /// for file in comparison.files {
    ///     println!("{}: +{} -{}", file.filename, file.additions, file.deletions);
    /// }
    ///
    /// // Compare branch to main
    /// let branch_diff = client.compare_commits(
    ///     "owner", "repo",
    ///     "main", "feature-branch"
    /// ).await?;
    ///
    /// match branch_diff.status.as_str() {
    ///     "ahead" => println!("Feature is ahead by {} commits", branch_diff.ahead_by),
    ///     "behind" => println!("Feature is behind by {} commits", branch_diff.behind_by),
    ///     "identical" => println!("Branches are identical"),
    ///     "diverged" => println!("Branches have diverged"),
    ///     _ => {},
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Comparison Status Values
    ///
    /// - `ahead`: Head is ahead of base (base is older)
    /// - `behind`: Head is behind base (base is newer)
    /// - `identical`: Both refs point to same commit
    /// - `diverged`: Refs have different histories (need merge)
    ///
    /// # Notes
    ///
    /// - Commits are in chronological order (oldest to newest)
    /// - File patches may be truncated for large diffs
    /// - GitHub limits comparison to 250 commits
    ///
    /// # GitHub API
    ///
    /// `GET /repos/{owner}/{repo}/compare/{base}...{head}`
    ///
    /// See: https://docs.github.com/en/rest/commits/commits#compare-two-commits
    pub async fn compare_commits(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<Comparison, ApiError>;
}
```

**Implementation Notes**:

1. Build path: `/repos/{owner}/{repo}/compare/{base}...{head}`
2. URL encode base and head refs
3. Call `self.get(path)`
4. Parse JSON response to `Comparison`
5. Map errors: 404 → NotFound, 403 → AuthorizationFailed

## Error Handling

### Error Mapping

| HTTP Status | ApiError Variant | Scenario |
|-------------|-----------------|----------|
| 404 | `ApiError::NotFound` | Commit/ref/repository not found |
| 422 | `ApiError::InvalidRequest` | Invalid SHA format, empty repository |
| 403 | `ApiError::AuthorizationFailed` | Insufficient permissions |
| 401 | `ApiError::AuthenticationFailed` | Token expired/invalid |
| 429 | `ApiError::RateLimitExceeded` | Rate limit exceeded |
| 5xx | `ApiError::HttpError` | GitHub API failure |

### Error Context

All errors must preserve context for debugging:
- Operation being performed
- Repository (owner/repo)
- Ref/SHA that failed
- GitHub API error message (if available)

**Security**: Never include authentication tokens in error messages.

## Testing Requirements

Based on assertions 31-37 in `docs/specs/assertions.md`:

### Unit Tests

1. **Deserialization Tests**:
   - Deserialize `FullCommit` from GitHub API JSON
   - Deserialize `Comparison` with all fields
   - Deserialize `FileChange` with all status types
   - Handle optional fields (author, committer, verification)
   - Handle commits with multiple parents (merges)
   - Handle commits with no parents (initial commit)

2. **URL Construction Tests**:
   - Build correct endpoint for get_commit
   - Build correct query string for list_commits with all filters
   - Build correct comparison endpoint format
   - URL encode special characters in refs

3. **Error Mapping Tests**:
   - Map 404 to NotFound
   - Map 422 to InvalidRequest
   - Map 403 to AuthorizationFailed
   - Map 401 to AuthenticationFailed

### Integration Tests

1. **Get Commit** (Assertion 31):
   - Get commit by SHA from public repository
   - Get commit by branch name
   - Get commit by tag name
   - Handle 404 for non-existent commit
   - Verify author/committer GitHub user associations
   - Verify GPG signature verification when present

2. **List Commits** (Assertion 32):
   - List commits returns reverse chronological order
   - Path filter returns only commits affecting that path
   - Author filter returns only commits by that author
   - Date range filter returns commits in range
   - Combined filters use AND logic
   - Empty repository returns InvalidRequest (422)
   - Pagination works correctly

3. **Compare Commits** (Assertions 33-35):
   - Compare identical refs returns "identical" status
   - Compare tags shows correct commit count
   - Commits are in chronological order
   - File changes include all modified files
   - File changes have correct statistics
   - Renamed files include previous_filename
   - Status values correct (ahead/behind/identical/diverged)

4. **Performance** (Assertion 37):
   - get_commit completes in <200ms (p95)
   - list_commits completes in <500ms (p95)
   - compare_commits completes in <1000ms (p95)
   - Single API call per operation
   - No redundant requests

## Usage Examples

### Generate Changelog Between Releases

```rust
use github_bot_sdk::client::InstallationClient;

async fn generate_changelog(
    client: &InstallationClient,
    owner: &str,
    repo: &str,
    from_tag: &str,
    to_tag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let comparison = client.compare_commits(owner, repo, from_tag, to_tag).await?;
    
    let mut changelog = format!("# Changes from {} to {}\n\n", from_tag, to_tag);
    changelog.push_str(&format!("**{} commits** by {} contributors\n\n", 
        comparison.total_commits,
        comparison.commits.iter()
            .filter_map(|c| c.author.as_ref().map(|a| a.login.as_str()))
            .collect::<std::collections::HashSet<_>>()
            .len()
    ));
    
    changelog.push_str("## Commits\n\n");
    for commit in &comparison.commits {
        let message = commit.commit.message.lines().next().unwrap_or("");
        let author = commit.author.as_ref()
            .map(|a| a.login.as_str())
            .unwrap_or("unknown");
        changelog.push_str(&format!("- {} (@{})\n", message, author));
    }
    
    changelog.push_str(&format!("\n## Files Changed ({})\n\n", comparison.files.len()));
    for file in &comparison.files {
        changelog.push_str(&format!("- `{}`: +{} -{}\n", 
            file.filename, file.additions, file.deletions));
    }
    
    Ok(changelog)
}
```

### Find Commits by Author in Date Range

```rust
use github_bot_sdk::client::InstallationClient;
use chrono::{DateTime, Utc};

async fn find_author_commits(
    client: &InstallationClient,
    owner: &str,
    repo: &str,
    author: &str,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let commits = client.list_commits(
        owner, repo,
        None,  // default branch
        None,  // all files
        Some(author),
        Some(since),
        Some(until),
        Some(100),  // max per page
        None,  // first page
    ).await?;
    
    Ok(commits.iter()
        .map(|c| c.commit.message.lines().next().unwrap_or("").to_string())
        .collect())
}
```

### Check if Commit Exists Before Creating Release

```rust
use github_bot_sdk::client::InstallationClient;
use github_bot_sdk::error::ApiError;

async fn validate_release_commit(
    client: &InstallationClient,
    owner: &str,
    repo: &str,
    commit_sha: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    match client.get_commit(owner, repo, commit_sha).await {
        Ok(_) => Ok(true),
        Err(ApiError::NotFound) => Ok(false),
        Err(e) => Err(e.into()),
    }
}
```

## Implementation Checklist

- [ ] Create `src/client/commit.rs` module
- [ ] Define all types (FullCommit, CommitDetails, GitSignature, CommitReference, Verification, Comparison, FileChange)
- [ ] Implement `get_commit` operation
- [ ] Implement `list_commits` operation with all filter parameters
- [ ] Implement `compare_commits` operation
- [ ] Add comprehensive rustdoc to all types and methods
- [ ] Create `src/client/commit_tests.rs` with unit tests
- [ ] Add integration tests for all three operations
- [ ] Verify error handling for all status codes
- [ ] Test pagination behavior
- [ ] Test filter combinations
- [ ] Validate performance targets
- [ ] Update module exports in `src/client/mod.rs`
- [ ] Update `README.md` with commit operations examples

## References

- **Specification**: `docs/specs/vocabulary.md` (Commit concepts)
- **Responsibilities**: `docs/specs/responsibilities.md` (CommitOperations component)
- **Assertions**: `docs/specs/assertions.md` (Assertions 31-37)
- **Constraints**: `docs/specs/constraints.md` (Implementation rules)
- **GitHub API Docs**:
  - [Get a commit](https://docs.github.com/en/rest/commits/commits#get-a-commit)
  - [List commits](https://docs.github.com/en/rest/commits/commits#list-commits)
  - [Compare commits](https://docs.github.com/en/rest/commits/commits#compare-two-commits)
