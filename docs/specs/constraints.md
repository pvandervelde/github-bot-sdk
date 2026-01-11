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
