# Edge Cases and Failure Modes

## Overview

This document catalogs non-standard flows, edge cases, and failure modes that the GitHub Bot SDK must handle gracefully. Each case includes the scenario, expected behavior, and recovery strategy.

## Authentication Edge Cases

### Edge Case 1: Private Key Format Variations

**Scenario**: Private keys may be provided in different formats (PKCS#1, PKCS#8, with/without headers).

**Expected Behavior**:

- Accept PEM-encoded RSA private keys in PKCS#1 or PKCS#8 format
- Handle keys with or without `-----BEGIN RSA PRIVATE KEY-----` headers
- Reject invalid key formats with clear error messages

**Error Handling**:

- Return `AuthError::InvalidPrivateKey` with format details
- Never expose key content in error messages
- Suggest format requirements in error message

**Test Cases**:

- Valid PKCS#1 format
- Valid PKCS#8 format
- Key with extra whitespace
- Key with missing headers
- Key with wrong algorithm (e.g., ECDSA)
- Completely invalid data

---

### Edge Case 2: Clock Skew Between Client and GitHub

**Scenario**: System clock is significantly ahead or behind GitHub's servers.

**Impact**:

- JWTs may be rejected as "not yet valid" or "expired"
- Installation tokens may appear expired when still valid
- Rate limit reset times may be misinterpreted

**Mitigation**:

- Accept GitHub's timestamp in responses as authoritative
- Use 5-minute margin for token expiration checks
- Include clock skew tolerance in JWT validation
- Log warnings if detected clock drift > 1 minute

**Recovery**:

- Automatic retry with fresh token if 401 received
- Recommend NTP synchronization in error messages
- Document clock synchronization requirements

---

### Edge Case 3: Token Refresh During Active Operations

**Scenario**: Installation token expires while a long-running operation is in progress.

**Expected Behavior**:

- Operations lasting longer than token lifetime should automatically refresh
- Pagination across multiple requests should handle token refresh
- No operation should fail due to mid-operation token expiration

**Implementation Strategy**:

- Check token expiration before each API call
- Automatically refresh if within expiration margin
- Retry 401 responses once with fresh token
- Document maximum operation time limits

---

### Edge Case 4: Multiple Installations for Same Repository

**Scenario**: Repository has multiple GitHub App installations (rare but possible with organization-level and user-level apps).

**Expected Behavior**:

- Clearly document which installation will be used
- Prefer organization-level installations over user-level
- Allow explicit installation ID override
- Return error if ambiguous and no override provided

**Resolution Strategy**:

- Expose `installation_by_id()` method for explicit selection
- Log warning when multiple installations detected
- Document selection algorithm clearly

---

### Edge Case 5: Installation Suspended or Uninstalled

**Scenario**: GitHub App installation is suspended or uninstalled while bot is running.

**Symptoms**:

- 404 errors when requesting installation tokens
- 403 errors when accessing repository resources
- Cached tokens become invalid

**Expected Behavior**:

- Detect installation unavailability
- Clear cached tokens for that installation
- Return specific error type: `InstallationNotFound` or `InstallationSuspended`
- Do not retry (permanent failure)

**Operational Impact**:

- Webhook deliveries for that installation should be acknowledged but skipped
- Periodic polling of installation status may be needed
- Logs should clearly indicate installation status issue

---

## API Client Edge Cases

### Edge Case 6: Rate Limit Exhaustion

**Scenario**: Bot exhausts its hourly rate limit (5000 requests/hour for GitHub Apps).

**Expected Behavior**:

- Detect approaching rate limit (within margin)
- Begin throttling requests automatically
- Return clear error when limit truly exhausted
- Include reset time in error context

**Recovery Strategy**:

- Wait until rate limit resets
- Implement exponential backoff for retries
- Consider operation priority queuing
- Monitor rate limit usage patterns

**Test Cases**:

- Rate limit remaining: 100 (warn level)
- Rate limit remaining: 0 (block level)
- Rate limit reset time handling
- Secondary rate limit (abuse detection)

---

### Edge Case 7: Secondary Rate Limiting (Abuse Detection)

**Scenario**: GitHub's abuse detection triggers even though primary rate limit not exhausted.

**Symptoms**:

- 403 responses with specific abuse detection message
- May include `Retry-After` header
- Not reflected in standard rate limit headers

**Expected Behavior**:

- Detect secondary rate limiting from response
- Extract `Retry-After` header if present
- Use longer backoff period (1-5 minutes)
- Log as WARNING for monitoring

**Recovery Strategy**:

- Respect `Retry-After` header
- If no header, use exponential backoff starting at 60 seconds
- Do not continue making requests until backoff period passes
- Consider reducing request rate permanently

---

### Edge Case 8: Pagination with Changing Data

**Scenario**: Repository data changes while paginating through results (e.g., new issues created during pagination).

**Expected Behavior**:

- Results may contain duplicates across pages
- Some items may be missed if deleted during pagination
- Pagination links may become stale

**Handling Strategy**:

- Document that pagination is not atomic
- Recommend timestamp-based filtering when available
- Implement deduplication at application level if needed
- Consider GraphQL for atomic queries of large datasets

---

### Edge Case 9: Very Large Responses

**Scenario**: API response is extremely large (e.g., file content, large comment, many pages).

**Expected Behavior**:

- Respect memory limits
- Handle streaming for large responses
- Provide pagination for list endpoints
- Timeout appropriately for slow responses

**Constraints**:

- Default request timeout: 30 seconds
- Configurable timeout for large operations
- Streaming APIs for file content
- Pagination for all list operations

---

### Edge Case 10: Transient Network Failures

**Scenario**: Network connection drops during API request.

**Expected Behavior**:

- Classify as transient error
- Retry with exponential backoff
- Maximum of 3 retry attempts by default
- Use jitter to prevent thundering herd

**Retry Logic**:

- Initial delay: 100ms
- Exponential backoff: 2x multiplier
- Maximum delay: 60 seconds
- Jitter: ±25% randomization

**Non-Retryable Errors**:

- Authentication errors (401)
- Authorization errors (403, non-rate-limit)
- Not found (404)
- Validation errors (422)
- Client errors (4xx generally)

**Retryable Errors**:

- Server errors (500, 502, 503, 504)
- Timeout errors
- Connection errors
- Rate limiting (429, with backoff)

---

## Webhook Processing Edge Cases

### Edge Case 11: Invalid Webhook Signature

**Scenario**: Webhook received with invalid or missing signature.

**Expected Behavior**:

- Reject immediately with 401 Unauthorized
- Log as potential security issue (ERROR level)
- Do not process payload at all
- Include delivery ID in security logs

**Security Implications**:

- May indicate attack attempt
- May indicate misconfigured webhook secret
- Should trigger alerts in production

**Response**:

- HTTP 401 response
- Clear error message (for debugging)
- No payload processing
- Log IP address if available

---

### Edge Case 12: Unknown Event Type

**Scenario**: GitHub sends a webhook for an event type the SDK doesn't recognize.

**Expected Behavior**:

- Accept and validate signature
- Parse envelope metadata
- Store raw payload
- Return success (200) to GitHub
- Log as INFO for monitoring

**Rationale**:

- GitHub adds new event types regularly
- Forward compatibility is important
- Bot applications can decide how to handle
- Prevents webhook delivery retries

---

### Edge Case 13: Malformed JSON Payload

**Scenario**: Webhook payload is not valid JSON or missing required fields.

**Expected Behavior**:

- Signature validation succeeds
- JSON parsing fails
- Return 400 Bad Request to GitHub
- Log payload size and delivery ID
- Do not log full payload (may be malformed/large)

**Recovery**:

- GitHub will retry delivery
- If persistent, check GitHub status
- May indicate GitHub API issue

---

### Edge Case 14: Duplicate Webhook Delivery

**Scenario**: GitHub sends the same webhook multiple times (network retries, GitHub infrastructure).

**Identification**:

- Same `X-GitHub-Delivery` header value
- Same event type and payload

**Expected Behavior**:

- Accept and process first delivery
- Detect duplicate on subsequent deliveries
- Return 200 OK (prevent further retries)
- Skip actual processing (idempotency)
- Log as INFO for monitoring

**Implementation**:

- Track delivery IDs in cache (10-minute TTL)
- Use fast lookup (hash map or Redis set)
- Clear old entries periodically

---

### Edge Case 15: Webhook Order Violation

**Scenario**: Webhooks arrive out of order (e.g., "closed" before "opened" due to network routing).

**Expected Behavior**:

- Accept all webhooks
- Process based on GitHub's event timestamps
- Application layer decides ordering guarantees
- SDK provides event metadata for ordering

**Application Guidance**:

- Use session IDs for related events
- Use event timestamps for ordering
- Handle out-of-order gracefully in business logic
- Consider queue-based ordered processing

---

## Token Caching Edge Cases

### Edge Case 16: Cache Invalidation on Error

**Scenario**: Cached token is rejected by GitHub (401 response) despite not being expired.

**Expected Behavior**:

- Detect 401 response
- Immediately invalidate cached token
- Request fresh token
- Retry original operation once
- If still fails, return error to caller

**Possible Causes**:

- Installation was suspended
- Token was manually revoked
- Clock skew caused premature expiration
- GitHub infrastructure issue

---

### Edge Case 17: Concurrent Token Refresh

**Scenario**: Multiple threads/tasks attempt to refresh the same expired token simultaneously.

**Expected Behavior**:

- Only one refresh request should be made
- Other threads should wait for the refresh to complete
- All threads should receive the same fresh token
- No race conditions or duplicate API calls

**Implementation**:

- Use mutex or async lock around refresh operation
- Check cache again after acquiring lock
- Return cached token if already refreshed

---

### Edge Case 18: Memory Pressure on Cache

**Scenario**: Bot has many installations and cache grows large.

**Expected Behavior**:

- Set maximum cache size
- Implement LRU eviction policy
- Evict expired tokens first
- Monitor cache size and hit rate

**Configuration**:

- Default: Cache up to 1000 installations
- Configurable limit for large bots
- Automatic eviction when limit reached
- Metrics for cache effectiveness

---

## Concurrency Edge Cases

### Edge Case 19: Thundering Herd on Restart

**Scenario**: Bot restarts and all cached tokens are lost, causing simultaneous token requests for many installations.

**Expected Behavior**:

- Rate limit token refresh operations
- Stagger initial token requests
- Use jitter to spread load
- Gracefully handle rate limiting

**Mitigation**:

- Lazy token acquisition (on first use)
- Exponential backoff with jitter
- Circuit breaker for token service
- Pre-warm cache on startup if needed

---

### Edge Case 20: Long-Running Operations During Shutdown

**Scenario**: Bot receives shutdown signal while operations are in progress.

**Expected Behavior**:

- Stop accepting new webhook deliveries
- Allow in-progress operations to complete (with timeout)
- Graceful shutdown period (30-60 seconds)
- Force termination after grace period

**Implementation**:

- Signal in-progress operations via cancellation token
- Drain operation queue
- Close connection pools cleanly
- Flush logs and metrics

---

## GitHub API Quirks

### Edge Case 21: Eventual Consistency in GitHub

**Scenario**: Newly created resource (issue, PR, etc.) is not immediately visible in subsequent API calls.

**Expected Behavior**:

- Document eventual consistency behavior
- Implement retry with exponential backoff for 404s on just-created resources
- Use resource IDs from creation response when possible
- Warn about eventual consistency in SDK documentation

---

### Edge Case 22: GitHub API Deprecation

**Scenario**: GitHub deprecates an API endpoint or changes response format.

**Expected Behavior**:

- SDK should handle both old and new formats during transition
- Log warnings when deprecated endpoints are used
- Provide clear migration path in documentation
- Monitor GitHub API changelog

---

### Edge Case 23: GitHub Status Page Indicates Issues

**Scenario**: GitHub API is experiencing degraded performance or outage.

**Expected Behavior**:

- Increase retry delays
- Activate circuit breakers more aggressively
- Log warnings about known GitHub issues
- Consider checking GitHub status API programmatically

---

## Configuration Edge Cases

### Edge Case 24: Missing Required Configuration

**Scenario**: Bot starts without required configuration (app ID, private key, webhook secret).

**Expected Behavior**:

- Fail fast on initialization
- Clear error message indicating missing configuration
- List all missing required fields
- Do not start serving webhooks with incomplete config

---

### Edge Case 25: Invalid Configuration Values

**Scenario**: Configuration values are present but invalid (e.g., negative timeout, invalid URL).

**Expected Behavior**:

- Validate all configuration on initialization
- Return specific validation errors
- Provide guidance on correct formats
- Use sensible defaults where possible

---

## Recovery Strategies Summary

| Failure Mode | Detection | Recovery | Alert Level |
|--------------|-----------|----------|-------------|
| Invalid private key | Immediate on init | Fix config, restart | CRITICAL |
| Clock skew | JWT rejection (401) | NTP sync, margins | WARNING |
| Token expired mid-op | 401 during request | Auto-refresh, retry | INFO |
| Rate limit exhausted | 429 response | Wait for reset | WARNING |
| Secondary rate limit | 403 with message | Long backoff | WARNING |
| Network failure | Connection error | Retry with backoff | INFO |
| Invalid webhook sig | Validation failure | Reject, log | ERROR |
| Unknown event type | Parse success, unknown type | Accept, log | INFO |
| Duplicate webhook | Same delivery ID | Skip processing | INFO |
| Installation suspended | 404 on token | Invalidate, return error | WARNING |
| GitHub API outage | Multiple 5xx errors | Circuit breaker | CRITICAL |

## Testing Recommendations

Each edge case should have:

1. **Unit test**: Isolated behavior verification
2. **Integration test**: Real GitHub API behavior (using test installations)
3. **Chaos test**: Simulated failure injection
4. **Documentation**: Clear explanation and example

### Priority Testing

- **P0 (Critical)**: Authentication failures, security issues, data loss
- **P1 (High)**: Rate limiting, token refresh, webhook validation
- **P2 (Medium)**: Pagination, concurrency, configuration
- **P3 (Low)**: Rare edge cases, future compatibility

---

## Issue Extended Operations Edge Cases

### Edge Case 17: Auto-Pagination with Token Refresh Mid-List

**Scenario**: Installation token expires after the first page of `list_issue_comments()` is fetched
but before the second page request.

**Expected Behavior**:

- Token refresh logic kicks in silently before the second page request
- The second page request succeeds with the fresh token
- Caller receives the complete comment list as if nothing happened

**Handling Strategy**:

- Check token expiry before each page request in the pagination loop (same as for single requests)
- Do not propagate a 401 to the caller; treat it as a refresh trigger and retry the page once
- Log the token refresh at DEBUG level

**Test Cases**:

- Token expires exactly between page 1 and page 2 requests
- Token refresh itself fails (network error): propagate `ApiError::AuthenticationFailed`

---

### Edge Case 18: Reaction Creates Duplicate for Different User

**Scenario**: User A has reacted with . The bot (a different user) also calls
`create_issue_reaction` with `ReactionContent::PlusOne`.

**Expected Behavior**:

- GitHub creates a new, separate reaction for the bot
- Returns HTTP 201 with the bot's reaction
- Caller receives `Ok(Reaction)` for the bot's new reaction

**Handling Strategy**:

- This is the normal, happy-path case when users and the bot react independently
- The duplicate-reaction idempotency (HTTP 200) only applies to the same authenticated user
- No special handling needed beyond treating both 200 and 201 as `Ok(Reaction)`

---

### Edge Case 19: Issue Locked  Comment Creation Rejected

**Scenario**: Bot attempts `create_issue_comment()` on an issue that has been locked and the bot
does not have collaborator access.

**Impact**:

- GitHub returns HTTP 403
- Bot cannot post the comment

**Expected Behavior**:

- `create_issue_comment()` returns `Err(ApiError::AuthorizationFailed)`
- Error message does not expose the lock reason (GitHub's response handles this)
- Callers should check issue lock status before attempting to comment if this is a concern

**Handling Strategy**:

- Map HTTP 403 to `ApiError::AuthorizationFailed` (existing pattern)
- Callers can inspect `issue.locked` field returned by `get_issue()` before commenting

---

### Edge Case 20: Timeline Unknown Event Types

**Scenario**: GitHub adds a new timeline event type that the SDK does not yet recognise (e.g.,
a new `"sub_issue_added"` event kind introduced in a future GitHub API update).

**Expected Behavior**:

- The new event type deserializes to `TimelineEvent::Unknown` without error
- The caller's result vector contains the `Unknown` variant for that item
- No panic or deserialization failure occurs
- SDK continues functioning for all known event types

**Handling Strategy**:

- The `TimelineEvent` enum uses `#[serde(other)]` or a custom deserializer for the catch-all
- Callers can `match` on variants and use a wildcard `_` arm for `Unknown`
- SDK does not log a warning for unknown types by default (too noisy); callers decide

---

### Edge Case 21: list_issue_activity_events with No Events

**Scenario**: A freshly created issue has no activity events yet (only a "created" record, which
GitHub sometimes omits from the events list on very new issues).

**Expected Behavior**:

- Returns `Ok(vec![])` (empty list, not an error)

---

### Edge Case 22: Assignment of Bot to Issue (Self-Assignment)

**Scenario**: Bot assigns itself using `add_assignees_to_issue()` with its own login.

**Expected Behavior**:

- GitHub Apps can be assigned if the installation has collaborator access
- If the app does not have collaborator access, GitHub silently ignores the login
- Returns `Ok(Issue)` in both cases (GitHub never 422s for ineligible assignees)

**Handling Strategy**:

- No special handling needed; document this behaviour in the method's rustdoc
- Callers can use `list_available_assignees()` to check eligibility before attempting

---

### Edge Case 23: Milestone Due Date in the Past

**Scenario**: `create_milestone()` or `update_milestone()` is called with a `due_on` timestamp
that is already in the past.

**Expected Behavior**:

- GitHub accepts past due dates without error
- Returns `Ok(Milestone)` with the past `due_on` value
- The milestone displays as "overdue" on GitHub's UI, but this is not an API error

**Handling Strategy**:

- No client-side validation of `due_on` in the past (GitHub permits it)
- Document in rustdoc that GitHub accepts past dates

---

### Edge Case 24: Delete Milestone with Associated Open Issues

**Scenario**: `delete_milestone()` is called on a milestone that still has open issues.

**Expected Behavior**:

- GitHub deletes the milestone without error
- The associated issues lose their milestone reference (set to null by GitHub)
- Returns `Ok(())`

**Handling Strategy**:

- No special handling; document this side-effect in the method's rustdoc
- Callers who need to preserve milestone state must migrate issues first
