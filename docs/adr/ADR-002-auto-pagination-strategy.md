# ADR-002: Auto-Pagination Strategy for Issue Comment Listing

Status: Accepted
Date: 2026-04-04
Owners: github-bot-sdk team

## Context

The SDK exposes two conceptually similar list operations that serve different caller needs:

1. **`list_issues()`** – callers may want to scroll through a large repository's issues page-by-page,
   filtering interactively, and stopping early once they have what they need.
2. **`list_issue_comments()`** – callers (e.g. `cog_works`) must read _all_ comments to find the
   most recent state snapshot or detect a lock prefix. Stopping mid-page yields an incorrect result.

GitHub returns paginated results for both endpoints (up to 100 items per page via `per_page=100`),
signalling further pages via `Link: <url>; rel="next"` headers as per RFC 5988.

The existing `list_issue_comments()` only fetches the first page, making it unusable for the
cog_works pipeline-state and processing-lock use cases described in issue #39.

Two competing patterns already exist in the SDK:

| Pattern | Return type | Pagination control | Use case |
|---------|------------|-------------------|----------|
| Manual paging | `PagedResponse<T>` | Caller decides when to stop | Large lists, early termination |
| Auto-pagination | `Vec<T>` | SDK follows all pages | Always-complete lists |

## Decision

Use **auto-pagination** (returning `Vec<T>`) for operations where **callers need the complete set**
to make a correct decision, and **manual paging** (`PagedResponse<T>`) for operations where callers
may legitimately stop early.

**Criteria for auto-pagination:**

- The calling use case requires inspecting _all_ items (not a subset).
- Returning a partial list produces incorrect behaviour in the caller.
- The total volume is bounded in practice (issue comments rarely exceed thousands).

**Criteria for manual paging:**

- Callers may want to stop early (e.g. "find the first open issue assigned to me").
- The collection can be very large (e.g. all repository issues).
- The caller needs to resume from a known page (e.g. cursor-based processing).

**Applied to this SDK:**

| Method | Return | Rationale |
|--------|--------|-----------|
| `list_issues()` | `PagedResponse<Issue>` | May stop early; large repos have thousands |
| `list_issue_comments()` | `Vec<Comment>` | Must read all; use cases require complete list |
| `list_labels()` | `Vec<Label>` | Small set, complete view always needed |
| `list_milestones()` | `Vec<Milestone>` | Small set, complete view always needed |
| `list_available_assignees()` | `Vec<IssueUser>` | Small set, complete view always needed |
| `list_issue_reactions()` | `Vec<Reaction>` | Small set, complete view always needed |
| `list_comment_reactions()` | `Vec<Reaction>` | Small set, complete view always needed |
| `list_issue_activity_events()` | `Vec<IssueActivityEvent>` | Audit / state reconstruction needs all |
| `list_issue_timeline()` | `Vec<TimelineEvent>` | Audit / state reconstruction needs all |

**Auto-pagination implementation rule:**

```text
loop:
  GET endpoint with per_page=100
  collect items
  if Link header contains rel="next": follow URL
  else: break
return collected items
```

Use `per_page=100` (GitHub's maximum) on the first request to minimise round trips.
Follow the exact URL from the `Link: rel="next"` header for subsequent pages — do not
reconstruct URLs manually, as GitHub may include additional state in the URL.

## Consequences

**Enables:**

- Callers get a correct, complete result without pagination boilerplate.
- Matches the contract expected by `cog_works` (issue #39).
- API surface is simpler for the "complete list" pattern.

**Forbids:**

- Auto-paginating list operations cannot return early once started.
- Memory is bounded only by total comment/event count (not a page limit).

**Trade-offs accepted:**

- A very large comment thread (thousands of comments) will load all into memory.
  This is acceptable: such threads are extremely rare and the SDK is a library, not a
  long-running service that accumulates data.
- `list_issues()` retains `PagedResponse<Issue>` and is NOT changed to auto-paginate,
  because repository issue counts routinely reach tens of thousands.

## Alternatives considered

### Option A: Always return `PagedResponse<T>`, let caller paginate

**Why not**: Callers must implement pagination loops themselves. For the state-reconstruction
use case, partial reads produce incorrect results (wrong state, stale lock). Code duplication
across every caller.

### Option B: Auto-paginate all list operations uniformly

**Why not**: `list_issues()` on a large repository would fetch thousands of objects and take
many seconds. Auto-pagination is appropriate where the collection is bounded, not for open-ended
repository-level lists.

### Option C: Add a separate `list_all_issue_comments()` alongside the existing paged version

**Why not**: Adds API noise. The paged version currently returns only page 1 and has no utility
in its current form. Replacing it with auto-pagination is a breaking change, but the method did
not fulfil its contract anyway.

## Implementation notes

- The pagination loop must propagate any `ApiError` immediately; do not swallow errors mid-page.
- Add `per_page=100` as a query parameter on the initial request.
- For subsequent pages, use the URL from the `Link` header verbatim (call `self.get_url(&url)`
  rather than `self.get(&path)`).
- Existing `parse_link_header()` utility in `client/mod.rs` extracts the `next` URL; reuse it.

## Examples

```rust
// Caller code – no pagination boilerplate needed
let comments = client
    .list_issue_comments("owner", "repo", 42)
    .await?;

// Find most recent state snapshot
let snapshot = comments
    .iter()
    .rev()
    .find(|c| c.body.starts_with("<!-- pipeline-state:"))
    .map(|c| serde_json::from_str::<PipelineState>(&c.body[20..]));
```

## References

- [GitHub issue #39](https://github.com/pvandervelde/github-bot-sdk/issues/39)
- [GitHub REST API Link headers](https://docs.github.com/en/rest/guides/using-pagination-in-the-rest-api)
- [RFC 5988 – Web Linking](https://datatracker.ietf.org/doc/html/rfc5988)
