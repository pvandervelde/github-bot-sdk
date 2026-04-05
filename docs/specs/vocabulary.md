# GitHub Bot SDK Domain Vocabulary

This document defines the core concepts and terminology used throughout the github-bot-sdk to ensure consistent understanding across all bot implementations and GitHub integrations.

## GitHub Integration Concepts

### GitHub App

A type of application that can be installed on GitHub organizations and repositories to provide automated functionality.

- **App ID**: Unique numeric identifier assigned by GitHub
- **Private Key**: RSA private key used for signing JWT tokens
- **Permissions**: Specific GitHub API scopes granted to the app
- **Installation**: Specific deployment of the app to an organization or repository
- **Authentication**: Two-step process: JWT for app identity, installation token for API access

### Installation

A specific deployment of a GitHub App to an organization or repository.

- **Installation ID**: Unique numeric identifier for the specific installation
- **Target**: Organization or repository where the app is installed
- **Permissions**: Subset of app permissions granted to this installation
- **Token**: Short-lived access token for API operations within installation scope
- **Events**: Webhook events that the installation should receive

### Installation Token

A short-lived access token that provides API access within a specific installation scope.

- **Scope**: Limited to repositories and permissions of the installation
- **Expiration**: Valid for 1 hour from issuance
- **Refresh**: Must be renewed before expiration using JWT
- **Authentication**: Used in Authorization header for GitHub API requests
- **Caching**: Should be cached until near expiration for efficiency

### JWT (JSON Web Token)

A signed token that authenticates the GitHub App identity for installation token requests.

- **Header**: Specifies RS256 algorithm and token type
- **Payload**: Contains app ID, issued time, and expiration time
- **Signature**: RSA signature using the app's private key
- **Expiration**: Valid for maximum 10 minutes as per GitHub requirements
- **Purpose**: Exchanges for installation tokens via GitHub API

## Event Processing Concepts

### Event Envelope

A standardized container that wraps GitHub webhook events with additional metadata.

- **Event ID**: Unique identifier for deduplication and tracking
- **Event Type**: GitHub event classification (pull_request, issues, etc.)
- **Repository**: Source repository information
- **Entity**: Primary GitHub object affected by the event
- **Session ID**: Grouping identifier for ordered processing
- **Payload**: Original GitHub webhook data
- **Metadata**: Processing timestamps, correlation IDs, routing information

### GitHub Event

A specific type of activity that occurs on GitHub and triggers webhook notifications.

- **Event Types**: pull_request, issues, push, release, check_run, etc.
- **Actions**: Specific activities within event types (opened, closed, synchronized)
- **Payload Structure**: GitHub-defined JSON schema for each event type
- **Headers**: HTTP headers including event type and delivery ID
- **Signature**: HMAC-SHA256 signature for authenticity verification

### Event Parsing

The process of converting raw GitHub webhook payloads into strongly-typed structures.

- **Type Safety**: Rust structures that match GitHub's event schemas
- **Validation**: Ensures required fields are present and correctly typed
- **Error Handling**: Graceful handling of unknown or malformed events
- **Extensibility**: Support for new GitHub event types and fields

### Session Processing

Ordered processing of related events based on GitHub entities.

- **Session Correlation**: Events for same PR/issue share session identifier
- **Sequential Processing**: Events within session processed in chronological order
- **Parallelism**: Different sessions can be processed concurrently
- **State Management**: Tracking entity state across multiple events

## Authentication Concepts

### GitHub App Authentication

The primary authentication mechanism for GitHub API access in bot applications.

- **App-Level**: JWT-based authentication proving app identity
- **Installation-Level**: Token-based authentication for specific installations
- **Two-Phase**: First authenticate as app, then obtain installation token
- **Security**: Private key never transmitted, only signatures

### Private Key

The RSA private key used to sign JWT tokens for GitHub App authentication.

- **Format**: PEM-encoded RSA private key
- **Security**: Must be stored securely and never logged or exposed
- **Usage**: Signs JWT tokens for GitHub API authentication
- **Rotation**: Can be rotated through GitHub App settings
- **Storage**: Environment variables, files, or secure key management services

### Token Caching

The strategy for storing and reusing installation tokens to improve performance.

- **TTL**: Tokens cached until 5 minutes before expiration
- **Keying**: Cached by installation ID for multi-installation bots
- **Invalidation**: Automatic removal of expired tokens
- **Concurrency**: Thread-safe access for concurrent bot operations
- **Fallback**: Fresh token acquisition on cache misses

### Secret Management

Secure handling of sensitive authentication data and configuration.

- **Principles**: Never log secrets, use secure storage, rotate regularly
- **Debug Output**: All secret types redact values in debug output
- **Environment**: Prefer environment variables over hardcoded values
- **Key Vault**: Integration with cloud secret management services
- **Scoping**: Minimum required permissions for each operation

## API Client Concepts

### GitHub API Client

A wrapper around GitHub's REST API that handles authentication and common operations.

- **Authentication**: Automatic injection of installation tokens
- **Rate Limiting**: Built-in respect for GitHub's rate limits
- **Retry Logic**: Automatic retry for transient failures
- **User Agent**: Proper identification for GitHub's request tracking
- **Endpoints**: Strongly-typed methods for common GitHub operations

### Rate Limiting

GitHub's mechanism for controlling API usage and ensuring service stability.

- **Limits**: Different limits for different endpoint categories
- **Headers**: Rate limit information returned in response headers
- **Backoff**: Exponential backoff when approaching limits
- **Reset**: Rate limits reset hourly based on UTC time
- **Monitoring**: Tracking usage to avoid limit exhaustion

### API Operation

A specific interaction with GitHub's API, such as creating comments or updating status.

- **HTTP Method**: GET, POST, PATCH, DELETE for different operations
- **Endpoint**: Specific GitHub API URL path and parameters
- **Request Body**: JSON payload for operations that modify data
- **Response**: GitHub's JSON response with operation results
- **Error Handling**: Proper interpretation of GitHub error responses

### Pagination

GitHub's mechanism for returning large result sets across multiple requests.

- **Page-Based**: Results divided into pages with page numbers and per-page limits
- **Link Headers**: GitHub returns Link header with next/previous page URLs
- **Helper Methods**: SDK utilities for iterating through all pages
- **Limits**: Per-page limits (typically 30-100) and total result set limits
- **Current Implementation**: Page-number based pagination (page, per_page parameters)

## Error Handling Concepts

### GitHub API Error

Errors returned by GitHub's API that indicate various failure conditions.

- **Status Codes**: HTTP status codes indicating error categories
- **Error Messages**: Human-readable descriptions of failures
- **Documentation URLs**: Links to relevant GitHub documentation
- **Rate Limit**: Special handling for rate limit exceeded errors
- **Validation**: Field-specific validation errors for malformed requests

### Authentication Error

Failures related to GitHub App authentication and authorization.

- **Invalid Credentials**: Wrong app ID or corrupted private key
- **Expired Token**: Installation token past its expiration time
- **Insufficient Permissions**: App lacks required permissions for operation
- **Installation Issues**: App not installed or installation suspended
- **Network Issues**: Connectivity problems during authentication

### Transient Error

Temporary failure conditions that may succeed if retried.

- **Network Timeouts**: Request timed out due to network issues
- **Server Errors**: GitHub API returning 5xx status codes
- **Rate Limiting**: Temporary throttling that resets over time
- **Service Unavailability**: GitHub maintenance or capacity issues
- **Retry Strategy**: Exponential backoff with maximum attempt limits

### Permanent Error

Failure conditions that will not succeed regardless of retries.

- **Authorization**: Insufficient permissions for the requested operation
- **Not Found**: Requested resource does not exist
- **Validation**: Malformed request data that will always fail
- **Conflict**: Operation conflicts with current resource state
- **No Retry**: Immediate failure without retry attempts

## Configuration Concepts

### Bot Configuration

Settings that control bot behavior and integration with GitHub services.

- **App Credentials**: GitHub App ID and private key location
- **User Agent**: Identification string for GitHub API requests
- **Timeout Settings**: Network and operation timeout configurations
- **Rate Limit**: Custom rate limiting settings beyond GitHub defaults
- **Environment**: Environment-specific settings and feature flags

### Environment Configuration

Settings that vary between development, staging, and production environments.

- **GitHub API URL**: GitHub.com vs GitHub Enterprise Server endpoints
- **Credential Sources**: Environment variables, files, or secret services
- **Logging Levels**: Different verbosity for different environments
- **Feature Flags**: Enable/disable features per environment
- **Monitoring**: Environment-specific telemetry and monitoring settings

## Testing Concepts

### Mock Authentication

Test doubles that simulate GitHub App authentication without real API calls.

- **Predetermined Responses**: Configurable token responses for tests
- **Error Simulation**: Ability to simulate authentication failures
- **No Network**: Pure in-memory implementation for fast tests
- **Deterministic**: Consistent behavior for reproducible tests

### Test Events

Sample GitHub webhook events used for testing bot behavior.

## Issue Domain Concepts (Extended)

### Issue Comment

A text reply attached to an issue. Key to bot automation because bots post, read, and
search comments to store state, communicate with humans, and implement mutual exclusion.

- **ID**: Numeric comment identifier (unique across the repository, not just the issue)
- **Body**: Markdown text; bots commonly embed structured prefixes for programmatic parsing
- **Author**: Login of the user or bot that created the comment
- **Created At**: UTC timestamp; ascending order (oldest first) is the GitHub default
- **Auto-pagination**: `list_issue_comments()` must fetch all pages to produce a
  correct result for state-reconstruction and lock-detection use cases (see ADR-002)

### Reaction

An emoji acknowledgement attached to an issue or issue comment by a GitHub user.

- **Content**: One of eight emoji: 👍 (`+1`), 👎 (`-1`), 😄 (`laugh`), 😕 (`confused`),
  ❤️ (`heart`), 🎉 (`hooray`), 🚀 (`rocket`), 👀 (`eyes`)
- **Idempotent creation**: GitHub returns the existing reaction (200) if the user has
  already reacted with the same emoji; no duplicate is created.
- **Scope**: Reactions exist independently on issues and on individual comments.
- **Bot use**: Bots use 👀 to signal "I received this" and ✅/❌ (via labels or comments)
  for outcome signalling.

### Issue Lock

A moderation state that prevents non-collaborator comments on an issue.

- **Lock Reason**: Optional; one of `off-topic`, `too-heated`, `resolved`, `spam`
- **Effect**: Only users with write access (collaborators) can comment while locked
- **Reversible**: Can be unlocked at any time by someone with admin or maintain permission
- **Webhook**: Lock/unlock actions appear in the `IssueEvent` webhook and in the
  issue activity event stream

### Assignee

A GitHub user who is responsible for an issue.

- **Eligible users**: Only collaborators, org members with triage access, or repo contributors
  can be assigned. `list_available_assignees()` returns the eligible set.
- **Multiple**: Issues can have zero or more assignees simultaneously.
- **Bot use**: Bots auto-assign issues based on routing rules (area, round-robin, etc.)
- **Idempotent**: Adding an already-assigned user is a no-op (GitHub ignores it silently)

### Milestone

A named target grouping open and closed issues toward a shared goal.

- **Number**: Repository-scoped integer; used in API calls (not the global ID)
- **State**: `open` or `closed`
- **Due On**: Optional UTC timestamp; bots use this for release deadline tracking
- **Issue counts**: GitHub tracks `open_issues` and `closed_issues` on the milestone object
- **Detach**: Setting `milestone: null` on an issue removes it from the milestone

### Issue Activity Event

A discrete historical record of a state change on an issue, returned by the issue events
REST endpoint. Different from a **GitHub Webhook Event** (which is a push notification).

- **Examples**: `labeled`, `unlabeled`, `assigned`, `unassigned`, `milestoned`,
  `demilestoned`, `closed`, `reopened`, `renamed`, `locked`, `unlocked`, `referenced`
- **Ordering**: Ascending chronological order
- **Use case**: Audit trails, historical state reconstruction, debugging bot behaviour
- **Not comments**: Comments do not appear in this stream; use the timeline endpoint for those

### Issue Timeline

A superset of issue activity events that also includes comments, cross-references,
review events, and commit references. Returns a heterogeneous ordered sequence.

- **Endpoint**: `GET /repos/{owner}/{repo}/issues/{issue_number}/timeline`
- **Items**: Mixed types identified by the `event` field on each object
- **Use case**: Complete audit trail, full state reconstruction, cross-reference tracking
- **Unknown kinds**: Unknown event types must deserialize gracefully to a catch-all variant

### IssueCommentEvent (Webhook)

A webhook event fired by GitHub when a comment on an issue is created, edited, or deleted.
Not to be confused with the REST API issue activity stream.

- **Header**: `X-GitHub-Event: issue_comment`
- **Actions**: `created`, `edited`, `deleted`
- **Payload**: Contains both the parent `issue` and the `comment` that changed
- **Distinction**: The `issues` webhook covers issue lifecycle; `issue_comment` covers
  comment lifecycle on issues only (not PR comments, which use `pull_request_review_comment`)

### Auto-Pagination

A pagination strategy where the SDK follows all `Link: rel="next"` headers automatically,
returning a complete `Vec<T>` rather than a page-at-a-time `PagedResponse<T>`.

- **When used**: For operations where callers need the complete set (see ADR-002)
- **Page size**: `per_page=100` (GitHub maximum) to minimise round trips
- **Memory**: All items are held in memory; appropriate for bounded collections
- **Contrast**: `list_issues()` uses manual paging because the caller may stop early

- **Representative**: Cover common GitHub event types and actions
- **Edge Cases**: Include malformed or unusual event structures
- **Complete**: Full event payloads with all relevant fields
- **Versioned**: Match current GitHub webhook schema versions

### Integration Testing

Testing that verifies bot behavior against real or realistic GitHub services.

- **Test Repositories**: Dedicated repositories for integration testing
- **Test Apps**: GitHub Apps specifically for testing purposes
- **Sandbox**: Isolated environment that doesn't affect production
- **Cleanup**: Automatic cleanup of test data and resources

## Observability Concepts

### Distributed Tracing

Tracking requests and operations across multiple services and systems.

- **Trace Context**: W3C standard headers for trace propagation
- **Spans**: Individual operations within a distributed request
- **Correlation**: Links SDK operations with upstream and downstream processing
- **Sampling**: Configurable sampling rates to control overhead

### Structured Logging

Logging that uses consistent, machine-readable formats for better analysis.

- **JSON Format**: Structured log entries with standard fields
- **Context**: Correlation IDs and operation metadata in log entries
- **Levels**: ERROR, WARN, INFO, DEBUG for different severity levels
- **Filtering**: Ability to filter and search logs by structured fields

### Metrics

Quantitative measurements of SDK operations and performance.

- **Counters**: API requests, authentication attempts, errors
- **Gauges**: Active connections, cached tokens, queue depth
- **Histograms**: Request latency, token refresh time, payload size
- **Labels**: GitHub organization, repository, operation type for filtering

## Commit and Repository History Concepts

### Commit

A single point in a repository's history representing a snapshot of the codebase at a specific time.

- **SHA**: 40-character hexadecimal Git hash uniquely identifying the commit
- **Node ID**: GraphQL API identifier for the commit
- **Author**: Git signature (name, email, date) from Git config of who wrote the code
- **Committer**: Git signature of who applied the commit (may differ from author)
- **Message**: Full commit message with subject and body
- **Parents**: List of parent commits (0 for initial, 1 for normal, 2+ for merge)
- **Tree**: SHA of directory snapshot at this commit
- **Verification**: GPG signature verification status (if signed)

### Comparison

The result of comparing two commits, branches, or tags to determine what changed between them.

- **Base**: Starting point for comparison (older ref)
- **Head**: Ending point for comparison (newer ref)
- **Merge Base**: Most recent common ancestor of base and head
- **Status**: Relationship between refs (`ahead`, `behind`, `identical`, `diverged`)
- **Commits**: All commits from base to head in chronological order
- **Files**: All changed files with addition/deletion statistics
- **Use Cases**: Changelog generation, release notes, version comparison

### File Change

A description of how a single file changed in a comparison.

- **Filename**: Current path of the file in the repository
- **Status**: Type of change (`added`, `removed`, `modified`, `renamed`, `copied`)
- **Additions**: Number of lines added
- **Deletions**: Number of lines removed
- **Changes**: Total lines changed (additions + deletions)
- **Previous Filename**: Original path if file was renamed
- **Patch**: Unified diff showing exact changes (may be truncated for large diffs)

## Sub-Client API Concepts (ADR-003)

### Domain Sub-Client

A lightweight struct that groups GitHub API methods for a single business domain
(issues, pull requests, labels, milestones, etc.) and is obtained from `InstallationClient`
via an infallible factory method (`client.issues()`, `client.labels()`, etc.).

- **Construction**: No API call; O(1) cost (at most an `Arc` increment)
- **Lifetime**: Sub-clients can be stored, cloned, and passed across async tasks
- **Scope**: Each sub-client covers exactly one business domain
- **Delegation**: All HTTP calls go through the parent `InstallationClient`'s generic helpers
- **Rationale**: Avoids placing 85+ methods on one type; groups related operations for discoverability

### IssuesClient

The domain sub-client for GitHub issues. Covers issue CRUD, comments, reactions,
label application, assignees, locking, milestone assignment, and timeline queries.

- **Obtained via**: `InstallationClient::issues()`
- **Source file**: `src/client/issue.rs`

### LabelsClient

The domain sub-client for the **repository-level label catalogue**. Manages the
`Label` definitions that exist in a repository (`list`, `get`, `create`, `update`, `delete`).

- **Distinct from**: Applying labels to individual issues (→ `IssuesClient`) or PRs (→ `PullRequestsClient`)
- **Obtained via**: `InstallationClient::labels()`
- **Source file**: `src/client/issue.rs`

### MilestonesClient

The domain sub-client for milestone lifecycle management. Covers `list`, `get`, `create`,
`update`, and `delete` of milestone definitions.

- **Distinct from**: Assigning a milestone to an issue (→ `IssuesClient::set_milestone`) or PR (→ `PullRequestsClient::set_milestone`)
- **Obtained via**: `InstallationClient::milestones()`
- **Source file**: `src/client/issue.rs`

### PullRequestsClient

The domain sub-client for pull requests. Covers PR CRUD, reviews, conversation-thread
comments, label application, and milestone assignment.

- **Obtained via**: `InstallationClient::pull_requests()`
- **Source file**: `src/client/pull_request.rs`

### RepositoriesClient

The domain sub-client for repository metadata, branch management, Git references,
tags, and commit history.

- **Obtained via**: `InstallationClient::repositories()`
- **Source files**: `src/client/repository.rs`, `src/client/commit.rs`

### WorkflowsClient

The domain sub-client for GitHub Actions workflows and runs (`list`, `get`, `trigger`,
`list_runs`, `get_run`, `cancel_run`, `rerun_run`).

- **Obtained via**: `InstallationClient::workflows()`
- **Source file**: `src/client/workflow.rs`

### ReleasesClient

The domain sub-client for GitHub Releases (`list`, `get`, `get_latest`, `get_by_tag`,
`create`, `update`, `delete`).

- **Obtained via**: `InstallationClient::releases()`
- **Source file**: `src/client/release.rs`

### ProjectsClient

The domain sub-client for GitHub Projects V2 (`list_for_org`, `list_for_user`, `get`,
`add_item`, `remove_item`, `list_for_issue`).

- **Obtained via**: `InstallationClient::projects()`
- **Source file**: `src/client/project.rs`

### Git Signature

An identity record in Git representing who performed an action and when.

- **Name**: From Git config `user.name`
- **Email**: From Git config `user.email`
- **Date**: Timestamp when action occurred
- **GitHub Association**: If email matches a GitHub user, parent Commit includes user object
- **Use Cases**: Distinguishing author from committer, audit trails, contributor attribution

### Verification

The status of a commit's GPG signature validation.

- **Verified**: Boolean indicating if signature is valid
- **Reason**: Why verified/unverified (`valid`, `invalid`, `expired_key`, `unknown_key`, `unsigned`)
- **Signature**: GPG signature payload (if present)
- **Payload**: Signed content (if present)
- **Use Cases**: Security compliance, audit trails, trust verification

This vocabulary establishes the shared language for github-bot-sdk architecture and implementation, ensuring consistent terminology across all bot implementations and GitHub integrations.
