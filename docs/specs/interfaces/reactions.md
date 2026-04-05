# Reactions Interface Specification

**Module**: `github-bot-sdk::client::issue` (extends `issue.rs`)
**Dependencies**: `InstallationClient`, `ApiError`, `IssueUser`, `Comment`

## Overview

Reactions allow GitHub Apps and bots to respond to issues and comments with emoji
acknowledgements (👍 👎 😄 etc.). They are lightweight signals that do not clutter the
comment thread. Bots commonly use reactions to:

- Acknowledge receipt of a command (👀 = "I see it")
- Signal success or failure (✅ ❌)
- Let humans vote on proposals (👍 👎)

Reactions live at two attachment points in the API:

| Attachment | Endpoint |
|-----------|---------|
| Issue | `/repos/{owner}/{repo}/issues/{issue_number}/reactions` |
| Issue comment | `/repos/{owner}/{repo}/issues/comments/{comment_id}/reactions` |

## Types

### ReactionContent

```rust
/// Emoji content for a reaction, as accepted by the GitHub API.
///
/// GitHub uses these exact string values in JSON; the serde representation
/// matches GitHub's API names.
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
```

**Implementation note**: `+1` and `-1` are the only reactions whose API names start with
symbols rather than letters. Explicit `#[serde(rename = "...")]` attributes are required
for those two variants.

### Reaction

```rust
/// A single reaction on an issue or comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    /// Unique reaction identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// User who reacted
    pub user: IssueUser,

    /// The emoji content of this reaction
    pub content: ReactionContent,

    /// When the reaction was created (UTC)
    pub created_at: DateTime<Utc>,
}
```

## Issue Reaction Operations

### List Issue Reactions

```rust
impl InstallationClient {
    /// List all reactions on an issue.
    ///
    /// Auto-paginates through all results (see ADR-002).
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `issue_number` - Issue number
    ///
    /// # Returns
    ///
    /// All reactions on the issue, across all pages.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Issue doesn't exist
    /// * `ApiError::AuthorizationFailed` - Missing `issues:read`
    pub async fn list_issue_reactions(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Vec<Reaction>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/{issue_number}/reactions?per_page=100`

### Create Issue Reaction

```rust
impl InstallationClient {
    /// Add a reaction to an issue.
    ///
    /// If the authenticated user has already reacted with this emoji,
    /// GitHub returns the existing reaction (200) rather than creating a
    /// duplicate (201). Both cases return `Ok(Reaction)`.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `issue_number` - Issue number
    /// * `content` - The emoji reaction to add
    ///
    /// # Returns
    ///
    /// The created or existing `Reaction`.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Issue doesn't exist
    /// * `ApiError::AuthorizationFailed` - Missing `issues:write`
    pub async fn create_issue_reaction(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        content: ReactionContent,
    ) -> Result<Reaction, ApiError>;
}
```

**Endpoint**: `POST /repos/{owner}/{repo}/issues/{issue_number}/reactions`
**Body**: `{ "content": "+1" }`
**Success codes**: 200 (already exists) and 201 (created) both succeed.

### Delete Issue Reaction

```rust
impl InstallationClient {
    /// Remove a reaction from an issue.
    ///
    /// Only the authenticated user can delete their own reactions.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `issue_number` - Issue number
    /// * `reaction_id` - ID of the reaction to remove
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Issue or reaction doesn't exist
    /// * `ApiError::AuthorizationFailed` - Missing `issues:write`
    pub async fn delete_issue_reaction(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        reaction_id: u64,
    ) -> Result<(), ApiError>;
}
```

**Endpoint**: `DELETE /repos/{owner}/{repo}/issues/{issue_number}/reactions/{reaction_id}`
**Success code**: 204 No Content

## Issue Comment Reaction Operations

### List Comment Reactions

```rust
impl InstallationClient {
    /// List all reactions on an issue comment.
    ///
    /// Auto-paginates through all results (see ADR-002).
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `comment_id` - Comment ID (not the issue number)
    ///
    /// # Returns
    ///
    /// All reactions on the comment, across all pages.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Comment doesn't exist
    /// * `ApiError::AuthorizationFailed` - Missing `issues:read`
    pub async fn list_comment_reactions(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<Vec<Reaction>, ApiError>;
}
```

**Endpoint**: `GET /repos/{owner}/{repo}/issues/comments/{comment_id}/reactions?per_page=100`

### Create Comment Reaction

```rust
impl InstallationClient {
    /// Add a reaction to an issue comment.
    ///
    /// If the authenticated user has already reacted with this emoji,
    /// GitHub returns the existing reaction (200) rather than a duplicate.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `comment_id` - Comment ID (not the issue number)
    /// * `content` - The emoji reaction to add
    ///
    /// # Returns
    ///
    /// The created or existing `Reaction`.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Comment doesn't exist
    /// * `ApiError::AuthorizationFailed` - Missing `issues:write`
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
**Body**: `{ "content": "eyes" }`
**Success codes**: 200 (already exists) and 201 (created) both succeed.

### Delete Comment Reaction

```rust
impl InstallationClient {
    /// Remove a reaction from an issue comment.
    ///
    /// Only the authenticated user can delete their own reactions.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `comment_id` - Comment ID (not the issue number)
    /// * `reaction_id` - ID of the reaction to remove
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Comment or reaction doesn't exist
    /// * `ApiError::AuthorizationFailed` - Missing `issues:write`
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
**Success code**: 204 No Content

## Implementation Location

All reaction types and methods are added to `src/client/issue.rs`, consistent with the
existing placement of all issue-domain types. No new source file is needed.

## Error Handling

All reaction operations inherit the standard error mapping:

| HTTP status | `ApiError` variant |
|------------|-------------------|
| 401 | `AuthenticationFailed` |
| 403 | `AuthorizationFailed` |
| 404 | `NotFound` |
| 422 | `InvalidRequest { message }` |
| other | `HttpError { status, message }` |

## Testing Requirements

Per `docs/standards/testing.md`, unit tests must cover:

| Scenario | Expected |
|---------|---------|
| `list_issue_reactions` — empty | Returns `Ok(vec![])` |
| `list_issue_reactions` — single page | Returns all items |
| `list_issue_reactions` — multi-page | Follows `Link: rel="next"`, returns all |
| `create_issue_reaction` — 201 Created | Returns `Ok(Reaction)` |
| `create_issue_reaction` — 200 (duplicate) | Returns `Ok(Reaction)` |
| `create_issue_reaction` — issue not found | Returns `Err(ApiError::NotFound)` |
| `delete_issue_reaction` — success | Returns `Ok(())` |
| `delete_issue_reaction` — not found | Returns `Err(ApiError::NotFound)` |
| Same scenarios repeated for comment reactions | — |

Tests live in `src/client/issue_tests.rs` in a `reaction_operations` submodule.
