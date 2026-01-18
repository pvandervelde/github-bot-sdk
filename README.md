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

### Basic Authentication and API Usage

```rust
use github_bot_sdk::{
    auth::{GitHubAppAuth, AuthConfig, GitHubAppId, InstallationId},
    client::{GitHubClient, ClientConfig},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure authentication
    let app_id = GitHubAppId::new(123456); // Your GitHub App ID
    let private_key = std::fs::read_to_string("private-key.pem")?;

    // Create authentication provider
    // Note: You'll need to implement SecretProvider, JwtSigner, etc.
    // See documentation for complete implementation examples
    let auth = create_auth_provider(app_id, private_key).await?;

    // Build GitHub client
    let client = GitHubClient::builder(auth)
        .config(ClientConfig::default()
            .with_user_agent("my-bot/1.0")
            .with_timeout(std::time::Duration::from_secs(30)))
        .build()?;

    // Get app information
    let app = client.get_app().await?;
    println!("Authenticated as: {}", app.name);

    // Create installation client for specific installation
    let installation_id = InstallationId::new(98765);
    let installation_client = client.installation(installation_id);

    // Use the installation client for operations
    let repos = installation_client.list_repositories().await?;
    println!("Found {} repositories", repos.len());

    Ok(())
}
```

### Webhook Validation

```rust
use github_bot_sdk::webhook::SignatureValidator;
use github_bot_sdk::auth::SecretProvider;
use std::sync::Arc;

async fn handle_webhook(
    validator: &SignatureValidator,
    payload: &[u8],
    signature: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate webhook signature
    if validator.validate(payload, signature).await? {
        println!("✓ Valid webhook signature");

        // Parse and process the webhook
        let event: serde_json::Value = serde_json::from_slice(payload)?;
        println!("Event type: {}", event["action"]);

        // Handle the event...

        Ok(())
    } else {
        Err("Invalid webhook signature")?
    }
}
```

### Repository Operations

```rust
use github_bot_sdk::client::{GitHubClient, RepositoryClient};
use github_bot_sdk::auth::{InstallationId, RepositoryId};

async fn repository_operations(
    client: &GitHubClient,
    installation_id: InstallationId,
) -> Result<(), Box<dyn std::error::Error>> {
    let installation = client.installation(installation_id);

    // Get repository information
    let repo = installation
        .get_repository("owner", "repo")
        .await?;
    println!("Repository: {} (stars: {})", repo.full_name, repo.stargazers_count);

    // List branches
    let branches = installation
        .list_branches("owner", "repo")
        .await?;

    for branch in branches {
        println!("Branch: {}", branch.name);
    }

    Ok(())
}
```

### Issue and Pull Request Operations

```rust
use github_bot_sdk::client::{IssueClient, PullRequestClient};

async fn issue_pr_operations(
    installation: &InstallationClient,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create an issue
    let issue = installation
        .create_issue("owner", "repo", "Bug Report", "Found a bug...")
        .await?;
    println!("Created issue #{}", issue.number);

    // Add a comment
    installation
        .create_issue_comment("owner", "repo", issue.number, "Looking into this...")
        .await?;

    // Get pull requests
    let prs = installation
        .list_pull_requests("owner", "repo")
        .await?;

    for pr in prs {
        println!("PR #{}: {}", pr.number, pr.title);
    }

    Ok(())
}
```

### Event Processing

```rust
use github_bot_sdk::events::{EventEnvelope, EventProcessor};

struct MyEventProcessor;

#[async_trait::async_trait]
impl EventProcessor for MyEventProcessor {
    async fn process(&self, envelope: EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        match envelope.event_type.as_str() {
            "issues" => {
                println!("Issue event: {:?}", envelope.payload);
                // Handle issue event
            }
            "pull_request" => {
                println!("PR event: {:?}", envelope.payload);
                // Handle pull request event
            }
            _ => {
                println!("Unhandled event: {}", envelope.event_type);
            }
        }
        Ok(())
    }
}
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
