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

```rust
// Tokens must be handled securely
pub struct SecureString(String);

impl Drop for SecureString {
    fn drop(&mut self) {
        // Zero memory on drop
        self.0.as_mut_vec().fill(0);
    }
}

// No Debug, Display, or Clone for tokens
#[derive(Clone)] // FORBIDDEN for token types
pub struct JwtToken(SecureString);
```

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

```rust
// JWT generation must follow GitHub spec exactly
pub struct JwtClaims {
    pub iss: AppId,           // GitHub App ID
    pub iat: i64,             // Issued at (current time)
    pub exp: i64,             // Expires (max 10 minutes from iat)
}

// Installation tokens have different constraints
pub struct InstallationTokenRequest {
    pub installation_id: InstallationId,
    pub permissions: HashMap<String, String>, // Optional permissions
    pub repositories: Option<Vec<RepositoryId>>, // Optional repo filter
}
```

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

```rust
pub struct WebhookValidator {
    secret: SecureString,
}

impl WebhookValidator {
    // Signature validation must be constant-time
    pub fn validate_signature(&self, payload: &[u8], signature: &str) -> Result<bool, ValidationError> {
        let expected = self.compute_signature(payload);
        // MUST use constant-time comparison
        Ok(constant_time_eq(expected.as_bytes(), signature.as_bytes()))
    }
}
```

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

```rust
pub struct TokenCache {
    // JWT tokens cached until near expiry
    jwt_cache: Arc<RwLock<HashMap<AppId, (JwtToken, Instant)>>>,
    // Installation tokens cached with 5-minute buffer before expiry
    installation_cache: Arc<RwLock<HashMap<InstallationId, (InstallationToken, Instant)>>>,
}
```

## Error Recovery Constraints

### Retry Policies

```rust
pub struct GitHubRetryPolicy {
    pub max_attempts: u32,           // Default: 3
    pub initial_delay: Duration,     // Default: 1s
    pub max_delay: Duration,         // Default: 60s
    pub backoff_multiplier: f64,     // Default: 2.0
    pub jitter: bool,                // Default: true
}
```

### Circuit Breaker

- Circuit opens after 5 consecutive GitHub API failures
- Half-open state after 60 seconds
- Full recovery after 3 successful operations
- Separate circuit breakers for authentication vs. API operations

### Error Classification

```rust
pub enum GitHubError {
    // Retryable errors
    RateLimited { reset_at: Instant },
    ServerError { status: u16 },
    NetworkError { source: reqwest::Error },

    // Non-retryable errors
    AuthenticationFailed,
    PermissionDenied,
    ResourceNotFound,
    ValidationError { field: String, message: String },
}
```

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

```rust
// Required metrics via `metrics` crate
metrics::counter!("github_api_requests_total", "method" => method, "status" => status);
metrics::histogram!("github_api_request_duration", duration);
metrics::gauge!("github_rate_limit_remaining", remaining);
metrics::counter!("github_auth_token_refreshes_total", "type" => token_type);
```

### Tracing

- Support distributed tracing via OpenTelemetry
- Propagate trace context through all async operations
- Include GitHub request IDs in spans for correlation
- Never include sensitive data in trace attributes

## Configuration Constraints

### Environment Configuration

```rust
pub struct GitHubConfig {
    pub app_id: AppId,
    pub private_key_path: PathBuf,          // Path to private key file
    pub webhook_secret: Option<String>,     // For webhook validation
    pub api_base_url: Url,                  // Default: https://api.github.com
    pub user_agent: String,                 // Required by GitHub API
}
```

### Secret Configuration

- Private keys MUST be loaded from files or secure storage
- Webhook secrets MUST be configurable via environment variables
- Configuration MUST support multiple environments (dev, staging, prod)
- Sensitive configuration MUST NOT be logged or serialized

## Deployment Constraints

### Binary Size

- Library MUST compile with minimal feature flags
- Optional features for different authentication methods:

  ```toml
  [features]
  default = ["app-auth"]
  app-auth = ["jsonwebtoken", "rsa"]
  webhook-validation = ["hmac", "sha2"]
  ```

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
