# ADR-003: Domain Sub-Client Pattern for InstallationClient

Status: Accepted
Date: 2026-04-04
Owners: github-bot-sdk team

## Context

The `InstallationClient` type is a flat collection of every GitHub API operation. At the
time of this decision it contains 65 methods across 8 source files, and the upcoming
issue-area expansion (ADR-002) would add ~20 more, reaching ~85 methods on a single type.

User feedback (and the call pattern expected in issue #39) reveals that callers expect a
grouped, domain-oriented API:

```rust
// What cog_works expected:
sdk.issues().list_comments(owner, repo, id)

// What actually existed:
sdk.list_issue_comments(owner, repo, id)
```

Beyond discoverability, the flat layout has three concrete problems:

1. **Naming tax**: Every method must carry its domain as a prefix to avoid ambiguity
   (`list_issue_comments`, `list_pull_request_comments`) even though the prefix adds
   no information when navigating from a domain-specific context.
2. **Missing prefix inconsistency**: Several methods that *should* have domain prefixes
   don't (e.g. `list_reviews`, `get_review`, `create_review` are PR-only but look generic).
3. **No natural grouping for autocomplete**: Typing `client.` shows all 85+ methods with
   no structure. Typing `client.issues().` shows only issue-domain methods.

## Decision

Introduce **immutable, zero-cost domain sub-clients** returned by factory methods on
`InstallationClient`. Each sub-client groups one cohesive domain of GitHub API operations.

Factory methods are synchronous, return by value, and require no API calls:

```rust
client.issues()        → IssuesClient
client.pull_requests() → PullRequestsClient
client.labels()        → LabelsClient
client.milestones()    → MilestonesClient
client.repositories()  → RepositoriesClient
client.workflows()     → WorkflowsClient
client.releases()      → ReleasesClient
client.projects()      → ProjectsClient
```

**Flat methods within each sub-client (Option A)**: operations remain directly on
the sub-client, not nested further. `client.issues().list_comments(...)` not
`client.issues().comments().list(...)`. This keeps the API one layer deep for call sites
and avoids the cognitive overhead of further nesting.

**`owner`/`repo` stay on every method**: sub-clients are not bound to a specific repository.
This preserves the ability to operate across multiple repositories from a single
`InstallationClient`, which is essential for bots that process events from many repos.

**Labels use a two-client split**:

- `client.labels()` → `LabelsClient` for *repository-level label catalogue* (create, update,
  delete, list, get a label definition).
- `client.issues()` → label application methods (`add_labels`, `remove_label`,
  `replace_labels`, `list_labels`) for attaching/detaching labels on specific issues.
- `client.pull_requests()` → same label application methods for PRs.

This reflects GitHub's own API structure: label definitions live at `/repos/{owner}/{repo}/labels`,
while label application lives at `/repos/{owner}/{repo}/issues/{number}/labels`.

**Method naming within sub-clients**: redundant domain prefixes are dropped because the
sub-client itself carries the domain context.

| Old flat name | New sub-client + method |
|---|---|
| `client.list_issues(...)` | `client.issues().list(...)` |
| `client.list_issue_comments(...)` | `client.issues().list_comments(...)` |
| `client.create_issue_comment(...)` | `client.issues().create_comment(...)` |
| `client.list_reviews(...)` | `client.pull_requests().list_reviews(...)` |
| `client.list_labels(...)` | `client.labels().list(...)` |
| `client.create_label(...)` | `client.labels().create(...)` |
| `client.add_labels_to_issue(...)` | `client.issues().add_labels(...)` |
| `client.add_labels_to_pull_request(...)` | `client.pull_requests().add_labels(...)` |
| `client.list_releases(...)` | `client.releases().list(...)` |
| `client.get_workflow(...)` | `client.workflows().get(...)` |

## Consequences

**Enables:**

- Discoverability: typing `client.issues().` shows only issue-domain operations.
- Concise call sites: no repetitive domain prefixes on every method name.
- Consistent naming: `list`, `get`, `create`, `update`, `delete` at the sub-client level.
- `reviews` are unambiguously PR-only when accessed via `client.pull_requests().list_reviews()`.
- Missing PR comment methods (`update_comment`, `delete_comment`) added as part of this change,
  completing the CRUD symmetry with issue comments.

**Constraints:**

- `InstallationClient` retains its generic HTTP helpers (`get`, `post`, `patch`, `put`,
  `delete`) so sub-clients can delegate to it.
- Sub-clients MUST NOT hold state beyond what is needed to delegate to the parent.
- Sub-clients MUST be cheap to construct (no API calls, no heap allocation beyond an
  `Arc` increment or reference copy).
- Sub-clients MUST be `Clone` and `Debug`.
- The old flat methods on `InstallationClient` are REMOVED (not deprecated alongside the
  new API) — this is a breaking change but the SDK is pre-1.0.

**Trade-offs accepted:**

- One extra `.method()` call at the call site: `client.issues()` wrapping.
- Sub-clients cannot be stored without cloning the `Arc<GitHubClient>` or borrowing
  `InstallationClient`; the interface designer chooses the ownership model.

## Alternatives considered

### Option B: Two levels of nesting for issues (e.g. `client.issues().comments().list()`)

**Why not**: Adds call-site verbosity without significant discoverability benefit over
Option A. The `client.issues()` grouping already scopes operations to the issue domain;
further nesting produces longer chains and more types with no practical payoff.

### Option C: Keep flat, only fix naming consistency

**Why not**: Solves Inconsistency 2 (prefixes) but not the 85-method discoverability
problem. Callers still see all domains mixed together in autocomplete.

### Option D: Keep flat, add doc-comments with `# See Also` groups

**Why not**: Documentation cannot replace API organisation. Autocomplete remains noisy.
This is a non-solution masquerading as one.

### Option E: Repository-bound sub-clients (bind `owner`/`repo` at construction)

**Why not**: Unnecessary rigidity. Bots frequently process events from multiple repositories
within a single installation. Having to construct a new sub-client per repo adds ceremony
for a marginal naming benefit.

## Implementation notes

- Source file organisation does NOT change: `issue.rs`, `pull_request.rs`, etc. each define
  their own sub-client struct and `impl` block. `InstallationClient` factory methods are
  added to `installation.rs` (or a thin `mod.rs` delegating import).
- Tests use the new method names from day one; old tests are updated, not kept alongside.
- The `pub use` re-exports in `client/mod.rs` add the new sub-client types.

## References

- [GitHub issue #39](https://github.com/pvandervelde/github-bot-sdk/issues/39) — `list_comments` gap and expected API shape
- [ADR-002](ADR-002-auto-pagination-strategy.md) — auto-pagination strategy
- API consistency review (2026-04-04 session)
