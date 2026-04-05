# GitHub Bot SDK Behavioral Assertions

## Overview

This document defines testable behavioral assertions for the GitHub Bot SDK. These assertions verify that authentication, API operations, and event processing work correctly and securely according to GitHub's requirements and best practices.

## Authentication Assertions

### Assertion 1: JWT Token Generation

**Given**: A valid GitHub App ID and private key
**When**: `generate_jwt_token()` is called
**Then**: Operation returns `Ok(JwtToken)` with valid JWT
**And**: Token expires within 10 minutes (GitHub requirement)
**And**: Token contains correct `iss` claim matching App ID

**Test Criteria**:

- JWT structure is valid (header.payload.signature)
- `iss` claim matches provided App ID
- `iat` claim is current timestamp (±5 seconds)
- `exp` claim is `iat + duration` (max 10 minutes)
- Token validates against provided private key

### Assertion 2: App-Level API Operations

**Given**: A valid JWT token
**When**: App-level operations are called (e.g., `list_installations()`, `get_app()`)
**Then**: Operations use JWT directly in Authorization header
**And**: GitHub API accepts the JWT authentication
**And**: Operations return app-level data (not installation-scoped)

**Test Criteria**:

- Authorization header contains `Bearer <JWT>`
- Requests go to app-level endpoints (`/app`, `/app/installations`)
- Responses contain app-level information
- No installation token is used or cached

### Assertion 3: JWT Token with Invalid Private Key

**Given**: A GitHub App ID and malformed private key
**When**: `generate_jwt_token()` is called
**Then**: Operation returns `Err(AuthenticationError::InvalidPrivateKey)`
**And**: No token is generated
**And**: Error message does not expose private key content

**Test Criteria**:

- Error type is specifically `InvalidPrivateKey`
- Private key content never appears in error message
- No partial token generation occurs

### Assertion 4: Installation Token Retrieval

**Given**: A valid JWT token and installation ID
**When**: `get_installation_token()` is called
**Then**: Operation returns `Ok(InstallationToken)` with GitHub API response
**And**: Token has valid expiry time from GitHub
**And**: Token is cached for subsequent requests

**Test Criteria**:

- JWT is used to authenticate the token exchange request
- Installation token format matches GitHub specification
- Expiry time is parsed correctly from GitHub response (1 hour from GitHub)
- Subsequent calls within cache period return cached token
- Cache respects token expiry times

### Assertion 5: Installation-Level API Operations

**Given**: A valid installation token for installation ID 12345
**When**: Installation-level operations are called (e.g., create issue, list PRs)
**Then**: Operations use installation token in Authorization header
**And**: GitHub API accepts the installation token authentication
**And**: Operations are scoped to repositories within the installation

**Test Criteria**:

- Authorization header contains `Bearer <installation_token>` or `token <installation_token>`
- Requests go to installation-scoped endpoints (`/repos/{owner}/{repo}/*`)
- Operations succeed only for repositories within the installation
- Operations fail with 404 for repositories outside the installation scope

### Assertion 6: Installation Token for Non-Existent Installation

**Given**: A valid JWT token and non-existent installation ID
**When**: `get_installation_token()` is called
**Then**: Operation returns `Err(GitHubError::InstallationNotFound)`
**And**: No token is cached
**And**: Error includes installation ID for debugging

**Test Criteria**:

- Error type is specifically `InstallationNotFound`
- Installation ID included in error context
- No cache pollution occurs

### Assertion 7: Token Cache Expiry Handling

**Given**: A cached installation token that has expired
**When**: Operations requiring authentication are performed
**Then**: New token is automatically requested from GitHub
**And**: Operations succeed with fresh token
**And**: Cache is updated with new token

**Test Criteria**:

- Expired tokens trigger automatic refresh
- Refresh happens transparently to caller
- New token replaces expired token in cache
- Cache timestamps updated correctly

## API Operations Assertions

### Assertion 8: Repository Information Retrieval

**Given**: Valid authentication and existing repository
**When**: `get_repository()` is called with repository ID
**Then**: Operation returns `Ok(Repository)` with complete metadata
**And**: Response includes owner, name, permissions, and settings

**Test Criteria**:

- Repository data matches GitHub API response format
- All required fields are populated
- Permissions reflect actual GitHub App installation permissions
- Data types match specification

### Assertion 9: Repository Access Without Permission

**Given**: Valid authentication but repository not in installation scope
**When**: `get_repository()` is called
**Then**: Operation returns `Err(GitHubError::PermissionDenied)`
**And**: Error clearly indicates permission issue
**And**: No partial data is returned

**Test Criteria**:

- Error type is specifically `PermissionDenied`
- Error message is actionable for troubleshooting
- Security context preserved (no data leaks)

### Assertion 10: Pull Request Creation

**Given**: Valid authentication and repository with write permissions
**When**: `create_pull_request()` is called with valid PR data
**Then**: Operation returns `Ok(PullRequest)` with GitHub-assigned ID
**And**: PR is visible in GitHub repository
**And**: All specified metadata is set correctly

**Test Criteria**:

- PR ID is valid GitHub-assigned identifier
- Title, body, base, and head match request
- Labels, assignees, and reviewers applied correctly
- PR state is "open" initially

### Assertion 11: Pull Request Creation Without Write Permission

**Given**: Valid authentication but repository with read-only permissions
**When**: `create_pull_request()` is called
**Then**: Operation returns `Err(GitHubError::PermissionDenied)`
**And**: No PR is created in repository
**And**: Error indicates specific permission needed

**Test Criteria**:

- Error type is specifically `PermissionDenied`
- Error indicates "write" permission requirement
- No partial PR creation occurs

### Assertion 12: Issue Management Operations

**Given**: Valid authentication and repository with appropriate permissions
**When**: Issue operations are performed (create, update, close)
**Then**: Operations succeed and modify GitHub repository state
**And**: Issue state changes are immediately visible
**And**: All metadata updates are preserved

**Test Criteria**:

- Issue creation returns valid issue ID
- Updates modify exact fields specified
- State transitions (open/closed) work correctly
- Comments, labels, and assignments persist

## Rate Limiting Assertions

### Assertion 13: Rate Limit Respect

**Given**: GitHub API client approaching rate limits
**When**: Multiple API requests are made rapidly
**Then**: Client automatically throttles requests
**And**: Rate limit headers are monitored and respected
**And**: Operations eventually succeed without rate limit violations

**Test Criteria**:

- No HTTP 429 (rate limited) responses from GitHub
- Request delays increase as rate limit approaches
- Rate limit headers parsed correctly
- Operations resume after rate limit reset

### Assertion 14: Rate Limit Exceeded Handling

**Given**: GitHub API returning HTTP 429 rate limit exceeded
**When**: Additional API requests are attempted
**Then**: Client implements exponential backoff with jitter
**And**: Requests are retried after appropriate delay
**And**: Operations eventually succeed when rate limit resets

**Test Criteria**:

- Exponential backoff with jitter implemented
- Retry delays respect `Retry-After` header when present
- Maximum retry attempts honored
- Success after rate limit window expires

### Assertion 15: Secondary Rate Limit Handling

**Given**: GitHub API returning secondary rate limit (abuse detection)
**When**: Client receives HTTP 403 with rate limit message
**Then**: Client implements longer backoff period
**And**: Aggressive retry patterns are avoided
**And**: Operations resume after cooldown period

**Test Criteria**:

- Secondary rate limits detected correctly
- Longer backoff periods applied (60+ seconds)
- No aggressive retry behavior
- Graceful degradation during limits

## Webhook Processing Assertions

### Assertion 16: Webhook Signature Validation

**Given**: Valid webhook payload and matching signature
**When**: `validate_webhook()` is called
**Then**: Operation returns `Ok(true)` indicating valid signature
**And**: Payload content is verified as authentic
**And**: Validation uses constant-time comparison

**Test Criteria**:

- HMAC-SHA256 signature computed correctly
- Signature comparison is constant-time (timing attack resistant)
- Payload integrity confirmed
- No timing side channels in validation

### Assertion 17: Webhook Signature Validation Failure

**Given**: Webhook payload with invalid or tampered signature
**When**: `validate_webhook()` is called
**Then**: Operation returns `Ok(false)` indicating invalid signature
**And**: Payload is rejected as potentially malicious
**And**: Validation timing is consistent regardless of failure type

**Test Criteria**:

- Invalid signatures correctly rejected
- Tampered payloads detected
- Consistent timing prevents timing attacks
- No information leakage about signature validation

### Assertion 18: Webhook Event Processing

**Given**: Valid authenticated webhook with supported event type
**When**: `process_webhook_event()` is called
**Then**: Event is routed to appropriate handler
**And**: Event processing is idempotent
**And**: Handler receives correctly parsed event data

**Test Criteria**:

- Event type routing works correctly
- Duplicate events (same ID) handled idempotently
- Event data parsed according to GitHub schema
- Handler context includes authentication information

### Assertion 19: Webhook Event Deduplication

**Given**: Multiple webhook deliveries with same event ID
**When**: Events are processed sequentially
**Then**: Only first event is processed completely
**And**: Subsequent events are recognized as duplicates
**And**: No duplicate processing side effects occur

**Test Criteria**:

- Event ID tracking prevents duplicates
- First processing completes normally
- Duplicate detection is immediate
- No resource waste on duplicate processing

## Error Handling and Recovery Assertions

### Assertion 20: Network Connectivity Failure

**Given**: GitHub API client when network connectivity is lost
**When**: API operations are attempted
**Then**: Operations return `Err(GitHubError::NetworkError)`
**And**: Client can recover when connectivity is restored
**And**: Retry logic handles transient failures

**Test Criteria**:

- Network errors classified correctly
- Automatic retry with exponential backoff
- Recovery after connectivity restoration
- Circuit breaker prevents cascading failures

### Assertion 21: GitHub API Server Errors

**Given**: GitHub API returning HTTP 5xx server errors
**When**: API operations are attempted
**Then**: Operations are retried with backoff
**And**: Eventually succeed when GitHub recovers
**And**: Circuit breaker protects against sustained failures

**Test Criteria**:

- Server errors trigger retry logic
- Exponential backoff between retries
- Circuit breaker opens after sustained failures
- Operations succeed after GitHub recovery

### Assertion 22: Authentication Token Expiry During Operations

**Given**: Long-running operations with expiring tokens
**When**: Token expires during operation sequence
**Then**: New tokens are automatically obtained
**And**: Operations continue seamlessly
**And**: No authentication errors surface to caller

**Test Criteria**:

- Token expiry detected automatically
- Refresh happens transparently
- Operations resume with new token
- No user-visible authentication interruptions

## Security Assertions

### Assertion 23: Private Key Security

**Given**: GitHub App private key loaded into memory
**When**: Private key is used for JWT generation
**Then**: Private key never appears in logs or error messages
**And**: Memory is securely cleared after use
**And**: Key material is not accessible through debugging

**Test Criteria**:

- No private key content in any logs
- Memory zeroed after cryptographic operations
- Debug output excludes sensitive data
- Error messages don't leak key information

### Assertion 24: Token Security in Transit

**Given**: Authentication tokens being used for API requests
**When**: HTTP requests are made to GitHub API
**Then**: Tokens are transmitted only over HTTPS
**And**: TLS certificate validation is enforced
**And**: Tokens are included only in Authorization headers

**Test Criteria**:

- All requests use HTTPS protocol
- Certificate validation is strict
- No tokens in URLs or query parameters
- Authorization headers properly formatted

### Assertion 25: Sensitive Data Logging Prevention

**Given**: SDK operations involving authentication or API data
**When**: Logging occurs at any level
**Then**: No sensitive data appears in log output
**And**: Structured logging maintains security boundaries
**And**: Debug information excludes secrets

**Test Criteria**:

- No tokens, keys, or passwords in logs
- API request/response bodies sanitized
- Personal data (emails, names) redacted appropriately
- Debug tracing excludes sensitive context

## Performance and Scalability Assertions

### Assertion 26: Concurrent API Operations

**Given**: Multiple concurrent GitHub API requests
**When**: Operations execute simultaneously
**Then**: All operations complete successfully
**And**: Connection pooling is utilized efficiently
**And**: No race conditions in token management

**Test Criteria**:

- Thread safety across all operations
- HTTP connection reuse working
- Token cache handles concurrent access
- Performance scales with concurrency

### Assertion 27: Memory Usage Under Load

**Given**: High-volume API operations over extended time
**When**: Continuous operations are performed
**Then**: Memory usage remains bounded
**And**: No memory leaks in token or connection management
**And**: Garbage collection is efficient

**Test Criteria**:

- Memory usage stabilizes under constant load
- No unbounded cache growth
- Connection pools respect size limits
- Long-running operations don't accumulate memory

## Commit Operations Assertions

### Assertion 28: Get Commit by SHA Returns Full Details

**Given**: Valid repository with known commit SHA
**When**: `get_commit(owner, repo, sha)` is called
**Then**: Returns `Ok(Commit)` with complete details
**And**: Commit contains author/committer signatures
**And**: Commit contains message and parent references
**And**: Commit has correct SHA matching request

**Test Criteria**:

- SHA matches requested value
- Commit message is non-empty
- Author and committer signatures present
- Parent commits array exists (may be empty for initial commit)
- HTML URL is valid

### Assertion 29: Get Commit by Branch Resolves to HEAD

**Given**: Repository with branch "main" at commit "abc123"
**When**: `get_commit(owner, repo, "main")` is called
**Then**: Returns commit for branch HEAD
**And**: Commit SHA is "abc123"
**And**: Same result as calling with SHA directly

**Test Criteria**:

- Branch name resolves to current HEAD commit
- Result matches direct SHA lookup

### Assertion 30: Get Non-Existent Commit Returns NotFound

**Given**: Valid repository
**When**: `get_commit(owner, repo, "nonexistent-sha")` is called
**Then**: Returns `Err(ApiError::NotFound)`
**And**: Error includes context for debugging
**And**: No authentication tokens in error

**Test Criteria**:

- Error type is NotFound
- Error message provides context
- No sensitive data leaked

### Assertion 31: List Commits Returns Chronological Order

**Given**: Repository with multiple commits
**When**: `list_commits(owner, repo, None, None, ...)` is called
**Then**: Returns commits in reverse chronological order (newest first)
**And**: Each commit is newer than the next in array
**And**: Maximum 30 commits returned (default page size)

**Test Criteria**:

- Commits sorted by date descending
- Dates verified in order
- Default pagination applied

### Assertion 32: List Commits with Path Filter Shows Only Relevant Commits

**Given**: Repository where 10 commits modified "README.md" out of 100 total
**When**: `list_commits(owner, repo, None, Some("README.md"), ...)` is called
**Then**: Returns approximately 10 commits (not 100)
**And**: Each commit modified the specified path

**Test Criteria**:

- Result count much less than total commits
- Path filter applied correctly

### Assertion 33: Compare Identical Refs Returns Identical Status

**Given**: Tag "v1.0.0" pointing to specific commit
**When**: `compare_commits(owner, repo, "v1.0.0", "v1.0.0")` is called
**Then**: Returns comparison with status "identical"
**And**: ahead_by = 0, behind_by = 0
**And**: Commits array is empty
**And**: Files array is empty

**Test Criteria**:

- Status is "identical"
- No commits in result
- No file changes

### Assertion 34: Compare Tags Shows Commits Between Releases

**Given**: Tag "v1.0.0" is 5 commits behind "v1.1.0"
**When**: `compare_commits(owner, repo, "v1.0.0", "v1.1.0")` is called
**Then**: Returns comparison with status "ahead"
**And**: ahead_by = 5, total_commits = 5
**And**: Commits array has 5 commits in chronological order
**And**: Files array shows all changed files

**Test Criteria**:

- Correct commit count
- Commits in order
- File changes included
- Status correctly shows "ahead"

### Assertion 35: Comparison Includes File Changes with Statistics

**Given**: Comparison between two refs with file changes
**When**: `compare_commits(owner, repo, base, head)` is called
**Then**: Files array includes all changed files
**And**: Each file has status (added/removed/modified/renamed)
**And**: Each file has additions/deletions counts
**And**: Renamed files include previous_filename

**Test Criteria**:

- All file statuses present
- Addition/deletion counts accurate
- Renamed files properly marked

### Assertion 36: List Commits on Empty Repository Returns InvalidRequest

**Given**: Newly created repository with no commits
**When**: `list_commits(owner, repo, ...)` is called
**Then**: Returns `Err(ApiError::InvalidRequest)`
**And**: GitHub API returns 422 status

**Test Criteria**:

- Error type is InvalidRequest
- Handles empty repository gracefully

### Assertion 37: Commit Operations Complete Within Performance Targets

**Given**: Normal network conditions
**When**: Commit operations are called
**Then**: get_commit completes in <200ms (p95)
**And**: list_commits completes in <500ms (p95)
**And**: compare_commits completes in <1000ms (p95)

**Test Criteria**:

- Performance targets met
- Single API call per operation
- No redundant requests

---

## Issue Extended Operations Assertions

### Assertion 38: list_issue_comments Returns All Pages

**Given**: An issue with 150 comments (two pages of 100 + 50)
**When**: `list_issue_comments(owner, repo, issue_number)` is called
**Then**: Returns `Ok(Vec<Comment>)` with all 150 comments
**And**: Comments are in ascending `created_at` order (oldest first)
**And**: Exactly two HTTP requests were made (one per page)

**Test Criteria**:

- Result length is 150
- Timestamps are non-decreasing across the full result
- First request uses `per_page=100`; second follows `Link: rel="next"` URL verbatim
- No third HTTP request is made after the last page

### Assertion 39: list_issue_comments on Non-Existent Issue

**Given**: An issue number that does not exist in the repository
**When**: `list_issue_comments(owner, repo, 9999999)` is called
**Then**: Returns `Err(ApiError::NotFound)`
**And**: Error propagates immediately without further page requests

**Test Criteria**:

- Error type is `ApiError::NotFound`
- Exactly one HTTP request was made
- No partial results returned

### Assertion 40: list_issue_comments on Empty Issue

**Given**: An issue with zero comments
**When**: `list_issue_comments(owner, repo, issue_number)` is called
**Then**: Returns `Ok(vec![])`

**Test Criteria**:

- Result is an empty vector, not an error
- Exactly one HTTP request was made

### Assertion 41: add_assignees_to_issue with Eligible Users

**Given**: Users "alice" and "bob" are collaborators in the repository
**When**: `add_assignees_to_issue(owner, repo, 1, vec!["alice", "bob"])` is called
**Then**: Returns `Ok(Issue)` with both users in the `assignees` field
**And**: A POST to `/repos/{owner}/{repo}/issues/1/assignees` was made

**Test Criteria**:

- Updated issue contains both assignees
- Body sent was `{ "assignees": ["alice", "bob"] }`

### Assertion 42: add_assignees_to_issue with Ineligible User (Silent Ignore)

**Given**: User "external-user" is not a collaborator
**When**: `add_assignees_to_issue(owner, repo, 1, vec!["external-user"])` is called
**Then**: Returns `Ok(Issue)` with no change to the assignee list
**And**: GitHub silently ignores the ineligible user (no 422)

**Test Criteria**:

- Returns `Ok(Issue)` (not an error)
- Assignee list unchanged in returned issue

### Assertion 43: remove_assignees_from_issue

**Given**: Issue 1 is currently assigned to "alice"
**When**: `remove_assignees_from_issue(owner, repo, 1, vec!["alice"])` is called
**Then**: Returns `Ok(Issue)` with empty assignees
**And**: A DELETE to `/repos/{owner}/{repo}/issues/1/assignees` was made with the body

**Test Criteria**:

- Returned issue has empty assignees list
- HTTP method is DELETE (not POST); body contains `{ "assignees": ["alice"] }`

### Assertion 44: lock_issue with Reason

**Given**: An open, unlocked issue
**When**: `lock_issue(owner, repo, 42, Some(LockReason::TooHeated))` is called
**Then**: Returns `Ok(())`
**And**: A PUT to `/repos/{owner}/{repo}/issues/42/lock` was made
**And**: Request body contained `{ "lock_reason": "too heated" }`

**Test Criteria**:

- HTTP status 204 treated as success
- `lock_reason` serialised as `"too heated"` (kebab-case)

### Assertion 45: lock_issue without Reason

**Given**: An open, unlocked issue
**When**: `lock_issue(owner, repo, 42, None)` is called
**Then**: Returns `Ok(())`
**And**: Request body is empty (or omits the `lock_reason` field)

**Test Criteria**:

- No `lock_reason` key in JSON body when reason is `None`

### Assertion 46: unlock_issue

**Given**: A locked issue
**When**: `unlock_issue(owner, repo, 42)` is called
**Then**: Returns `Ok(())`
**And**: A DELETE to `/repos/{owner}/{repo}/issues/42/lock` was made

**Test Criteria**:

- HTTP status 204 treated as success
- No body required

### Assertion 47: replace_labels_on_issue Atomically Replaces All Labels

**Given**: Issue 3 currently has labels ["bug", "help-wanted"]
**When**: `replace_labels_on_issue(owner, repo, 3, vec!["enhancement".into()])` is called
**Then**: Returns `Ok(Vec<Label>)` containing only "enhancement"
**And**: "bug" and "help-wanted" are no longer on the issue

**Test Criteria**:

- Result contains exactly one label: "enhancement"
- A PUT (not POST) request was made
- Body was `{ "labels": ["enhancement"] }`

### Assertion 48: replace_labels_on_issue with Empty List Clears All Labels

**Given**: Issue 3 with existing labels
**When**: `replace_labels_on_issue(owner, repo, 3, vec![])` is called
**Then**: Returns `Ok(vec![])`
**And**: All labels removed

**Test Criteria**:

- Body sent was `{ "labels": [] }`
- Result is empty vector

### Assertion 49: create_issue_reaction Returns Existing on Duplicate

**Given**: The authenticated bot has already reacted with 👀 on issue 1
**When**: `create_issue_reaction(owner, repo, 1, ReactionContent::Eyes)` is called again
**Then**: Returns `Ok(Reaction)` (not an error)
**And**: GitHub returns HTTP 200 (not 201) for the duplicate

**Test Criteria**:

- Both HTTP 200 and HTTP 201 from GitHub result in `Ok(Reaction)`
- The returned `Reaction` has the same `id` as the original

### Assertion 50: create_issue_reaction on Non-Existent Issue

**Given**: Issue number that does not exist
**When**: `create_issue_reaction(owner, repo, 9999, ReactionContent::PlusOne)` is called
**Then**: Returns `Err(ApiError::NotFound)`

**Test Criteria**:

- Error type is `ApiError::NotFound`

### Assertion 51: delete_issue_reaction Succeeds

**Given**: A known reaction ID 42 on issue 1
**When**: `delete_issue_reaction(owner, repo, 1, 42)` is called
**Then**: Returns `Ok(())`
**And**: A DELETE to `/repos/{owner}/{repo}/issues/1/reactions/42` was made

**Test Criteria**:

- HTTP 204 treated as success

### Assertion 52: list_issue_reactions Auto-Paginates

**Given**: Issue with 150 reactions across two pages
**When**: `list_issue_reactions(owner, repo, issue_number)` is called
**Then**: Returns all 150 reactions
**And**: Two HTTP requests were made following `Link: rel="next"`

**Test Criteria**:

- Result length is 150
- Two requests made

### Assertion 53: Comment reactions follow identical contract to issue reactions

All assertions from 49–52 apply symmetrically to:

- `create_comment_reaction()`
- `delete_comment_reaction()`
- `list_comment_reactions()`

with the endpoint swapped to `/repos/{owner}/{repo}/issues/comments/{comment_id}/reactions`.

**Test Criteria**:

- Identical success and error behaviour
- Correct endpoint used (`issues/comments/{id}` not `issues/{number}`)

### Assertion 54: create_milestone Returns Created Milestone

**Given**: Repository with `issues:write` permission
**When**: `create_milestone(owner, repo, CreateMilestoneRequest { title: "v2.0", .. })` is called
**Then**: Returns `Ok(Milestone)` with a GitHub-assigned `number`
**And**: Milestone `title` matches the request

**Test Criteria**:

- HTTP 201 treated as success
- `number` is non-zero

### Assertion 55: create_milestone with Duplicate Title Returns InvalidRequest

**Given**: Milestone "v2.0" already exists in the repository
**When**: `create_milestone(owner, repo, ...)` is called with title "v2.0"
**Then**: Returns `Err(ApiError::InvalidRequest { .. })`

**Test Criteria**:

- GitHub returns HTTP 422; maps to `ApiError::InvalidRequest`

### Assertion 56: delete_milestone Succeeds

**Given**: Milestone number 3 exists
**When**: `delete_milestone(owner, repo, 3)` is called
**Then**: Returns `Ok(())`
**And**: HTTP 204 was returned

**Test Criteria**:

- Milestone is removed
- Associated issues retain their milestone field until manually cleared

### Assertion 57: list_issue_activity_events Returns All Events

**Given**: Issue 5 with 120 recorded activity events (two pages)
**When**: `list_issue_activity_events(owner, repo, 5)` is called
**Then**: Returns all 120 events in ascending chronological order
**And**: Two HTTP requests were made

**Test Criteria**:

- Result length is 120
- Events sorted oldest-first
- Second request follows `Link: rel="next"` verbatim

### Assertion 58: list_issue_timeline Returns Mixed Event Types Without Error

**Given**: Issue with a mix of commented, labeled, closed, and referenced events
**When**: `list_issue_timeline(owner, repo, issue_number)` is called
**Then**: Returns `Ok(Vec<TimelineEvent>)` with all items
**And**: Each item deserializes to the correct `TimelineEvent` variant
**And**: Unknown event types deserialize to `TimelineEvent::Unknown` without panicking

**Test Criteria**:

- No deserialization errors
- Commented variant contains body text
- Labeled variant contains label name
- Unknown variant does not cause an `Err`

### Assertion 59: IssueCommentEvent Webhook Deserializes Correctly

**Given**: Raw `issue_comment` webhook payload with `action: "created"`
**When**: `serde_json::from_value::<IssueCommentEvent>(payload)` is called
**Then**: Returns `Ok(IssueCommentEvent)` with correct fields
**And**: `action` is `IssueCommentAction::Created`
**And**: `issue` contains the parent issue metadata
**And**: `comment` contains the new comment body

**Test Criteria**:

- All three action variants (`created`, `edited`, `deleted`) deserialize correctly
- `issue.number` matches the issue number in the payload
- `comment.body` contains the comment text
