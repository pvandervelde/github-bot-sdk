# github-bot-sdk

[![Crates.io](https://img.shields.io/crates/v/github-bot-sdk.svg)](https://crates.io/crates/github-bot-sdk)
[![Documentation](https://docs.rs/github-bot-sdk/badge.svg)](https://docs.rs/github-bot-sdk)
[![CI](https://github.com/pvandervelde/github-bot-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/pvandervelde/github-bot-sdk/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/pvandervelde/github-bot-sdk/branch/master/graph/badge.svg)](https://codecov.io/gh/pvandervelde/github-bot-sdk)
[![License](https://img.shields.io/crates/l/github-bot-sdk.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A comprehensive GitHub Bot SDK for Rust, providing authentication, webhook validation, and API client functionality for building robust GitHub Apps and integrations.

## Why Use This SDK?

Building GitHub Apps requires handling complex authentication flows, webhook validation, rate limiting, and API interactions. This SDK provides:

- **Type-Safe API**: Leverage Rust's type system to catch errors at compile time rather than runtime
- **Security by Default**: Built-in HMAC signature validation, secure token handling, and constant-time comparisons
- **Production-Ready**: Automatic retry logic with exponential backoff, rate limit handling, and comprehensive error types
- **Developer Experience**: Clear abstractions, extensive documentation, and idiomatic Rust patterns
- **Zero-Cost Abstractions**: High-level ergonomics without runtime overhead

### Comparison with Direct API Usage

| Feature | Direct API Calls | This SDK |
|---------|-----------------|----------|
| Type Safety | Manual JSON parsing | Strongly-typed models |
| Authentication | Manual JWT signing | Automated token management |
| Rate Limiting | Manual tracking | Built-in detection and retry |
| Error Handling | HTTP status codes | Rich error types with context |
| Webhook Security | Manual HMAC validation | Verified by default |
| Token Expiry | Manual tracking | Automatic refresh and caching |

## Features

- **🔐 GitHub App Authentication**
  - RS256 JWT signing for GitHub App authentication
  - Automated installation token generation and caching
  - Secure token handling with memory zeroing on drop
  - Support for installation-scoped permissions

- **🔒 Webhook Validation**
  - HMAC-SHA256 signature verification
  - Constant-time comparison to prevent timing attacks
  - Built-in request body validation
  - Support for GitHub webhook secret rotation

- **🌐 API Client**
  - Type-safe wrappers for GitHub REST API endpoints
  - Repository, Issue, Pull Request, Project, Release, and Workflow operations
  - Automatic rate limit detection and handling
  - Exponential backoff retry logic for transient failures
  - Comprehensive pagination support

- **📨 Event Processing**
  - Webhook event envelope normalization
  - Type-safe event parsing and routing
  - Session-based event processing patterns
  - Support for all GitHub webhook event types

- **🦀 Rust-First Design**
  - Zero-cost abstractions with async/await
  - Leverages Rust's type system for correctness
  - Built on tokio runtime for high-performance async I/O
  - Comprehensive error types with rich context

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
github-bot-sdk = "0.1.0"
```

## Quick Start

```rust
// TODO: Add quick start example
```

## Documentation

- [API Documentation](https://docs.rs/github-bot-sdk)
- [Specification](docs/specs/)

## Examples

See the [examples/](examples/) directory for complete working examples.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
