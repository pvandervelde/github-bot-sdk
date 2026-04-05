# Reactions Interface Specification

**Module**: `github-bot-sdk::client::issue`
**Contained in**: `src/client/issue.rs`

> **Scope**: This file defines the *types* only. The reaction methods themselves
> (`list_reactions`, `create_reaction`, `delete_reaction`, `list_comment_reactions`,
> `create_comment_reaction`, `delete_comment_reaction`) are specified in
> `issue-operations.md` as part of `IssuesClient`. See ADR-003.

**Dependencies**: `ApiError`, `IssueUser`, `Comment`

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

## API Endpoints (for reference)

| Attachment | Endpoint |
|-----------|----------|
| Issue reactions | `GET/POST/DELETE /repos/{owner}/{repo}/issues/{issue_number}/reactions[/{id}]` |
| Comment reactions | `GET/POST/DELETE /repos/{owner}/{repo}/issues/comments/{comment_id}/reactions[/{id}]` |

For method signatures, error mapping, and pagination behaviour see `issue-operations.md`
(`IssuesClient::list_reactions`, `create_reaction`, `delete_reaction`,
`list_comment_reactions`, `create_comment_reaction`, `delete_comment_reaction`).

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
