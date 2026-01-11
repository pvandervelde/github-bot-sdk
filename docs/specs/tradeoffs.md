# Architectural Tradeoffs and Decisions

## Overview

This document captures the key architectural decisions made for the GitHub Bot SDK, including alternatives considered, tradeoffs analyzed, and rationale for choices made.

## Decision 1: Two-Level Authentication Model

### Decision

Separate app-level authentication (JWT) from installation-level authentication (installation tokens).

### Alternatives Considered

1. **Single token type**: Use only installation tokens
2. **Two-level separation**: JWT for app ops, installation tokens for repository ops (chosen)
3. **Personal Access Tokens (PAT)**: Simpler but less secure

### Analysis

**Single Token Type**:

- ✅ Simpler API surface
- ✅ Fewer concepts to learn
- ❌ Cannot perform app-level operations (list installations, etc.)
- ❌ Doesn't match GitHub's actual authentication model
- ❌ Forces workarounds for discovery operations

**Two-Level Separation (Chosen)**:

- ✅ Matches GitHub's authentication model exactly
- ✅ Clear separation of concerns
- ✅ Enables app-level operations (installation discovery)
- ✅ Proper scoping of permissions
- ❌ More complex API
- ❌ Users must understand both token types

**Personal Access Tokens**:

- ✅ Simplest to implement
- ❌ Security risk (user-scoped, not app-scoped)
- ❌ No fine-grained permissions
- ❌ Not suitable for production bots
- ❌ Token rotation is manual

### Rationale

Chose two-level authentication to **properly model GitHub's security architecture**. The additional complexity is justified by security benefits and alignment with GitHub's intended usage patterns.

---

## Decision 2: Trait-Based Abstraction for External Dependencies

### Decision

Use Rust traits to abstract authentication, HTTP clients, and secret providers.

### Alternatives Considered

1. **Concrete implementations only**: No abstraction layer
2. **Trait-based abstraction**: Interface definitions with multiple implementations (chosen)
3. **Dynamic plugins**: Runtime-loaded plugins

### Analysis

**Concrete Implementations**:

- ✅ Simplest implementation
- ✅ No indirection overhead
- ✅ Easier to understand for beginners
- ❌ Impossible to test without real GitHub API
- ❌ Cannot swap secret providers (Azure KV vs AWS Secrets)
- ❌ Tight coupling to specific services
- ❌ No flexibility for different deployments

**Trait-Based Abstraction (Chosen)**:

- ✅ Testable with mock implementations
- ✅ Supports multiple secret providers
- ✅ Clean dependency inversion
- ✅ Zero runtime cost (monomorphization)
- ✅ Clear contracts for implementers
- ❌ More complex type signatures
- ❌ Requires understanding of traits
- ❌ Additional compile time

**Dynamic Plugins**:

- ✅ Runtime flexibility
- ✅ Can add providers without recompilation
- ❌ Runtime overhead (dynamic dispatch)
- ❌ More complex error handling
- ❌ Version compatibility challenges
- ❌ Unnecessary for this use case

### Rationale

Chose trait-based abstraction for **testability and deployment flexibility** without runtime overhead. The SDK needs to work with different secret providers (Azure Key Vault, AWS Secrets Manager, environment variables) and must be thoroughly testable.

---

## Decision 3: Token Caching Strategy

### Decision

Implement automatic token caching with proactive refresh (5 minutes before expiration).

### Alternatives Considered

1. **No caching**: Request fresh tokens for every operation
2. **Cache until expiration**: Hold tokens until they expire
3. **Proactive refresh**: Refresh 5 minutes before expiration (chosen)
4. **Lazy refresh**: Only refresh when token is rejected

### Analysis

**No Caching**:

- ✅ Simplest implementation
- ✅ No stale token issues
- ❌ Excessive API calls (rate limiting issues)
- ❌ Poor performance (latency on every operation)
- ❌ Wastes GitHub's rate limits

**Cache Until Expiration**:

- ✅ Good performance
- ✅ Reduced API calls
- ❌ Operations can fail mid-expiration
- ❌ Clock skew issues between client and GitHub
- ❌ Sudden failures when token expires

**Proactive Refresh (Chosen)**:

- ✅ Excellent performance
- ✅ No sudden failures from expiration
- ✅ Handles clock skew gracefully
- ✅ Reduces API calls significantly
- ❌ Slightly more complex logic
- ❌ May refresh unused tokens

**Lazy Refresh**:

- ✅ Only refreshes when needed
- ✅ Handles expiration automatically
- ❌ Operations fail first, then retry (poor UX)
- ❌ Increased latency on first failure
- ❌ Complicates error handling

### Rationale

Chose proactive refresh for **reliability and performance**. The 5-minute margin handles clock skew and prevents operations from failing due to token expiration. This is critical for long-running bots that need high availability.

---

## Decision 4: Synchronous vs Async API

### Decision

Use fully async API with tokio runtime.

### Alternatives Considered

1. **Synchronous blocking API**: Simple but blocking
2. **Fully async API**: All operations async (chosen)
3. **Hybrid API**: Both sync and async versions

### Analysis

**Synchronous Blocking**:

- ✅ Simplest to implement and use
- ✅ Easier error handling
- ✅ No runtime complexity
- ❌ Blocks threads during I/O
- ❌ Cannot handle high concurrency
- ❌ Poor fit for webhook processing
- ❌ Doesn't match Rust ecosystem trends

**Fully Async (Chosen)**:

- ✅ Excellent concurrency and throughput
- ✅ Non-blocking I/O
- ✅ Matches tokio ecosystem
- ✅ Suitable for webhook servers
- ✅ Can handle thousands of concurrent operations
- ❌ More complex API surface
- ❌ Requires understanding of async/await
- ❌ Async trait complexity

**Hybrid API**:

- ✅ Supports both use cases
- ✅ Flexibility for users
- ❌ Double the API surface to maintain
- ❌ Confusing for users (which to use?)
- ❌ Sync version just blocks on async anyway
- ❌ Maintenance burden

### Rationale

Chose fully async to support **high-throughput bot operations** and webhook processing. Modern Rust bot infrastructure is async-first, and blocking alternatives would limit scalability. The complexity tradeoff is acceptable for the performance benefits.

---

## Decision 5: Error Handling Strategy

### Decision

Use `Result<T, E>` with structured error types, no panics in library code.

### Alternatives Considered

1. **Panic on errors**: Simplest but unsafe
2. **`Result` with structured errors**: Explicit error handling (chosen)
3. **Exception-like error propagation**: Using unwrap/expect
4. **Error codes**: C-style integer error codes

### Analysis

**Panic on Errors**:

- ✅ Simplest implementation
- ❌ Crashes entire application
- ❌ No recovery possible
- ❌ Unacceptable for library code
- ❌ Violates Rust best practices

**Result with Structured Errors (Chosen)**:

- ✅ Explicit error handling
- ✅ Caller controls recovery strategy
- ✅ Type-safe error propagation
- ✅ Can distinguish transient vs permanent errors
- ✅ Integration with `?` operator
- ❌ More verbose
- ❌ Requires error type design

**Exception-like Propagation**:

- ✅ Looks simpler at call site
- ❌ Hides error conditions
- ❌ Can panic unexpectedly
- ❌ No type safety
- ❌ Bad practice for libraries

**Error Codes**:

- ✅ Simple to implement
- ❌ Not idiomatic in Rust
- ❌ No type safety
- ❌ Easy to ignore errors
- ❌ No context information

### Rationale

Chose `Result` with structured errors for **reliability and explicit error handling**. Library code must never panic, and callers need clear information about error types to make retry decisions. This aligns with Rust best practices and enables robust error handling in bot implementations.

---

## Decision 6: Rate Limiting Strategy

### Decision

Implement proactive rate limiting with margin-based throttling and exponential backoff.

### Alternatives Considered

1. **No rate limiting**: Let GitHub reject requests
2. **Reactive rate limiting**: Only back off after 429
3. **Proactive with margin**: Throttle before hitting limits (chosen)
4. **Fixed delay between requests**: Conservative but slow

### Analysis

**No Rate Limiting**:

- ✅ Simplest implementation
- ❌ Wastes requests on rejections
- ❌ Poor user experience (failures)
- ❌ Risk of abuse detection
- ❌ Damages rate limit budget

**Reactive Rate Limiting**:

- ✅ Uses full rate limit budget
- ✅ Simple implementation
- ❌ Some requests will fail
- ❌ Requires retry logic
- ❌ Impacts latency on failures

**Proactive with Margin (Chosen)**:

- ✅ Prevents rate limit errors
- ✅ Smooth operation without failures
- ✅ Respects GitHub's infrastructure
- ✅ Handles secondary rate limits
- ❌ Doesn't use full rate limit budget
- ❌ More complex implementation

**Fixed Delay**:

- ✅ Guaranteed to stay under limits
- ✅ Simple implementation
- ❌ Too conservative (wastes time)
- ❌ Poor throughput
- ❌ Doesn't adapt to actual limits

### Rationale

Chose proactive rate limiting with 10% margin for **reliability and user experience**. The SDK should prevent errors rather than react to them. The margin accounts for clock skew and concurrent operations. This is critical for production bots that need predictable behavior.

---

## Decision 7: Webhook Signature Validation

### Decision

Always validate webhook signatures using constant-time comparison.

### Alternatives Considered

1. **No validation**: Trust all webhooks
2. **Optional validation**: Configurable
3. **Always validate**: Required validation (chosen)

### Analysis

**No Validation**:

- ✅ Simplest to implement
- ❌ **Critical security vulnerability**
- ❌ Allows spoofed webhooks
- ❌ Unacceptable for production
- ❌ Violates GitHub security best practices

**Optional Validation**:

- ✅ Flexibility for development
- ❌ Easy to forget in production
- ❌ Configuration drift risk
- ❌ Security by obscurity

**Always Validate (Chosen)**:

- ✅ Secure by default
- ✅ No configuration mistakes
- ✅ Constant-time comparison prevents timing attacks
- ✅ Aligns with security best practices
- ❌ Requires webhook secret configuration
- ❌ Cannot disable for testing (use mock instead)

### Rationale

Chose always validate for **security**. Webhook validation is not optional in production systems. Constant-time comparison prevents timing attacks. Development/testing should use proper mocks, not disabled security.

---

## Summary of Key Tradeoffs

| Decision | Chosen Approach | Key Tradeoff |
|----------|----------------|--------------|
| Authentication | Two-level (JWT + Installation) | Complexity vs proper modeling |
| Abstractions | Trait-based | Cognitive load vs testability |
| Token Caching | Proactive refresh | Memory usage vs reliability |
| API Style | Fully async | Complexity vs throughput |
| Error Handling | Explicit Result types | Verbosity vs safety |
| Rate Limiting | Proactive with margin | Unused quota vs reliability |
| Security | Always validate | Configuration burden vs safety |

## Future Decisions to Consider

1. **GraphQL vs REST**: Currently REST-only, may add GraphQL for complex queries
2. **Metrics/Observability**: Integration with OpenTelemetry vs custom metrics
3. **Retry Policies**: Fixed vs adaptive retry strategies
4. **Pagination**: Automatic page fetching vs manual iteration
5. **Caching Strategy**: In-memory vs Redis for multi-instance deployments
