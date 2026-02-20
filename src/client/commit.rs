// Commit operations for GitHub API

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::issue::IssueUser;
use crate::client::InstallationClient;
use crate::error::ApiError;

// ============================================================================
// Types
// ============================================================================

/// A complete GitHub commit with all metadata.
///
/// Named `FullCommit` to distinguish it from the minimal [`Commit`] struct in
/// `repository.rs` which only carries a `sha` and `url`.
///
/// # Examples
///
/// ```no_run
/// # use github_bot_sdk::client::InstallationClient;
/// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
/// let commit = client.get_commit("owner", "repo", "main").await?;
/// println!("Latest commit: {} by {}",
///     commit.sha,
///     commit.commit.author.name);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullCommit {
    /// Commit SHA (40-character hexadecimal hash).
    pub sha: String,

    /// Node ID for GraphQL API.
    pub node_id: String,

    /// Git-level commit details (message, author, tree, etc.).
    pub commit: CommitDetails,

    /// GitHub user who authored the commit.
    ///
    /// `None` when the author email does not match any GitHub account.
    pub author: Option<IssueUser>,

    /// GitHub user who committed the change.
    ///
    /// `None` when the committer email does not match any GitHub account.
    pub committer: Option<IssueUser>,

    /// Parent commits. Empty for the initial commit; two entries for merge commits.
    pub parents: Vec<CommitReference>,

    /// API URL for this commit.
    pub url: String,

    /// Web interface URL for this commit.
    pub html_url: String,

    /// Number of comments on this commit.
    pub comment_count: u32,
}

/// Git-level commit metadata.
///
/// Contains the information stored in the Git object itself, as opposed to
/// the GitHub-specific metadata on [`FullCommit`].
///
/// # Examples
///
/// ```no_run
/// # use github_bot_sdk::client::InstallationClient;
/// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
/// let commit = client.get_commit("owner", "repo", "abc123").await?;
/// let details = &commit.commit;
/// println!("Author: {} <{}>", details.author.name, details.author.email);
/// println!("Message: {}", details.message.lines().next().unwrap_or(""));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetails {
    /// Commit author identity from Git config.
    pub author: GitSignature,

    /// Commit committer identity from Git config.
    pub committer: GitSignature,

    /// Full commit message (subject + optional body).
    pub message: String,

    /// Reference to the Git tree object for this commit.
    pub tree: CommitReference,

    /// GPG / SSH signature verification status.
    ///
    /// `None` when the GitHub API does not include verification data.
    pub verification: Option<Verification>,

    /// Number of comments on this commit.
    pub comment_count: u32,
}

/// An identity record stored in a Git commit object.
///
/// Corresponds to Git's `user.name` and `user.email` configuration values
/// together with the timestamp of the action (authoring or committing).
///
/// # Examples
///
/// ```no_run
/// # use github_bot_sdk::client::InstallationClient;
/// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
/// let commit = client.get_commit("owner", "repo", "abc123").await?;
/// let sig = &commit.commit.author;
/// println!("{} <{}> at {}", sig.name, sig.email, sig.date);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSignature {
    /// Name from Git config (`user.name`).
    pub name: String,

    /// Email from Git config (`user.email`).
    pub email: String,

    /// Timestamp of the action (authoring or committing).
    pub date: DateTime<Utc>,
}

/// A minimal reference to a Git object containing only its SHA and API URL.
///
/// Used for parent commits, tree references, and related object links.
///
/// # Examples
///
/// ```no_run
/// # use github_bot_sdk::client::InstallationClient;
/// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
/// let commit = client.get_commit("owner", "repo", "abc123").await?;
/// for parent in &commit.parents {
///     println!("Parent SHA: {}", parent.sha);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitReference {
    /// Git object SHA (40-character hexadecimal hash).
    pub sha: String,

    /// API URL for the referenced object.
    pub url: String,
}

/// GPG or SSH signature verification status for a commit.
///
/// GitHub verifies signatures against known public keys associated with GitHub
/// accounts. The `reason` field explains the outcome of that verification.
///
/// # Examples
///
/// ```no_run
/// # use github_bot_sdk::client::InstallationClient;
/// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
/// let commit = client.get_commit("owner", "repo", "abc123").await?;
/// if let Some(v) = &commit.commit.verification {
///     if v.verified {
///         println!("Commit is signed and verified");
///     } else {
///         println!("Signature issue: {}", v.reason);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    /// Whether the signature is cryptographically valid and the key is trusted.
    pub verified: bool,

    /// Human-readable reason for the verification status.
    ///
    /// Common values: `"valid"`, `"invalid"`, `"expired_key"`, `"unknown_key"`,
    /// `"unsigned"`, `"no_user"`.
    pub reason: String,

    /// Raw GPG signature payload, if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// Signed content (the data that was signed), if present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
}

/// The result of comparing two Git refs (commits, branches, or tags).
///
/// Contains the full list of commits between the two refs and every file that
/// changed, together with statistics useful for generating changelogs and
/// release notes.
///
/// # Examples
///
/// ```no_run
/// # use github_bot_sdk::client::InstallationClient;
/// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
/// let cmp = client.compare_commits("owner", "repo", "v1.0.0", "v1.1.0").await?;
/// println!("Status: {} ({} ahead, {} behind)",
///     cmp.status, cmp.ahead_by, cmp.behind_by);
/// println!("{} commits, {} files changed",
///     cmp.total_commits, cmp.files.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    /// The base commit (starting point of the comparison).
    pub base_commit: FullCommit,

    /// The common ancestor (merge-base) of the two refs.
    pub merge_base_commit: FullCommit,

    /// The head commit (ending point of the comparison).
    pub head_commit: FullCommit,

    /// Relationship between base and head.
    ///
    /// Possible values: `"ahead"`, `"behind"`, `"identical"`, `"diverged"`.
    pub status: String,

    /// Number of commits the head is ahead of the base.
    pub ahead_by: u32,

    /// Number of commits the head is behind the base.
    pub behind_by: u32,

    /// Total number of commits between base and head.
    pub total_commits: u32,

    /// All commits from base to head in chronological order (oldest first).
    ///
    /// GitHub limits this to 250 commits.
    pub commits: Vec<FullCommit>,

    /// All files that changed between base and head.
    pub files: Vec<FileChange>,

    /// Web interface URL for this comparison.
    pub html_url: String,

    /// Permalink URL for this comparison.
    pub permalink_url: String,

    /// URL for the diff view.
    pub diff_url: String,

    /// URL for the patch view.
    pub patch_url: String,
}

/// A single file change within a [`Comparison`].
///
/// Carries per-file statistics (additions, deletions) and the unified diff
/// patch when available.
///
/// # Examples
///
/// ```no_run
/// # use github_bot_sdk::client::InstallationClient;
/// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
/// let cmp = client.compare_commits("owner", "repo", "v1.0.0", "v1.1.0").await?;
/// for file in &cmp.files {
///     println!("{} [{}]: +{} -{}", file.filename, file.status,
///         file.additions, file.deletions);
///     if let Some(prev) = &file.previous_filename {
///         println!("  (renamed from {})", prev);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Current file path in the repository.
    pub filename: String,

    /// Change status for this file.
    ///
    /// Possible values: `"added"`, `"removed"`, `"modified"`, `"renamed"`,
    /// `"copied"`, `"changed"`, `"unchanged"`.
    pub status: String,

    /// Number of lines added.
    pub additions: u32,

    /// Number of lines deleted.
    pub deletions: u32,

    /// Total lines changed (`additions + deletions`).
    pub changes: u32,

    /// URL for the blob view of this file.
    pub blob_url: String,

    /// URL for the raw file content.
    pub raw_url: String,

    /// GitHub Contents API URL for this file.
    pub contents_url: String,

    /// Unified diff patch for this file.
    ///
    /// May be `None` for binary files or when the diff exceeds GitHub's size
    /// limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,

    /// Previous file path, present only when `status` is `"renamed"` or
    /// `"copied"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_filename: Option<String>,
}

// ============================================================================
// Operations
// ============================================================================

impl InstallationClient {
    // ------------------------------------------------------------------------
    // Commit Operations
    // ------------------------------------------------------------------------

    /// Get a single commit by SHA, branch name, or tag name.
    ///
    /// Retrieves complete commit details including the author, committer,
    /// commit message, parent references, and GPG verification status.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner login
    /// * `repo` - Repository name
    /// * `ref_name` - Commit SHA (full or abbreviated), branch name, or tag name
    ///
    /// # Returns
    ///
    /// Returns a [`FullCommit`] with complete metadata.
    ///
    /// # Errors
    ///
    /// * [`ApiError::NotFound`] - Commit or ref doesn't exist, or repository not found
    /// * [`ApiError::AuthorizationFailed`] - Missing `contents:read` permission
    /// * [`ApiError::AuthenticationFailed`] - Token expired or invalid
    /// * [`ApiError::HttpError`] - Unexpected API response
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
    /// See <https://docs.github.com/en/rest/commits/commits#get-a-commit>
    pub async fn get_commit(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
    ) -> Result<FullCommit, ApiError> {
        let path = format!("/repos/{}/{}/commits/{}", owner, repo, ref_name);
        let response = self.get(&path).await?;

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

    /// List commits in a repository with optional filtering.
    ///
    /// Returns commits in reverse chronological order (newest first). All
    /// filter arguments are combined with AND logic.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner login
    /// * `repo` - Repository name
    /// * `sha` - SHA or branch name to start listing from (default: repository default branch)
    /// * `path` - Only return commits that modified this file path or directory
    /// * `author` - Filter by GitHub username or commit author email address
    /// * `since` - Only include commits after this timestamp (inclusive)
    /// * `until` - Only include commits before this timestamp (inclusive)
    /// * `per_page` - Number of results per page; clamped to a maximum of 100
    /// * `page` - Page number for pagination (1-based, default: 1)
    ///
    /// # Returns
    ///
    /// Returns a [`Vec<FullCommit>`] in reverse chronological order.
    ///
    /// # Errors
    ///
    /// * [`ApiError::NotFound`] - Repository not found
    /// * [`ApiError::InvalidRequest`] - Repository is empty (GitHub returns 422)
    /// * [`ApiError::AuthorizationFailed`] - Missing `contents:read` permission
    /// * [`ApiError::HttpError`] - Unexpected API response
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # use chrono::{DateTime, Utc};
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // List recent commits on the default branch
    /// let commits = client.list_commits(
    ///     "owner", "repo",
    ///     None, None, None, None, None, None, None
    /// ).await?;
    ///
    /// // List commits on a feature branch
    /// let feature_commits = client.list_commits(
    ///     "owner", "repo",
    ///     Some("feature-branch"), None, None, None, None, None, None
    /// ).await?;
    ///
    /// // List commits affecting a specific file
    /// let readme_commits = client.list_commits(
    ///     "owner", "repo",
    ///     None, Some("README.md"), None, None, None, Some(50), None
    /// ).await?;
    ///
    /// // List commits by author in a date range
    /// let since = "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>()?;
    /// let until = "2026-01-31T23:59:59Z".parse::<DateTime<Utc>>()?;
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
    /// - `per_page` is silently clamped to 100 (GitHub API maximum).
    /// - Empty repositories cause GitHub to return 422, mapped to [`ApiError::InvalidRequest`].
    ///
    /// # GitHub API
    ///
    /// `GET /repos/{owner}/{repo}/commits`
    ///
    /// See <https://docs.github.com/en/rest/commits/commits#list-commits>
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<Vec<FullCommit>, ApiError> {
        let base = format!("/repos/{}/{}/commits", owner, repo);
        let mut query_params: Vec<String> = Vec::new();

        if let Some(s) = sha {
            query_params.push(format!("sha={}", urlencoding::encode(s)));
        }
        if let Some(p) = path {
            query_params.push(format!("path={}", urlencoding::encode(p)));
        }
        if let Some(a) = author {
            query_params.push(format!("author={}", urlencoding::encode(a)));
        }
        if let Some(s) = since {
            query_params.push(format!("since={}", urlencoding::encode(&s.to_rfc3339())));
        }
        if let Some(u) = until {
            query_params.push(format!("until={}", urlencoding::encode(&u.to_rfc3339())));
        }
        if let Some(pp) = per_page {
            let clamped = pp.min(100);
            query_params.push(format!("per_page={}", clamped));
        }
        if let Some(pg) = page {
            query_params.push(format!("page={}", pg));
        }

        let request_path = if query_params.is_empty() {
            base
        } else {
            format!("{}?{}", base, query_params.join("&"))
        };

        let response = self.get(&request_path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                422 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Git Repository is empty".to_string());
                    ApiError::InvalidRequest { message }
                }
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

    /// Compare two commits, branches, or tags.
    ///
    /// Returns a complete comparison including the list of commits between the
    /// two refs and every file that changed, with per-file statistics. Useful
    /// for changelog generation and release notes automation.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner login
    /// * `repo` - Repository name
    /// * `base` - Base ref (SHA, branch, or tag) — the starting point
    /// * `head` - Head ref (SHA, branch, or tag) — the ending point
    ///
    /// # Returns
    ///
    /// Returns a [`Comparison`] with commits, file changes, and statistics.
    ///
    /// # Errors
    ///
    /// * [`ApiError::NotFound`] - Base or head ref not found, or repository not found
    /// * [`ApiError::AuthorizationFailed`] - Missing `contents:read` permission
    /// * [`ApiError::HttpError`] - Unexpected API response
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // Compare two tags for release notes
    /// let cmp = client.compare_commits("owner", "repo", "v1.0.0", "v1.1.0").await?;
    ///
    /// println!("Status: {}", cmp.status);
    /// println!("Commits: {}", cmp.total_commits);
    /// println!("Files changed: {}", cmp.files.len());
    ///
    /// for commit in &cmp.commits {
    ///     let subject = commit.commit.message.lines().next().unwrap_or("");
    ///     println!("- {}", subject);
    /// }
    ///
    /// // Compare a feature branch to main
    /// let branch_diff = client.compare_commits("owner", "repo", "main", "feature-x").await?;
    /// match branch_diff.status.as_str() {
    ///     "ahead"    => println!("Feature is {} commits ahead", branch_diff.ahead_by),
    ///     "behind"   => println!("Feature is {} commits behind", branch_diff.behind_by),
    ///     "identical" => println!("Branches are identical"),
    ///     "diverged" => println!("Branches have diverged"),
    ///     _ => {}
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Comparison Status Values
    ///
    /// | Status | Meaning |
    /// |--------|---------|
    /// | `"ahead"` | Head has commits not in base |
    /// | `"behind"` | Base has commits not in head |
    /// | `"identical"` | Both refs point to the same commit |
    /// | `"diverged"` | Refs have different histories |
    ///
    /// # Notes
    ///
    /// - Commits are returned in chronological order (oldest to newest).
    /// - GitHub limits comparisons to 250 commits.
    /// - File patches may be absent for binary files or very large diffs.
    ///
    /// # GitHub API
    ///
    /// `GET /repos/{owner}/{repo}/compare/{base}...{head}`
    ///
    /// See <https://docs.github.com/en/rest/commits/commits#compare-two-commits>
    pub async fn compare_commits(
        &self,
        owner: &str,
        repo: &str,
        base: &str,
        head: &str,
    ) -> Result<Comparison, ApiError> {
        let path = format!(
            "/repos/{}/{}/compare/{}...{}",
            owner,
            repo,
            urlencoding::encode(base),
            urlencoding::encode(head)
        );
        let response = self.get(&path).await?;

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
}

#[cfg(test)]
#[path = "commit_tests.rs"]
mod tests;
