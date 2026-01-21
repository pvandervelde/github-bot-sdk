# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - TBD

Initial release of the GitHub Bot SDK for Rust.

### ⛰️  Features

#### Authentication
- **GitHub App Authentication** - Complete JWT and installation token lifecycle management
  - RS256 JWT signing for app-level authentication
  - Automatic installation token generation and caching
  - Token expiration tracking and automatic refresh
  - Secure token handling with memory zeroing on drop
  - Custom Debug implementations that redact sensitive values
- **Branded ID Types** - Type-safe identifiers preventing ID confusion at compile time
  - `GitHubAppId` for GitHub App identification
  - `InstallationId` for installation-scoped operations
  - `RepositoryId` and `UserId` for entity identification
- **Permission Management** - Installation permission tracking and validation
- **Trait-Based Design** - Flexible authentication provider interface for custom implementations
  - `AuthenticationProvider` - Main authentication interface
  - `SecretProvider` - Secure secret storage integration
  - `TokenCache` - Token caching strategy
  - `JwtSigner` - JWT signing implementation

#### GitHub API Client
- **Type-Safe API Operations** - Comprehensive GitHub REST API coverage
  - **Repository Operations** - Get, list, create branches, manage git references
  - **Issue Operations** - Create, update, close issues; manage comments and labels
  - **Pull Request Operations** - Create, merge PRs; manage reviews and comments
  - **Project Operations** - GitHub Projects V2 integration
  - **Release Operations** - Create releases and upload assets
  - **Workflow Operations** - Trigger workflow dispatches and list runs
- **Automatic Rate Limiting** - Built-in GitHub rate limit detection and handling
  - Proactive backoff when approaching limits (configurable margin)
  - Respects `Retry-After` headers on 429 responses
  - Secondary rate limit detection (403 with retry-after)
- **Retry Logic** - Exponential backoff for transient failures
  - Configurable max retries and backoff parameters
  - Intelligent retry classification (transient vs permanent errors)
  - Network error handling with automatic retries
- **Pagination Support** - Helper functions for paginated API responses
  - Link header parsing
  - Page navigation methods
  - Typed paged response wrapper
- **Flexible Configuration** - Customizable timeouts, retry behavior, and API endpoints
  - Builder pattern for client configuration
  - GitHub Enterprise Server support via custom API URL

#### Webhook Processing
- **HMAC Signature Validation** - Secure webhook validation with constant-time comparison
  - SHA-256 signature verification
  - Timing-attack resistant comparison
  - Integration with secure secret providers
- **Event Normalization** - Structured event envelope for consistent processing
  - `EventEnvelope` type wrapping all webhook events
  - Metadata extraction (delivery ID, event type, timestamps)
  - Repository and installation context
- **Session Management** - Ordered event processing with session tracking
  - Configurable session ID strategies
  - Support for correlation and tracing
- **Type-Safe Event Parsing** - Strongly-typed event structures for major event types
  - Issue events
  - Pull request events
  - Push events
  - Release events
  - And more

#### Error Handling
- **Rich Error Types** - Comprehensive error classification with context
  - `ApiError` - GitHub API errors with status codes and messages
  - `AuthError` - Authentication and token errors
  - `ValidationError` - Input validation and webhook signature errors
  - `EventError` - Event parsing and processing errors
  - `CacheError`, `SecretError`, `SigningError` - Infrastructure errors
- **Retry Classification** - Automatic detection of transient vs permanent errors
  - `is_transient()` method on all error types
  - `retry_after()` for errors with known retry delays
  - Support for intelligent retry policies

### 📚 Documentation

- **Comprehensive Rustdoc** - Production-quality documentation for all public APIs
  - Crate-level usage guide with multiple examples
  - Module-level overviews and architecture explanations
  - Type-level documentation with usage examples
  - Method-level documentation with error conditions
- **Architecture Documentation** - Complete specifications in `docs/specs/`
  - System architecture and design principles
  - Interface specifications for all major components
  - Behavioral assertions and test requirements
  - Security considerations and best practices
- **README** - Complete user-facing documentation
  - Quick start guide
  - Feature overview with comparison table
  - Comprehensive usage examples
  - Configuration guide
  - Testing instructions
  - Contributing guidelines
- **Examples** - Code examples demonstrating common use cases
  - Basic authentication setup
  - Repository operations
  - Issue and PR management
  - Webhook handling
  - Event processing

### 🧪 Testing

- **Comprehensive Test Suite** - High test coverage across all modules
  - Unit tests for all core functionality
  - Integration tests with mocked GitHub API (wiremock)
  - Property-based tests where applicable
  - Doc tests ensuring examples compile and work
- **Test Organization** - Co-located tests using `*_tests.rs` pattern
- **Mock Support** - Mock implementations for all trait interfaces
- **CI Integration** - Automated testing on every commit
  - Test execution with coverage reporting
  - Lint checks (clippy, rustfmt)
  - Documentation build verification
  - Dependency security audits

### ⚙️ Miscellaneous

- **CI/CD Pipelines** - GitHub Actions workflows
  - Continuous integration (tests, coverage)
  - Lint checks (rustfmt, clippy, cargo-deny)
  - Documentation generation
  - Automated releases with release-plz
- **Dependency Management** - Automated dependency updates with Renovate
- **Security** - Dependency security scanning with cargo-deny
- **Conventional Commits** - Structured commit messages for automated versioning
- **Automated Changelog** - Generated from commit history using git-cliff

### 🛡️ Security

- **No Token Logging** - Sensitive types implement custom `Debug` that redacts values
- **Memory Safety** - Token types zero memory on drop to prevent leakage
- **Constant-Time Comparison** - HMAC verification uses timing-attack resistant comparison
- **HTTPS Only** - All GitHub API communication uses TLS
- **Type Safety** - Branded types prevent mixing up different identifier types
- **Secret Provider Interface** - Integration with secure secret storage (Azure Key Vault, AWS Secrets Manager, etc.)

### 📦 Dependencies

- **tokio** (1.49) - Async runtime
- **reqwest** (0.13) - HTTP client
- **serde** (1.0) - Serialization framework
- **jsonwebtoken** (9.3) - JWT signing
- **chrono** (0.4) - Date/time handling
- **thiserror** (1.0) - Error derivation
- **async-trait** (0.1) - Async trait support
- **hmac** (0.12) / **sha2** (0.10) - Cryptographic operations
- **wiremock** (0.6.5) - HTTP mocking for tests

### Breaking Changes

None - this is the initial release.

### Known Limitations

- Requires tokio async runtime (no async-std support)
- GitHub Enterprise Server support requires manual API URL configuration
- Some less common GitHub API endpoints not yet implemented
- Pagination requires manual iteration (no automatic page fetching)

---

## Notes

This is the initial public release of github-bot-sdk. The SDK is production-ready and follows Rust best practices for API design, error handling, and documentation.

Future releases will maintain semantic versioning:
- **Patch** (0.1.x) - Bug fixes and documentation improvements
- **Minor** (0.x.0) - New features, additional API endpoints
- **Major** (x.0.0) - Breaking API changes

[Unreleased]: https://github.com/pvandervelde/github-bot-sdk/compare/v0.1.0..HEAD
[0.1.0]: https://github.com/pvandervelde/github-bot-sdk/releases/tag/v0.1.0

<!-- generated by git-cliff -->
