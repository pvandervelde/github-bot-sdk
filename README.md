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

## Usage

### Authentication

#### JWT Generation for GitHub Apps

The SDK handles JWT (JSON Web Token) generation automatically for app-level operations:

```rust
use github_bot_sdk::auth::{GitHubAppId, JsonWebToken};

// JWT tokens are generated automatically by the authentication provider
// Maximum 10-minute expiration (GitHub requirement)
// Uses RS256 algorithm (RSA SHA-256)
```

#### Installation Tokens

Installation tokens provide access to specific installations with scoped permissions:

```rust
use github_bot_sdk::auth::{InstallationId, InstallationToken};

// Tokens are automatically requested and cached
// Refresh happens automatically before expiration
// Respects GitHub's rate limits during token refresh

let installation = client.installation(installation_id);
// Installation token is obtained and used automatically
```

#### Token Caching

The SDK includes built-in token caching to minimize API calls:

- Installation tokens are cached until near expiration
- Automatic refresh with 60-second buffer before expiry
- Thread-safe cache for concurrent access
- Configurable cache implementation

### API Operations

#### Repository Operations

```rust
// Get repository details
let repo = installation.get_repository("owner", "repo").await?;

// List repositories for installation
let repos = installation.list_repositories().await?;

// Get branch information
let branch = installation.get_branch("owner", "repo", "main").await?;

// List all branches
let branches = installation.list_branches("owner", "repo").await?;

// Create a branch
installation.create_branch("owner", "repo", "feature-branch", "base-sha").await?;

// Delete a branch
installation.delete_branch("owner", "repo", "old-branch").await?;
```

#### Issue Operations

```rust
// List issues
let issues = installation.list_issues("owner", "repo").await?;

// Get specific issue
let issue = installation.get_issue("owner", "repo", 123).await?;

// Create issue
let new_issue = installation
    .create_issue("owner", "repo", "Title", "Description")
    .await?;

// Update issue
installation
    .update_issue("owner", "repo", 123, Some("New Title"), Some("Updated body"))
    .await?;

// Close issue
installation.close_issue("owner", "repo", 123).await?;

// Add comment
installation
    .create_issue_comment("owner", "repo", 123, "Comment text")
    .await?;

// Add/remove labels
installation.add_labels("owner", "repo", 123, vec!["bug", "urgent"]).await?;
installation.remove_label("owner", "repo", 123, "wontfix").await?;
```

#### Pull Request Operations

```rust
// List pull requests
let prs = installation.list_pull_requests("owner", "repo").await?;

// Get specific PR
let pr = installation.get_pull_request("owner", "repo", 456).await?;

// Create pull request
let new_pr = installation
    .create_pull_request(
        "owner",
        "repo",
        "Feature: New capability",
        "feature-branch",
        "main",
        "Detailed description"
    )
    .await?;

// Update pull request
installation
    .update_pull_request("owner", "repo", 456, Some("Updated title"), None)
    .await?;

// Merge pull request
installation
    .merge_pull_request("owner", "repo", 456, "Merge commit message")
    .await?;

// Request reviewers
installation
    .request_reviewers("owner", "repo", 456, vec!["reviewer1", "reviewer2"])
    .await?;

// Add review comment
installation
    .create_review_comment("owner", "repo", 456, "path/to/file.rs", 10, "Review comment")
    .await?;
```

#### Project Operations (GitHub Projects V2)

```rust
// Get project details
let project = installation.get_project("owner", "repo", project_id).await?;

// List project items
let items = installation.list_project_items(project_id).await?;

// Add item to project
installation.add_project_item(project_id, content_id).await?;

// Update project item field
installation
    .update_project_item_field(project_id, item_id, field_id, "value")
    .await?;
```

#### Release Operations

```rust
// List releases
let releases = installation.list_releases("owner", "repo").await?;

// Get latest release
let latest = installation.get_latest_release("owner", "repo").await?;

// Create release
let release = installation
    .create_release("owner", "repo", "v1.0.0", "Release notes")
    .await?;

// Upload release asset
installation
    .upload_release_asset(release.id, "artifact.zip", asset_bytes)
    .await?;
```

#### Workflow Operations

```rust
// List workflows
let workflows = installation.list_workflows("owner", "repo").await?;

// Trigger workflow dispatch
installation
    .dispatch_workflow("owner", "repo", workflow_id, "main", inputs)
    .await?;

// List workflow runs
let runs = installation.list_workflow_runs("owner", "repo", workflow_id).await?;
```

### Event Processing

#### Parsing Webhook Events

```rust
use github_bot_sdk::events::{EventEnvelope, parse_webhook};

// Parse incoming webhook
let envelope = parse_webhook(headers, body)?;

// Access event data
println!("Event type: {}", envelope.event_type);
println!("Delivery ID: {}", envelope.delivery_id);
println!("Installation ID: {:?}", envelope.installation_id);

// Parse specific event payload
match envelope.event_type.as_str() {
    "issues" => {
        let issue_event: IssueEvent = serde_json::from_value(envelope.payload)?;
        // Handle issue event
    }
    "pull_request" => {
        let pr_event: PullRequestEvent = serde_json::from_value(envelope.payload)?;
        // Handle PR event
    }
    _ => {}
}
```

#### Session-Based Processing

```rust
use github_bot_sdk::events::{Session, SessionConfig};

// Create processing session
let session = Session::new(SessionConfig::default());

// Process event with session
session.process(envelope, |ctx| async {
    // Access installation client via context
    let client = ctx.installation_client();

    // Perform operations
    client.create_issue_comment(/*...*/).await?;

    Ok(())
}).await?;
```

### Webhook Handling

#### HMAC Signature Validation

```rust
use github_bot_sdk::webhook::{SignatureValidator, WebhookReceiver};

// Create validator
let validator = SignatureValidator::new(secret_provider);

// Validate webhook signature
let signature = request.headers().get("X-Hub-Signature-256")?;
let is_valid = validator.validate(body, signature).await?;

if !is_valid {
    return Err("Invalid webhook signature");
}
```

#### Complete Webhook Handler

```rust
use github_bot_sdk::webhook::WebhookHandler;

struct MyWebhookHandler {
    client: GitHubClient,
}

#[async_trait::async_trait]
impl WebhookHandler for MyWebhookHandler {
    async fn handle(&self, envelope: EventEnvelope) -> Result<(), Box<dyn std::error::Error>> {
        // Validate event
        // Process based on event type
        // Perform GitHub API operations
        Ok(())
    }
}
```

### Rate Limiting and Retry

#### Automatic Rate Limit Handling

The SDK automatically detects and handles GitHub's rate limits:

```rust
// Rate limits are detected from response headers
// Primary rate limits: Automatic backoff before hitting limit
// Secondary rate limits (429/403): Exponential backoff retry
// Retry-After headers are respected
```

#### Retry Configuration

```rust
use github_bot_sdk::client::ClientConfig;
use std::time::Duration;

let config = ClientConfig::default()
    .with_max_retries(5)  // Maximum retry attempts
    .with_initial_retry_delay(Duration::from_millis(100))
    .with_max_retry_delay(Duration::from_secs(60))
    .with_rate_limit_margin(0.1);  // Keep 10% buffer

let client = GitHubClient::builder(auth)
    .config(config)
    .build()?;
```

#### Pagination

```rust
use github_bot_sdk::client::PagedResponse;

// Automatic pagination support
let mut page = 1;
loop {
    let response: PagedResponse<Issue> = installation
        .list_issues_paginated("owner", "repo", page)
        .await?;

    // Process issues
    for issue in response.items {
        println!("Issue: {}", issue.title);
    }

    // Check for next page
    if response.next_page().is_none() {
        break;
    }
    page += 1;
}
```

## Configuration

### Environment Variables

The SDK can be configured using environment variables for common settings:

```bash
# GitHub App Configuration
GITHUB_APP_ID=123456
GITHUB_PRIVATE_KEY_PATH=/path/to/private-key.pem
GITHUB_WEBHOOK_SECRET=your_webhook_secret

# API Configuration
GITHUB_API_URL=https://api.github.com  # Default, override for GitHub Enterprise
GITHUB_USER_AGENT=my-bot/1.0

# Timeouts and Retries
GITHUB_REQUEST_TIMEOUT_SECS=30
GITHUB_MAX_RETRIES=3
```

### Client Configuration Options

Customize client behavior through `ClientConfig`:

```rust
use github_bot_sdk::client::ClientConfig;
use std::time::Duration;

let config = ClientConfig::builder()
    .user_agent("my-bot/1.0")                    // Required by GitHub API
    .timeout(Duration::from_secs(60))            // Request timeout
    .max_retries(5)                              // Maximum retry attempts
    .initial_retry_delay(Duration::from_millis(100))  // Base delay for retries
    .max_retry_delay(Duration::from_secs(60))    // Maximum backoff delay
    .rate_limit_margin(0.1)                      // Keep 10% rate limit buffer
    .github_api_url("https://api.github.com")   // API base URL
    .build();
```

### GitHub App Setup Requirements

To use this SDK, you need a GitHub App with appropriate permissions:

1. **Create a GitHub App**:
   - Go to Settings → Developer settings → GitHub Apps
   - Click "New GitHub App"
   - Configure webhook URL and permissions

2. **Required Permissions**:
   - Repository permissions (based on your use case):
     - Contents: Read & Write (for file operations)
     - Issues: Read & Write (for issue operations)
     - Pull requests: Read & Write (for PR operations)
     - Workflows: Read & Write (for workflow operations)
   - Organization permissions (if needed):
     - Members: Read (for team operations)

3. **Generate Private Key**:
   - In your app settings, scroll to "Private keys"
   - Click "Generate a private key"
   - Download and securely store the `.pem` file

4. **Install the App**:
   - Install the app on your organization or repositories
   - Note the installation ID from the installation URL

5. **Webhook Configuration** (if using webhooks):
   - Set webhook URL to your server endpoint
   - Generate and securely store webhook secret
   - Select event subscriptions you need

### Security Best Practices

- **Never commit private keys or secrets to version control**
- Store secrets in environment variables or secret management systems (Azure Key Vault, AWS Secrets Manager)
- Use separate keys for development and production
- Rotate webhook secrets periodically
- Implement proper webhook signature validation
- Use HTTPS for all webhook endpoints
- Monitor failed authentication attempts

## Testing

### Running Tests

Run the complete test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test auth::

# Run doc tests
cargo test --doc
```

### Test Coverage

Generate and view test coverage:

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --all-features --workspace --html

# Open coverage report
open target/llvm-cov/html/index.html  # macOS/Linux
start target/llvm-cov/html/index.html # Windows
```

### Mocking GitHub API with WireMock

The SDK uses `wiremock` for testing GitHub API interactions:

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[tokio::test]
async fn test_get_repository() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Setup mock response
    Mock::given(method("GET"))
        .and(path("/repos/owner/repo"))
        .and(header("Authorization", "token ghs_..."))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "name": "repo",
            "full_name": "owner/repo",
        })))
        .mount(&mock_server)
        .await;

    // Test with mock server
    let client = create_test_client(&mock_server.uri());
    let repo = client.get_repository("owner", "repo").await.unwrap();
    assert_eq!(repo.name, "repo");
}
```

### Integration Testing

For integration tests with real GitHub API:

```rust
// tests/integration_test.rs
use github_bot_sdk::client::GitHubClient;

#[tokio::test]
#[ignore]  // Run with: cargo test -- --ignored
async fn test_real_api() {
    let client = create_authenticated_client_from_env();
    let app = client.get_app().await.unwrap();
    assert!(!app.name.is_empty());
}
```

### Test Organization

Tests are colocated with source files using the `_tests.rs` pattern:

```
src/
├── auth/
│   ├── mod.rs          # Implementation
│   ├── mod_tests.rs    # Tests for mod.rs
│   ├── tokens.rs       # Implementation
│   └── tokens_tests.rs # Tests for tokens.rs
```

### Continuous Integration

Tests run automatically on every push via GitHub Actions:

- Unit tests with full coverage
- Doc tests
- Lint checks (rustfmt, clippy)
- Dependency security audits

See `.github/workflows/ci.yml` for the complete CI configuration.

## Documentation

- [API Documentation](https://docs.rs/github-bot-sdk)
- [Specification](docs/specs/)

## Examples

The SDK provides comprehensive examples demonstrating common use cases:

### Available Examples

Examples will be added in the `examples/` directory. Planned examples include:

- **Basic Bot** - Simple webhook listener with event processing
- **Issue Labeler** - Automatically label issues based on content
- **PR Review Bot** - Automated PR review comments and checks
- **Release Manager** - Automated release creation and notes
- **Repository Manager** - Bulk repository configuration and maintenance
- **Webhook Server** - Complete webhook receiver with signature validation
- **Multi-Installation Bot** - Handle events across multiple installations

### Running Examples

Once examples are added, run them with:

```bash
# Set required environment variables
export GITHUB_APP_ID=your_app_id
export GITHUB_PRIVATE_KEY_PATH=/path/to/key.pem
export GITHUB_WEBHOOK_SECRET=your_secret

# Run an example
cargo run --example basic_bot
```

### Example Structure

Each example demonstrates:

- Complete authentication setup
- Proper error handling
- Production-ready patterns
- Security best practices
- Logging and observability

See the [examples/](examples/) directory for complete working code with detailed comments.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
