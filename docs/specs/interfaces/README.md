# GitHub Bot SDK Interface Specifications

This directory contains detailed interface specifications for the GitHub Bot SDK
installation-level client operations. Each specification defines type signatures,
function contracts, error conditions, and usage examples.

## Architecture: Domain Sub-Clients (ADR-003)

All GitHub API operations are grouped into **domain sub-clients** accessed via factory
methods on `InstallationClient`. Each sub-client is cheap to construct (no API call,
at most an `Arc` increment) and contains flat methods (no further nesting).

```rust
// All operations accessed through dedicated sub-clients
client.issues().list("owner", "repo", state, page).await?;
client.pull_requests().merge("owner", "repo", 42, None).await?;
client.labels().create("owner", "repo", req).await?;
client.milestones().list("owner", "repo", None).await?;
client.repositories().get_branch("owner", "repo", "main").await?;
client.workflows().trigger("owner", "repo", "ci.yml", req).await?;
client.releases().get_latest("owner", "repo").await?;
client.projects().list_for_issue("owner", "repo", 42).await?;
```

## Interface Documents

### Core Client

- **[installation-client.md](./installation-client.md)** — `InstallationClient` foundation
  - Client struct definition and thread-safety guarantees
  - Generic request helpers (GET, POST, PUT, DELETE, PATCH)
  - **Domain sub-client factory methods** (`issues()`, `pull_requests()`, `labels()`, etc.)
  - Token exchange integration

### Issue Domain

- **[issue-operations.md](./issue-operations.md)** — `IssuesClient`
  - Issue CRUD (`list`, `get`, `create`, `update`)
  - Comments (`list_comments`, `get_comment`, `create_comment`, `update_comment`, `delete_comment`)
  - Label application (`list_labels`, `add_labels`, `remove_label`, `replace_labels`)
  - Reactions on issues and comments (6 methods)
  - Assignees (`list_available_assignees`, `add_assignees`, `remove_assignees`)
  - Lock / unlock
  - Timeline and activity events (`list_timeline`, `list_activity_events`)
  - Milestone assignment (`set_milestone`)

- **[labels-client.md](./labels-client.md)** — `LabelsClient`
  - Repository-level label catalogue (`list`, `get`, `create`, `update`, `delete`)
  - *Applying* labels to issues: use `IssuesClient`; to PRs: use `PullRequestsClient`

- **[milestones-client.md](./milestones-client.md)** — `MilestonesClient`
  - Milestone lifecycle (`list`, `get`, `create`, `update`, `delete`)
  - *Assigning* milestones to issues/PRs: use `IssuesClient::set_milestone` / `PullRequestsClient::set_milestone`

### Pull Request Domain

- **[pull-request-operations.md](./pull-request-operations.md)** — `PullRequestsClient`
  - PR CRUD (`list`, `get`, `create`, `update`, `merge`)
  - Reviews (`list_reviews`, `get_review`, `create_review`, `update_review`, `dismiss_review`)
  - Comments (`list_comments`, `create_comment`, `update_comment`, `delete_comment`)
  - Label application (`add_labels`, `remove_label`)
  - Milestone assignment (`set_milestone`)

### Repository Domain

- **[repository-operations.md](./repository-operations.md)** — `RepositoriesClient`
  - Repository metadata (`get`)
  - Branches (`list_branches`, `get_branch`, `create_branch`)
  - Git references (`get_ref`, `create_ref`, `update_ref`, `delete_ref`)
  - Tags (`list_tags`, `create_tag`)
  - Commits (`get_commit`, `list_commits`, `compare`)

### Additional Domains

- **[project-operations.md](./project-operations.md)** — `ProjectsClient`
  - GitHub Projects V2 (`list_for_org`, `list_for_user`, `get`, `add_item`, `remove_item`, `list_for_issue`)

- **[additional-operations.md](./additional-operations.md)** — `WorkflowsClient` and `ReleasesClient`
  - Workflows: `list`, `get`, `trigger`, `list_runs`, `get_run`, `cancel_run`, `rerun_run`
  - Releases: `list`, `get`, `get_latest`, `get_by_tag`, `create`, `update`, `delete`

- **[reactions.md](./reactions.md)** — `ReactionContent` and `Reaction` types
  - Shared types used by `IssuesClient` reaction methods
  - 8 reaction variants with serde rename constraints

### Infrastructure

- **[pagination.md](./pagination.md)** — Pagination support
  - `PagedResponse<T>` generic type
  - Link header parsing
  - Auto-pagination helper (ADR-002)

- **[rate-limiting-retry.md](./rate-limiting-retry.md)** — Rate limiting and retry logic
  - Installation-level rate tracking
  - Exponential backoff with jitter
  - 429 and 403 handling

## Dependency Graph

```
InstallationClient (foundation)
    │
    ├── issues()         → IssuesClient      (issue.rs)
    ├── labels()         → LabelsClient       (issue.rs)
    ├── milestones()     → MilestonesClient   (issue.rs)
    ├── pull_requests()  → PullRequestsClient (pull_request.rs)
    ├── repositories()   → RepositoriesClient (repository.rs + commit.rs)
    ├── workflows()      → WorkflowsClient    (workflow.rs)
    ├── releases()       → ReleasesClient     (release.rs)
    └── projects()       → ProjectsClient     (project.rs)
                               │
                    PagedResponse<T> (pagination.rs)
                    Rate-limit + retry (rate_limit.rs + retry.rs)
```

## Method Naming Conventions

Methods on sub-clients drop the domain noun from their name because the sub-client
provides the context:

| Old flat API | New sub-client API |
|---|---|
| `client.list_issues(...)` | `client.issues().list(...)` |
| `client.get_issue_comment(...)` | `client.issues().get_comment(...)` |
| `client.list_labels(...)` | `client.labels().list(...)` |
| `client.add_labels_to_issue(...)` | `client.issues().add_labels(...)` |
| `client.get_git_ref(...)` | `client.repositories().get_ref(...)` |
| `client.list_pull_request_reviews(...)` | `client.pull_requests().list_reviews(...)` |
| `client.get_latest_release(...)` | `client.releases().get_latest(...)` |

## ADR References

- **ADR-001**: Hexagonal architecture — why business logic is separated from GitHub API adapters
- **ADR-002**: Auto-pagination strategy — which operations use `Vec<T>` vs `PagedResponse<T>`
- **ADR-003**: Sub-client API pattern — why domain sub-clients instead of flat methods

## Architectural Location

All installation client interfaces are part of the **GitHub Bot SDK** library:

**Layer**: Infrastructure adapters
**Crate**: `github-bot-sdk`
**Module**: `client`
**Dependencies**: `auth` module (for tokens), `error` module

## Type Organization

Types are organized by domain in separate files:

- `installation.rs` - InstallationClient and sub-client factory methods
- `repository.rs` - RepositoriesClient, Repository, Branch, GitRef types
- `commit.rs` - commit operation helpers (part of RepositoriesClient)
- `issue.rs` - IssuesClient, LabelsClient, MilestonesClient, Issue, Comment, Label, Reaction types
- `pull_request.rs` - PullRequestsClient, PullRequest, Review types
- `workflow.rs` - WorkflowsClient, Workflow, WorkflowRun types
- `release.rs` - ReleasesClient, Release types
- `project.rs` - ProjectsClient, ProjectV2 types
- `pagination.rs` - PagedResponse generic type
- `retry.rs` - RetryPolicy and backoff strategies

**Note**: There is no `milestone.rs` file — `MilestonesClient` and all milestone types
live in `issue.rs` (see constraints.md Sub-Client File Placement table).

## Naming Conventions

- **Types**: PascalCase (e.g., `InstallationClient`, `Repository`, `PagedResponse`)
- **Functions**: snake_case (e.g., `get_repository`, `list_branches`)
- **Enums**: PascalCase with PascalCase variants (e.g., `IssueState::Open`)
- **Error types**: Reuse `ApiError` from `error.rs`

## Common Patterns

### Result Types

All operations return `Result<T, ApiError>`:

```rust
pub async fn operation(&self) -> Result<SuccessType, ApiError>
```

### Permission Errors

- `ApiError::PermissionDenied` - 403 responses (insufficient permissions)
- `ApiError::NotFound` - 404 responses (resource not found OR no permission)
- `ApiError::AuthorizationFailed` - authentication token issues

### Request Helpers

All operations use generic request methods:

```rust
self.get(path).await
self.post(path, body).await
self.put(path, body).await
self.delete(path).await
self.patch(path, body).await
```

### Serde Patterns

All API types derive:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

Optional fields use `Option<T>`, with `#[serde(default)]` where appropriate.

## Implementation Guidelines

1. **Read the specification** - Each spec document defines exact signatures
2. **Check shared registry** - Verify types aren't already defined elsewhere
3. **Follow patterns** - Use existing `GitHubClient` and `App` as examples
4. **Test thoroughly** - Mock GitHub API responses with `wiremock`
5. **Document examples** - All public APIs need rustdoc with examples

## Security Considerations

- Never log installation tokens
- All requests use HTTPS (enforced by `reqwest`)
- Validate all external input
- Handle rate limiting to avoid abuse detection
- Respect GitHub's API terms of service

## References

- [GitHub REST API Documentation](https://docs.github.com/en/rest)
- [GitHub App Authentication](https://docs.github.com/en/developers/apps/building-github-apps/authenticating-with-github-apps)
- [Rate Limiting](https://docs.github.com/en/rest/overview/resources-in-the-rest-api#rate-limiting)
