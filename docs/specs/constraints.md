# GitHub Bot SDK Implementation Constraints

## Overview

This document defines the implementation rules and architectural boundaries that must be enforced when implementing the GitHub Bot SDK. These constraints ensure secure GitHub App integration, proper authentication handling, and consistent API interaction patterns.

## Type System Constraints

### Branded Types Required

All GitHub domain identifiers MUST use branded types (newtype pattern) to prevent accidental misuse:

- GitHub App IDs must be distinct from Installation IDs
- Installation IDs must be distinct from Repository IDs
- Repository IDs must be distinct from Issue/PR IDs
- Authentication tokens must be opaque types

**Rationale**: Prevents accidentally using wrong ID type in function calls, caught at compile time.

**Interface Designer**: Will define concrete type structures following this constraint.

### Error Handling

- All GitHub operations MUST return `Result<T, GitHubError>`
- Never use `panic!` in library code - all errors must be recoverable
- GitHub API errors MUST be mapped to structured error types
- Include rate limit information in error context when applicable
- Authentication errors MUST NOT leak sensitive information

### Async Constraints

- All I/O operations MUST be async and cancellable via `CancellationToken`
- Use `tokio` as the async runtime (no `async-std` compatibility needed)
- All HTTP timeouts MUST be configurable and respect cancellation
- Token refresh operations MUST be non-blocking with async retry

## Module Boundary Constraints

### Logical Dependency Rules

**Critical**: These constraints define WHAT can depend on WHAT, not WHERE code lives.

- **Domain logic** NEVER imports infrastructure implementations
- **Domain logic** depends ONLY on abstraction interfaces (traits/ports)
- **Abstraction interfaces** defined with or near domain logic
- **Infrastructure implementations** import and implement interfaces
- **Authentication code** NEVER logs secrets or tokens
- **Business logic modules** NEVER import HTTP libraries directly
- **Business logic modules** NEVER import secret management SDKs directly

**Physical organization is the Interface Designer's responsibility** - they will determine actual file structure following Rust conventions and business domain naming.

## Security Constraints

### Authentication Token Handling

**Security Requirements**:

- Token types MUST zero memory on drop (implement Drop trait)
- Token types MAY implement Clone if wrapped in Arc for shared ownership
- Token types MUST implement custom Debug that redacts token values
- Token types MUST NOT implement Display (prevents string conversion)
- Token storage must use secure string wrappers

**Rationale**: Custom Debug implementation allows safe debugging while preventing accidental token exposure. Clone is permitted when needed for shared ownership patterns (e.g., concurrent access), but the implementation must maintain security guarantees.

### Secret Management

- Private keys MUST be loaded from secure storage (Azure Key Vault, etc.)
- Private keys NEVER appear in logs or error messages
- JWT tokens MUST expire within 10 minutes (GitHub requirement)
- Installation tokens MUST be cached securely with proper expiry
- All cryptographic operations use constant-time comparisons

### Network Security

- MUST validate GitHub API TLS certificates
- MUST use HTTPS for all GitHub API communications
- Support corporate proxy configurations
- Webhook signature validation MUST use constant-time comparison
- Rate limit headers MUST be respected to prevent API abuse

## GitHub API Constraints

### Authentication Flow

**JWT Requirements**:

- Claims MUST include: issuer (App ID), issued-at time, expiration time
- Expiration MUST be maximum 10 minutes from issue time (GitHub requirement)
- Use RS256 algorithm (RSA SHA-256) for signing
- Never exceed 10-minute maximum expiration

**Installation Token Requirements**:

- Requests MUST specify installation ID
- MAY optionally specify permissions subset
- MAY optionally specify repository subset
- Responses contain token string and expiration time

### Rate Limiting

- MUST respect GitHub's rate limits (5000 requests/hour for apps)
- Implement exponential backoff when rate limited
- Cache API responses where appropriate to reduce API calls
- Support secondary rate limits (abuse detection)
- Monitor rate limit headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

### API Versioning

- Use GitHub API version 2022-11-28 (latest stable)
- All requests MUST include `Accept: application/vnd.github+json` header
- Support API deprecation notices gracefully
- Version compatibility testing for API changes

## Event Processing Constraints

### Webhook Validation

**Security Requirements**:

- Signature validation MUST use constant-time comparison (prevents timing attacks)
- Webhook secret MUST be stored as secure string (zeroed on drop)
- Use HMAC-SHA256 for signature computation
- Validate signature BEFORE any payload processing
- Return generic validation error (don't leak signature details)

**Algorithm**:

1. Compute HMAC-SHA256 of payload using webhook secret
2. Compare computed signature with provided signature using constant-time equality
3. Return success/failure without exposing computed signature

### Event Processing

- Event handlers MUST be idempotent (GitHub may retry webhooks)
- Event IDs MUST be tracked to prevent duplicate processing
- Large webhook payloads (>1MB) MUST be handled efficiently
- Event processing MUST complete within reasonable time limits
- Failed event processing MUST not crash the entire application

## Performance Constraints

### Response Times

- JWT generation: <50ms
- Installation token retrieval: <500ms (including cache check)
- API requests: <2000ms (including retries)
- Webhook validation: <100ms

### Throughput

- Support minimum 100 concurrent API operations
- Token cache MUST handle high concurrent access
- Connection pooling for HTTP clients
- Efficient memory usage for large API responses

### Caching Strategy

**Cache Requirements**:

- JWT tokens cached until near expiry (recommend 1-minute buffer)
- Installation tokens cached with 5-minute buffer before expiry
- Cache keyed by App ID for JWTs, Installation ID for installation tokens
- Thread-safe access (use appropriate synchronization primitives)
- Automatic eviction of expired entries
- Cache hit/miss metrics for monitoring

**Cache Invalidation**:

- Tokens evicted automatically before expiration (proactive refresh)
- Manual invalidation on authentication errors
- Clear all tokens on shutdown

## Error Recovery Constraints

### Retry Policies

**Retry Configuration**:

- Maximum retry attempts: 3 (default, should be configurable)
- Initial retry delay: 1 second (default)
- Maximum retry delay: 60 seconds (default)
- Backoff multiplier: 2.0 (exponential backoff)
- Jitter: Enabled (prevents thundering herd)

**Retry Rules**:

- Only retry transient errors (5xx, network failures, timeouts)
- Never retry authentication failures (401)
- Never retry authorization failures (403, non-rate-limit)
- Never retry validation errors (422)
- Respect Retry-After header when present

### Circuit Breaker

- Circuit opens after 5 consecutive GitHub API failures
- Half-open state after 60 seconds
- Full recovery after 3 successful operations
- Separate circuit breakers for authentication vs. API operations

### Error Classification

**Retryable Errors** (transient failures):

- Rate limiting (429) - retry after rate limit reset
- Server errors (500, 502, 503, 504) - retry with exponential backoff
- Network failures (connection timeouts, DNS failures) - retry with backoff
- Request timeouts - retry with backoff

**Non-Retryable Errors** (permanent failures):

- Authentication failures (401) - fix credentials, don't retry
- Authorization failures (403, non-rate-limit) - fix permissions
- Not found (404) - resource doesn't exist
- Validation errors (422) - fix request data
- Client errors (4xx generally) - fix request, don't retry

**Error Context Requirements**:

- Include operation context for debugging
- Include correlation/trace ID
- Never include sensitive data (tokens, keys)
- Include recovery suggestions when applicable

## Testing Constraints

### Unit Testing

- Authentication logic MUST have 100% test coverage
- Use test doubles for all GitHub API interactions
- Mock time-dependent operations (JWT expiry, etc.)
- Test error scenarios thoroughly

### Integration Testing

- Test against GitHub API test endpoints where available
- Use GitHub Apps in test mode for integration tests
- Clean up test resources after execution
- Test rate limiting and retry behavior

### Security Testing

- Verify tokens are never logged or exposed
- Test signature validation with malicious payloads
- Verify constant-time operations in security-critical code
- Test for timing attacks in authentication flows

## Observability Constraints

### Logging

- Use structured logging via `tracing` crate
- Log levels:
  - `ERROR`: Authentication failures, unrecoverable API errors
  - `WARN`: Rate limiting, retry attempts, token expiry warnings
  - `INFO`: Successful operations, token refresh events
  - `DEBUG`: API request/response details (NO sensitive data)
  - `TRACE`: Flow control, detailed timing information

### Metrics

**Required Metrics** (using `metrics` crate or compatible):

- Counter: `github_api_requests_total` (labels: method, status)
- Histogram: `github_api_request_duration` (in milliseconds)
- Gauge: `github_rate_limit_remaining` (current quota)
- Counter: `github_auth_token_refreshes_total` (labels: token_type)
- Counter: `github_webhook_signatures_validated` (labels: result)
- Counter: `github_errors_total` (labels: error_type, operation)

### Tracing

- Support distributed tracing via OpenTelemetry
- Propagate trace context through all async operations
- Include GitHub request IDs in spans for correlation
- Never include sensitive data in trace attributes

## Configuration Constraints

### Environment Configuration

**Required Configuration**:

- GitHub App ID (numeric identifier)
- Private key (PEM format, from file or secret management)
- API base URL (default: <https://api.github.com>)
- User agent string (required by GitHub API)

**Optional Configuration**:

- Webhook secret (for signature validation)
- Request timeout (default: 30 seconds)
- Maximum retries (default: 3)
- Rate limit margin (default: 0.1 = 10%)
- Token cache TTL overrides

### Secret Configuration

- Private keys MUST be loaded from files or secure storage
- Webhook secrets MUST be configurable via environment variables
- Configuration MUST support multiple environments (dev, staging, prod)
- Sensitive configuration MUST NOT be logged or serialized

## Deployment Constraints

### Binary Size

- Library MUST compile with minimal feature flags
- Optional features for different authentication methods
- Feature flags should enable/disable major functionality groups
- Default features should cover common use cases
- Example feature categories: app-auth, webhook-validation, tracing

### Runtime Dependencies

- Minimal runtime dependencies to reduce attack surface
- Use well-maintained, security-audited crates
- Pin dependency versions to prevent supply chain attacks
- Regular security updates for dependencies

## Documentation Constraints

### API Documentation

- All public APIs MUST have rustdoc comments with examples
- Include GitHub API documentation links where relevant
- Document rate limiting behavior and error conditions
- Provide security best practices in documentation

### Security Documentation

- Document secure configuration practices
- Provide examples of proper secret management
- Include security considerations for deployment
- Document webhook security requirements

### Examples

- Provide complete working examples for GitHub App setup
- Include webhook processing examples
- Show proper error handling patterns
- Demonstrate authentication flow variations

## Commit Operations Constraints

### Module Placement

**File Location**: `src/client/commit.rs`

**Pattern**: Infrastructure adapter layer following same structure as `repository.rs`, `release.rs`, `pull_request.rs`

**Rationale**: Commit operations are installation-scoped GitHub API calls, consistent with other client operations

### Type Definitions

**Required Derivations**: All commit types MUST derive `Debug, Clone, Serialize, Deserialize`

**Types to Create**:

- `Commit` - Full commit with GitHub metadata
- `CommitDetails` - Git-level metadata (author, committer, message, tree)
- `GitSignature` - Identity record (name, email, date)
- `CommitReference` - Minimal pointer (SHA + URL)
- `Verification` - GPG signature verification status
- `Comparison` - Comparison result between two refs
- `FileChange` - File diff in comparison

**Type Reuse**: `IssueUser` for author/committer GitHub user associations

### Operation Signatures

All operations MUST return `Result<T, ApiError>` and be async:

- `get_commit(owner, repo, sha) -> Result<Commit, ApiError>`
- `list_commits(owner, repo, filters...) -> Result<Vec<Commit>, ApiError>`
- `compare_commits(owner, repo, base, head) -> Result<Comparison, ApiError>`

### GitHub API Endpoints

MUST use exact endpoints:

1. `GET /repos/{owner}/{repo}/commits/{ref}` - get_commit
2. `GET /repos/{owner}/{repo}/commits?params` - list_commits
3. `GET /repos/{owner}/{repo}/compare/{base}...{head}` - compare_commits

### Error Mapping

| Status | ApiError | Scenario |
|--------|----------|----------|
| 404 | NotFound | Commit/ref/repo not found |
| 422 | InvalidRequest | Invalid SHA, empty repo |
| 403 | AuthorizationFailed | Insufficient permissions |
| 401 | AuthenticationFailed | Token expired |
| 429 | RateLimitExceeded | Rate limit hit |

### Pagination and Performance

- Default: 30 items/page, Max: 100 items/page (GitHub limit)
- Performance targets (p95): get_commit <200ms, list_commits <500ms, compare_commits <1000ms
- Single API call per operation

### Security

Error messages MUST include context but NEVER include tokens, keys, or sensitive data

### Testing Coverage

- Unit tests: Type deserialization, error mapping, ordering
- Integration tests: Real API calls to public repos
- Target: >90% coverage

### Documentation

All public types/methods MUST have complete rustdoc with examples, parameters, errors, and links

---

## Issue Extended Operations Constraints

### Auto-Pagination

- `issues().list_comments()` MUST auto-paginate using `per_page=100` and follow all `Link: rel="next"` headers (ADR-002)
- All other auto-paginating list methods (reactions, activity events, timeline, milestones, available assignees) MUST follow the same loop pattern
- Auto-pagination MUST propagate any `ApiError` immediately on a page fetch failure — do not return a partial list
- Subsequent page URLs MUST be used verbatim from the `Link` header — do not reconstruct URLs manually

### Reactions

- `issues().create_reaction()` and `issues().create_comment_reaction()` MUST treat both HTTP 200 and HTTP 201 responses as `Ok(Reaction)`
- `ReactionContent::PlusOne` and `ReactionContent::MinusOne` MUST serialize to `"+1"` and `"-1"` respectively (GitHub API names)

### Issue Locking

- `issues().lock()` MUST use HTTP PUT to `/repos/{owner}/{repo}/issues/{number}/lock`
- `issues().unlock()` MUST use HTTP DELETE to the same path
- Both methods treat HTTP 204 as success
- `LockReason` MUST serialize with kebab-case (`off-topic`, `too-heated`, `resolved`, `spam`)

### Assignees

- `issues().remove_assignees()` MUST use HTTP DELETE with a JSON body containing `{ "assignees": [...] }` — GitHub requires a body on this DELETE
- Implementations MUST NOT validate assignee eligibility client-side; GitHub's silent ignore is the authoritative behaviour

### Timeline Deserialization

- `TimelineEvent` MUST deserialize unknown event kinds to a catch-all variant without returning `Err`
- The implementation MUST NOT `panic!` on unknown timeline event types
- The catch-all variant MAY discard fields from unknown types (no requirement to preserve raw JSON unless specified later)

### IssueCommentEvent

- `IssueCommentEvent` lives in `src/events/github_events.rs` alongside `IssueEvent` and `PullRequestEvent`
- `IssueCommentEvent::comment` MUST use the same `Comment` type from `src/client/issue.rs`
- `IssueCommentEvent::issue` MUST use the same `Issue` type from `src/client/issue.rs`

### Milestone CRUD

- `milestones().get()`, `milestones().list()`, `milestones().create()`, `milestones().update()`, `milestones().delete()` live in `src/client/issue.rs` alongside the existing `Milestone` type and `issues().set_milestone()`
- There is NO separate `milestone.rs` file — the note in `additional-operations.md` predates the decision to consolidate in `issue.rs`

---

## Sub-Client API Constraints (ADR-003)

### Sub-Client Construction

- All factory methods on `InstallationClient` MUST be infallible (`&self -> SubClient`, no `Result`)
- All factory methods MUST be O(1) and make NO API calls during construction
- Sub-clients MUST be `Clone` and `Debug`
- Sub-clients MUST NOT hold any mutable state

### Sub-Client Delegation

- All HTTP operations in sub-clients MUST delegate to `InstallationClient`'s internal helpers
- Sub-clients MUST NOT create their own HTTP clients or manage their own tokens
- Sub-clients hold a clone of (or a reference to) the parent `InstallationClient` so that
  token caching and rate limiting remain centralised

### Sub-Client File Placement

| Sub-Client | Source file |
|---|---|
| `IssuesClient` | `src/client/issue.rs` |
| `LabelsClient` | `src/client/issue.rs` |
| `MilestonesClient` | `src/client/issue.rs` |
| `PullRequestsClient` | `src/client/pull_request.rs` |
| `RepositoriesClient` | `src/client/repository.rs` (+ commit.rs for commit methods) |
| `WorkflowsClient` | `src/client/workflow.rs` |
| `ReleasesClient` | `src/client/release.rs` |
| `ProjectsClient` | `src/client/project.rs` |

### Label Responsibility Split

- **Repo-level label catalogue** (create/update/delete label definitions): `LabelsClient`
- **Applying labels to an issue**: `IssuesClient::add_labels`, `IssuesClient::remove_label`, `IssuesClient::replace_labels`
- **Applying labels to a PR**: `PullRequestsClient::add_labels`, `PullRequestsClient::remove_label`

### Method Naming on Sub-Clients

- Methods on sub-clients MUST NOT repeat the domain noun in their name because the
  sub-client already provides domain context
  - CORRECT: `client.issues().list(...)` / `client.labels().create(...)`
  - WRONG: `client.issues().list_issues(...)` / `client.labels().create_label(...)`
- Exception: compound methods where the noun disambiguates the sub-resource are allowed
  - `client.issues().list_comments(...)`, `client.issues().list_reactions(...)`,
    `client.pull_requests().list_reviews(...)` — all acceptable because the noun names
    the resource, not the domain

### Review Method Naming on PullRequestsClient

- Review methods MUST retain the `_review` / `_reviews` suffix to avoid ambiguity
  with `list_comments` and other comment methods
  - `list_reviews`, `get_review`, `create_review`, `update_review`, `dismiss_review`

### Pagination Policy (ADR-002)

| Operation | Strategy | Rationale |
|---|---|---|
| `issues().list_comments` | Auto-paginated | Must be complete for state reconstruction |
| `labels().list` | Auto-paginated | Bounded catalogue set |
| `milestones().list` | Auto-paginated | Bounded set per repo |
| `issues().list_reactions` / `list_comment_reactions` | Auto-paginated | Bounded per resource |
| `issues().list_available_assignees` | Auto-paginated | Bounded collaborator set |
| `issues().list_activity_events` | Auto-paginated | Bounded per issue |
| `issues().list_timeline` | Auto-paginated | Must be complete for audit |
| `issues().list_labels` | Auto-paginated | Bounded per issue |
| `pull_requests().list_reviews` | Auto-paginated | Bounded per PR |
| `repositories().list_branches` | Auto-paginated | Bounded per repo |
| `repositories().list_tags` | Auto-paginated | Bounded per repo |
| `issues().list` | Manual (`PagedResponse<T>`) | Caller may stop early; unbounded in large repos |
| `pull_requests().list` | Manual | Unbounded |
| `repositories().list_commits` | Manual | Arbitrarily large history |

### Backward Compatibility

- The old flat `InstallationClient` methods (`list_issues`, `get_issue_comment`, etc.) MUST be removed
- There is NO compatibility shim; this is a clean API replacement before the 1.0 release
- The change is tracked in CHANGELOG.md as a breaking change
