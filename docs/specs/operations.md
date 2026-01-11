# Operational Considerations

## Overview

This document defines operational aspects of deploying and maintaining the GitHub Bot SDK, including deployment patterns, monitoring requirements, scaling considerations, and operational constraints.

## Deployment Patterns

### Supported Deployment Models

#### 1. Serverless Function (Azure Functions, AWS Lambda)

**Characteristics**:

- Event-driven webhook processing
- Stateless execution per webhook
- Cold start considerations for token caching
- Short-lived execution context

**Operational Constraints**:

- Token cache must use external storage (Redis, DynamoDB)
- Private keys must come from secret management service
- Execution time limits (5-15 minutes typical)
- Concurrency limits apply

**Best Practices**:

- Use connection pooling to amortize TLS handshake costs
- Preload authentication tokens during function warmup
- Monitor cold start frequency and latency
- Implement circuit breakers for downstream API failures

#### 2. Long-Running Service (Kubernetes, Docker)

**Characteristics**:

- Persistent process with in-memory state
- Handles multiple webhooks over time
- Can maintain connection pools and caches
- Full control over lifecycle

**Operational Constraints**:

- Must handle graceful shutdown for deployments
- Token cache benefits from in-memory storage
- Need health check endpoints
- Requires log aggregation

**Best Practices**:

- Implement graceful shutdown with connection draining
- Use readiness/liveness probes for orchestration
- Set resource limits appropriately
- Implement structured logging for observability

#### 3. Background Worker (Sidekiq, Celery-style)

**Characteristics**:

- Processes webhooks from queue asynchronously
- Decoupled from webhook receiver
- Can retry failed operations independently
- Scales workers independently

**Operational Constraints**:

- Queue must be reliable and persistent
- Webhook ordering may not be preserved
- Need visibility into queue depth
- Requires idempotency

**Best Practices**:

- Use session-based routing for ordering guarantees
- Implement dead letter queues for failures
- Monitor queue depth and processing latency
- Set appropriate worker concurrency limits

## Monitoring and Observability

### Key Metrics to Track

#### Authentication Metrics

- **JWT generation count**: Rate of app-level authentication
- **Installation token requests**: Rate of installation token exchanges
- **Token cache hit rate**: Effectiveness of caching strategy
- **Authentication failures**: Errors by type (invalid key, expired token, etc.)
- **Token refresh operations**: Proactive vs reactive refreshes

**Alert Conditions**:

- Authentication failure rate > 5%
- Token cache hit rate < 80%
- JWT generation failures (indicates private key issues)

#### API Client Metrics

- **Request rate**: Overall API call volume
- **Request latency**: p50, p95, p99 response times
- **Rate limit remaining**: Current quota status
- **Rate limit resets**: When limits will refresh
- **Retry counts**: Frequency of retries by reason
- **Error rates**: By error type (4xx, 5xx, network)

**Alert Conditions**:

- Rate limit remaining < 10% for >5 minutes
- Error rate > 2%
- p95 latency > 5 seconds
- Secondary rate limit detections

#### Webhook Processing Metrics

- **Webhook receive rate**: Events per second
- **Processing latency**: Time from receive to complete
- **Signature validation failures**: Potential security issues
- **Parse errors**: Malformed or unknown events
- **Event types distribution**: Which events are most common

**Alert Conditions**:

- Signature validation failures (investigate immediately)
- Processing latency > 30 seconds at p95
- Parse error rate > 1%

### Logging Requirements

#### Structured Log Fields

Every log entry should include:

- `timestamp`: ISO 8601 format
- `level`: ERROR, WARN, INFO, DEBUG, TRACE
- `event_id`: Webhook delivery ID or generated correlation ID
- `installation_id`: GitHub App installation identifier
- `repository`: Repository full name (owner/repo)
- `operation`: High-level operation name
- `duration_ms`: Operation duration
- `error_type`: Error classification if applicable

#### Log Levels Guide

- **ERROR**: Authentication failures, API errors, unrecoverable issues
- **WARN**: Rate limiting activated, retries in progress, degraded performance
- **INFO**: Webhook received, operation completed, state changes
- **DEBUG**: API requests/responses, cache operations, detailed flow
- **TRACE**: Full request/response bodies, cryptographic operations

#### Sensitive Data Handling

**NEVER log**:

- Private keys or key material
- JWT tokens or installation tokens
- Webhook signatures
- Full authorization headers

**Redact in logs**:

- Show only first/last 4 characters of tokens: `ghp_****...**1234`
- Hash or omit webhook payload bodies
- Redact user email addresses and personal data

### Distributed Tracing

#### Trace Context Propagation

- GitHub webhook delivery ID becomes root span ID
- Propagate context through all operations in event processing
- Include repository and installation IDs as span attributes
- Track operation hierarchy:
  - Root: Webhook processing
  - Child: Authentication token acquisition
  - Child: API calls to GitHub
  - Child: Business logic operations

#### Span Attributes

Standardize span attributes:

- `github.event_type`: Webhook event type
- `github.installation_id`: Installation identifier
- `github.repository`: Repository full name
- `github.api.endpoint`: API endpoint called
- `github.api.method`: HTTP method
- `github.rate_limit.remaining`: Rate limit after operation
- `auth.token_type`: "jwt" or "installation_token"
- `auth.cache_hit`: Boolean for cache effectiveness

## Scaling Considerations

### Horizontal Scaling

#### Stateless Components

- **API Client**: Fully stateless, scales linearly
- **Webhook Validator**: Stateless, scales linearly
- **Event Parser**: Stateless, scales linearly

#### Stateful Components

- **Token Cache**: Requires shared storage (Redis) or consistent hashing
- **Rate Limiter**: Requires coordination across instances
- **Connection Pools**: Per-instance, not shared

#### Scaling Strategies

**Webhook Receiver**:

- Scale based on webhook ingestion rate
- Typical: 1 instance per 100 webhooks/second
- Use load balancer with session affinity for ordered processing
- Consider queue-based decoupling for burst handling

**Worker Processes**:

- Scale based on processing latency and queue depth
- Typical: 1 worker per 10-20 concurrent operations
- Monitor queue depth and adjust worker count dynamically
- Use session-based routing for ordering guarantees

### Vertical Scaling

#### Memory Requirements

- **Base SDK**: ~10-50 MB per instance
- **Token Cache**: ~1 KB per cached token
- **Connection Pools**: ~1-2 MB per pool
- **HTTP Client**: ~5-10 MB for reqwest internals

**Typical Memory**:

- Small bot (1-10 installations): 50-100 MB
- Medium bot (10-100 installations): 100-250 MB
- Large bot (100-1000 installations): 250-500 MB

#### CPU Requirements

- JWT signing: CPU-intensive (RSA operations)
- HMAC validation: Moderate CPU usage
- JSON parsing: Moderate CPU usage
- HTTP I/O: Minimal CPU (waiting on network)

**Typical CPU**:

- Webhook processing: 10-50ms CPU time per event
- JWT generation: 5-10ms per token
- API calls: <5ms CPU (mostly I/O wait)

### Rate Limiting at Scale

#### Single Instance

- Use in-memory rate limiter with margin-based throttling
- Track rate limit state per installation
- Implement exponential backoff for 429 responses

#### Multi-Instance

- **Challenge**: Shared rate limit pool across instances
- **Solutions**:
  1. **Redis-backed rate limiter**: Centralized state (adds latency)
  2. **Pessimistic allocation**: Each instance gets quota share
  3. **Optimistic coordination**: Use distributed counters with eventual consistency

**Recommended**: Pessimistic allocation with margin

- If 5000 req/hour limit and 10 instances
- Each instance assumes 450 req/hour budget (5000/10 - 10% margin)
- Simple, no coordination overhead
- Slight under-utilization acceptable

## Reliability and Resilience

### Failure Modes

#### GitHub API Outage

**Symptoms**: 5xx errors, timeouts, connection failures
**Impact**: Bot operations fail until GitHub recovers
**Mitigation**:

- Implement circuit breaker pattern
- Return 503 to webhook deliveries for retry
- Queue operations for later retry
- Monitor GitHub status page

#### Authentication Service Outage (Key Vault, etc.)

**Symptoms**: Cannot retrieve private keys or secrets
**Impact**: New token generation fails, existing cached tokens continue working
**Mitigation**:

- Long token cache TTL (55 minutes)
- Graceful degradation with cached tokens
- Alert on secret retrieval failures
- Consider backup secret storage

#### Rate Limit Exhaustion

**Symptoms**: 429 responses from GitHub API
**Impact**: Operations delayed or fail
**Mitigation**:

- Proactive rate limiting with margin
- Priority queuing for critical operations
- Exponential backoff with jitter
- Monitor rate limit usage trends

#### Token Expiration During Operation

**Symptoms**: 401 responses mid-operation
**Impact**: Operations fail unexpectedly
**Mitigation**:

- Proactive token refresh (5 min margin)
- Automatic retry with fresh token
- Monitor token refresh timing

### Circuit Breaker Pattern

**Implementation Requirements**:

- Track failure rate per installation
- Open circuit after 50% failure rate over 1 minute
- Half-open state after 30 seconds
- Reset after 3 successful operations

**Operational Impact**:

- Prevents cascade failures
- Reduces load on failing installations
- Allows other installations to continue operating
- Must log circuit state changes

## Configuration Management

### Required Configuration

#### Authentication

- `GITHUB_APP_ID`: GitHub App identifier
- `GITHUB_PRIVATE_KEY`: RSA private key (PEM format)
- `GITHUB_WEBHOOK_SECRET`: Webhook signature validation secret

#### Optional Configuration

- `GITHUB_API_URL`: Override for GitHub Enterprise
- `RATE_LIMIT_MARGIN`: Safety margin (default: 0.1)
- `TOKEN_CACHE_TTL`: Token cache duration (default: 55 minutes)
- `MAX_RETRIES`: Maximum retry attempts (default: 3)
- `REQUEST_TIMEOUT`: HTTP timeout (default: 30 seconds)

### Environment-Specific Settings

#### Development

- `GITHUB_API_URL`: May point to local mock server
- `LOG_LEVEL`: DEBUG or TRACE for detailed visibility
- Shorter timeouts for faster feedback

#### Production

- `GITHUB_API_URL`: Real GitHub API (<https://api.github.com>)
- `LOG_LEVEL`: INFO or WARN for performance
- Production-grade secret management (Key Vault, not env vars)
- Enable distributed tracing
- Configure structured logging with log aggregation

## Disaster Recovery

### Backup Requirements

- **Private Keys**: Store encrypted backups in multiple secure locations
- **Configuration**: Version control all configuration
- **Webhook Secrets**: Document rotation procedures

### Recovery Procedures

#### Private Key Compromise

1. Generate new private key in GitHub App settings
2. Update secret management system with new key
3. Deploy updated configuration to all instances
4. Rotate old key (GitHub supports up to 2 keys during rotation)
5. Remove old key after migration complete

#### Webhook Secret Rotation

1. Generate new webhook secret
2. Update secret in GitHub App settings (GitHub validates both old and new)
3. Update bot configuration with new secret
4. Deploy to all instances
5. Remove old secret from GitHub after validation

#### Total System Recovery

1. Restore private key from secure backup
2. Deploy bot infrastructure from version control
3. Configure secrets in secret management system
4. Deploy bot instances
5. Verify webhook delivery and processing
6. Monitor for authentication and API errors

## Performance Benchmarks

### Target Performance

- **Webhook Processing**: <500ms p95 end-to-end
- **JWT Generation**: <10ms p95
- **Installation Token Exchange**: <200ms p95
- **API Request Latency**: <1000ms p95 (depends on GitHub API)
- **Token Cache Hit Rate**: >90% under normal load

### Load Testing Recommendations

- Test with realistic webhook distribution
- Simulate rate limiting scenarios
- Test token expiration and refresh under load
- Verify graceful degradation during GitHub outages
- Test multi-instance coordination if applicable
