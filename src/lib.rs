//! # GitHub Bot SDK
//!
//! A comprehensive Rust SDK for building GitHub Apps and bots with strong type safety,
//! built-in security, and production-ready patterns.
//!
//! ## Features
//!
//! - **GitHub App Authentication** - RS256 JWT signing and automated installation token management
//! - **🌐 API Client** - Type-safe GitHub REST API operations with automatic rate limiting
//! - **Webhook Security** - HMAC-SHA256 signature validation with constant-time comparison
//! - **Event Processing** - Structured webhook event parsing and routing
//! - **Production Ready** - Exponential backoff, retry logic, and comprehensive error handling
//! - **Rust First** - Zero-cost abstractions leveraging Rust's type system for correctness
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! github-bot-sdk = "0.1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! ```
//!
//! ## Core Concepts
//!
//! ### Authentication
//!
//! GitHub Apps use a two-tier authentication model:
//!
//! 1. **App-level JWT** - Short-lived tokens (max 10 minutes) for app-level operations
//! 2. **Installation Tokens** - Scoped tokens for operations on behalf of installations
//!
//! This SDK handles both automatically through the [`AuthenticationProvider`] trait.
//!
//! ### API Client
//!
//! The [`GitHubClient`] provides typed access to GitHub's REST API with:
//!
//! - Automatic token injection and refresh
//! - Built-in rate limit detection and handling
//! - Exponential backoff retry for transient failures
//! - Pagination support for list operations
//!
//! ### Webhook Processing
//!
//! Validate and process GitHub webhooks securely:
//!
//! - HMAC-SHA256 signature verification via [`SignatureValidator`]
//! - Structured event parsing with [`events`] module
//! - Type-safe event routing and handling
//!
//! ## Usage Examples
//!
//! ### Basic GitHub Client Usage
//!
//! ```rust,no_run
//! use github_bot_sdk::{
//!     auth::{GitHubAppId, InstallationId, AuthenticationProvider},
//!     client::{GitHubClient, ClientConfig},
//!     error::ApiError,
//! };
//!
//! # async fn example(auth_provider: impl AuthenticationProvider + 'static) -> Result<(), ApiError> {
//! // Build GitHub client with authentication
//! let client = GitHubClient::builder(auth_provider)
//!     .config(ClientConfig::default()
//!         .with_user_agent("my-bot/1.0")
//!         .with_timeout(std::time::Duration::from_secs(30)))
//!     .build()?;
//!
//! // Get app information
//! let app = client.get_app().await?;
//! println!("Authenticated as: {}", app.name);
//!
//! // Work with specific installation
//! let installation_id = InstallationId::new(12345);
//! let installation_client = client.installation(installation_id);
//!
//! // List repositories accessible to this installation
//! let repos = installation_client.list_repositories().await?;
//! for repo in repos {
//!     println!("Repository: {}", repo.full_name);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Repository Operations
//!
//! ```rust,no_run
//! # use github_bot_sdk::client::GitHubClient;
//! # use github_bot_sdk::auth::InstallationId;
//! # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
//! let installation = client.installation(InstallationId::new(12345));
//!
//! // Get repository details
//! let repo = installation.get_repository("owner", "repo").await?;
//! println!("Repository: {} (⭐ {})", repo.full_name, repo.stargazers_count);
//!
//! // Work with branches
//! let branch = installation.get_branch("owner", "repo", "main").await?;
//! println!("Main branch commit: {}", branch.commit.sha);
//!
//! // Create a new branch
//! installation.create_branch("owner", "repo", "feature", &branch.commit.sha).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Issue and Pull Request Operations
//!
//! ```rust,no_run
//! # use github_bot_sdk::client::GitHubClient;
//! # use github_bot_sdk::auth::InstallationId;
//! # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
//! let installation = client.installation(InstallationId::new(12345));
//!
//! // Create an issue
//! let issue = installation
//!     .create_issue("owner", "repo", "Bug found", "Description of bug...")
//!     .await?;
//! println!("Created issue #{}", issue.number);
//!
//! // Add a comment
//! installation
//!     .create_issue_comment("owner", "repo", issue.number, "Looking into this!")
//!     .await?;
//!
//! // Add labels
//! installation
//!     .add_labels("owner", "repo", issue.number, vec!["bug", "priority:high"])
//!     .await?;
//!
//! // Work with pull requests
//! let prs = installation.list_pull_requests("owner", "repo").await?;
//! for pr in prs {
//!     println!("PR #{}: {} ({})", pr.number, pr.title, pr.state);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Webhook Signature Validation
//!
//! ```rust,no_run
//! use github_bot_sdk::{
//!     webhook::SignatureValidator,
//!     auth::SecretProvider,
//!     error::ValidationError,
//! };
//! use std::sync::Arc;
//!
//! # async fn example(secret_provider: Arc<dyn SecretProvider>) -> Result<(), ValidationError> {
//! let validator = SignatureValidator::new(secret_provider);
//!
//! // Validate incoming webhook
//! let payload = b"{\"action\":\"opened\",\"issue\":{...}}";
//! let signature = "sha256=5c4a8d..."; // From X-Hub-Signature-256 header
//!
//! if validator.validate(payload, signature).await? {
//!     println!("✓ Valid webhook - processing event");
//!     // Parse and process the event
//! } else {
//!     println!("✗ Invalid signature - rejecting webhook");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Working with Tokens
//!
//! ```rust
//! use github_bot_sdk::auth::{JsonWebToken, GitHubAppId};
//! use chrono::{Utc, Duration};
//!
//! let app_id = GitHubAppId::new(123456);
//! let expires_at = Utc::now() + Duration::minutes(10);
//! let jwt = JsonWebToken::new("eyJ0...".to_string(), app_id, expires_at);
//!
//! // Check expiration
//! if jwt.is_expired() {
//!     println!("Token has expired");
//! }
//!
//! // Check if token expires soon (within 5 minutes)
//! if jwt.expires_soon(Duration::minutes(5)) {
//!     println!("Token expires soon - should refresh");
//! }
//! ```
//!
//! ## Module Organization
//!
//! - [`auth`] - Authentication types, traits, and token management
//! - [`client`] - GitHub API client and operation implementations
//! - [`error`] - Error types for all operations
//! - [`events`] - Webhook event parsing and processing
//! - [`webhook`] - Webhook signature validation and security
//!
//! ## Architecture
//!
//! This SDK follows hexagonal architecture principles:
//!
//! - **Core Domain** - Authentication, events, and API operations (in this crate)
//! - **Abstraction Layer** - Traits for external dependencies ([`SecretProvider`], [`JwtSigner`], etc.)
//! - **Infrastructure** - Your implementations for secret management, key storage, etc.
//!
//! This design ensures:
//! - Testability through dependency injection
//! - Flexibility to integrate with your infrastructure
//! - Type safety at compile time
//! - Clear separation of concerns
//!
//! ## Security
//!
//! Security is built into the SDK's design:
//!
//! - **No Token Logging** - Sensitive types implement custom `Debug` that redacts secrets
//! - **Memory Safety** - Token types zero memory on drop
//! - **Constant-Time Comparison** - Webhook signatures use timing-attack resistant comparison
//! - **HTTPS Only** - All GitHub API communication uses TLS
//! - **Type Safety** - Branded types prevent mixing up different ID types
//!
//! ## Error Handling
//!
//! All operations return `Result<T, E>` with rich error types:
//!
//! - [`ApiError`] - GitHub API errors, rate limits, network failures
//! - [`AuthError`] - Authentication and token errors
//! - [`ValidationError`] - Input validation and webhook signature errors
//! - [`EventError`] - Event parsing and processing errors
//!
//! Errors include context for debugging and implement retry classification
//! to distinguish transient failures from permanent errors.
//!
//! ## Rate Limiting
//!
//! The SDK automatically handles GitHub's rate limits:
//!
//! - Detects rate limit headers in responses
//! - Automatically backs off when approaching limits
//! - Respects `Retry-After` headers on 429 responses
//! - Configurable safety margin to avoid hitting limits
//!
//! ## Testing
//!
//! The SDK is designed for testability:
//!
//! - Mock implementations for all traits
//! - [`wiremock`](https://docs.rs/wiremock) integration for HTTP mocking
//! - Comprehensive test coverage
//! - Doc tests for all examples
//!
//! ## Specifications
//!
//! For detailed architectural specifications, see [`docs/specs/`](https://github.com/pvandervelde/github-bot-sdk/tree/master/docs/specs).
//!
//! ## Examples
//!
//! See the [repository examples](https://github.com/pvandervelde/github-bot-sdk/tree/master/examples)
//! for complete, runnable examples demonstrating common use cases.

// Public modules
pub mod auth;
pub mod client;
pub mod error;
pub mod events;
pub mod webhook;

// Re-export commonly used types at crate root for convenience
pub use error::{
    ApiError, AuthError, CacheError, EventError, SecretError, SigningError, ValidationError,
};

pub use auth::{
    AuthenticationProvider, GitHubApiClient, GitHubAppId, Installation, InstallationId,
    InstallationPermissions, InstallationToken, JsonWebToken, JwtClaims, JwtSigner, KeyAlgorithm,
    Permission, PermissionLevel, PrivateKey, RateLimitInfo, Repository, RepositoryId,
    RepositorySelection, SecretProvider, TokenCache, User, UserId, UserType,
};

pub use client::{ClientConfig, ClientConfigBuilder, GitHubClient, GitHubClientBuilder};

pub use webhook::SignatureValidator;
