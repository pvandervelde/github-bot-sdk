//! GitHub API client for authenticated operations.
//!
//! This module provides the primary interface for interacting with GitHub's REST API as a GitHub App.
//! It handles authentication, rate limiting, retries, pagination, and provides type-safe operations
//! for repositories, issues, pull requests, projects, and more.
//!
//! # Overview
//!
//! The client operates at two levels:
//!
//! - **App-level** ([`GitHubClient`]) - Operations as the GitHub App itself (using JWT)
//! - **Installation-level** ([`InstallationClient`]) - Operations within a specific installation (using installation tokens)
//!
//! Most operations happen at the installation level, where you interact with repositories,
//! issues, pull requests, etc. on behalf of the installation.
//!
//! # Features
//!
//! - **Automatic Authentication** - Tokens are injected and refreshed automatically
//! - **Rate Limit Handling** - Detects rate limits and backs off appropriately
//! - **Retry Logic** - Exponential backoff for transient failures (network errors, 5xx responses)
//! - **Pagination Support** - Helper methods for paginated API responses
//! - **Type Safety** - Strongly-typed request and response models
//! - **Configurable** - Timeouts, retry behavior, rate limit margins
//!
//! # Quick Start
//!
//! ```no_run
//! use github_bot_sdk::{
//!     auth::{AuthenticationProvider, InstallationId},
//!     client::{GitHubClient, ClientConfig},
//!     error::ApiError,
//! };
//! use std::time::Duration;
//!
//! # async fn example(auth: impl AuthenticationProvider + 'static) -> Result<(), ApiError> {
//! // Create client with custom configuration
//! let client = GitHubClient::builder(auth)
//!     .config(ClientConfig::default()
//!         .with_user_agent("my-bot/1.0")
//!         .with_timeout(Duration::from_secs(30))
//!         .with_max_retries(5))
//!     .build()?;
//!
//! // Get app information (app-level operation)
//! let app = client.get_app().await?;
//! println!("Running as: {}", app.name);
//!
//! // Get installation information
//! let installation_id = InstallationId::new(12345);
//! let installation = client.get_installation(installation_id).await?;
//! println!("Installation: {}", installation.id.as_u64());
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! Use [`ClientConfig`] to customize client behavior:
//!
//! ```
//! use github_bot_sdk::client::ClientConfig;
//! use std::time::Duration;
//!
//! let config = ClientConfig::default()
//!     .with_user_agent("my-github-bot/1.0")           // Required by GitHub
//!     .with_timeout(Duration::from_secs(60))          // Request timeout
//!     .with_max_retries(5)                            // Max retry attempts
//!     .with_rate_limit_margin(0.1)                    // Keep 10% rate limit buffer
//!     .with_github_api_url("https://api.github.com"); // Override for GHE
//!
//! // Or use the builder pattern
//! let config = ClientConfig::builder()
//!     .user_agent("my-bot/2.0")
//!     .timeout(Duration::from_secs(45))
//!     .max_retries(3)
//!     .build();
//! ```
//!
//! # Operations by Category
//!
//! ## Repository Operations
//!
//! ```no_run
//! # use github_bot_sdk::client::GitHubClient;
//! # use github_bot_sdk::auth::InstallationId;
//! # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
//! let installation_id = InstallationId::new(12345);
//! let _installation = client.get_installation(installation_id).await?;
//!
//! // Repository operations are available through client methods
//! // See InstallationClient and related modules for detailed API
//! # Ok(())
//! # }
//! ```
//!
//! ## Issue Operations
//!
//! ```no_run
//! # use github_bot_sdk::client::GitHubClient;
//! # use github_bot_sdk::auth::InstallationId;
//! # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
//! let installation_id = InstallationId::new(12345);
//! let _installation = client.get_installation(installation_id).await?;
//!
//! // Issue operations through client methods
//! // Create, comment, label, and manage issues
//! # Ok(())
//! # }
//! ```
//!
//! ## Pull Request Operations
//!
//! ```no_run
//! # use github_bot_sdk::client::GitHubClient;
//! # use github_bot_sdk::auth::InstallationId;
//! # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
//! let installation_id = InstallationId::new(12345);
//! let _installation = client.get_installation(installation_id).await?;
//!
//! // Pull request operations through client methods
//! // Create, review, merge, and manage PRs
//! # Ok(())
//! # }
//! ```
//!
//! ## Project Operations (GitHub Projects V2)
//!
//! ```no_run
//! # use github_bot_sdk::client::GitHubClient;
//! # use github_bot_sdk::auth::InstallationId;
//! # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
//! let installation_id = InstallationId::new(12345);
//! let _installation = client.get_installation(installation_id).await?;
//!
//! // Project operations through client methods
//! # Ok(())
//! # }
//! ```
//!
//! # Rate Limiting
//!
//! The client automatically handles GitHub's rate limits:
//!
//! - **Detection** - Monitors `X-RateLimit-*` headers in responses
//! - **Proactive Backoff** - Slows down when approaching limit (configurable margin)
//! - **Reactive Handling** - Respects `Retry-After` headers on 429 responses
//! - **Secondary Rate Limits** - Handles abuse detection (403 with retry-after)
//!
//! ```no_run
//! # use github_bot_sdk::client::{GitHubClient, ClientConfig};
//! # use github_bot_sdk::auth::AuthenticationProvider;
//! # async fn example(auth: impl AuthenticationProvider + 'static) -> Result<(), Box<dyn std::error::Error>> {
//! // Configure rate limit behavior
//! let config = ClientConfig::default()
//!     .with_rate_limit_margin(0.15);  // Start backing off at 85% of limit
//!
//! let client = GitHubClient::builder(auth)
//!     .config(config)
//!     .build()?;
//!
//! // Client will automatically handle rate limits
//! // No manual rate limit checking needed
//! # Ok(())
//! # }
//! ```
//!
//! # Retry Behavior
//!
//! The client automatically retries transient failures with exponential backoff:
//!
//! - **Retryable Errors**:
//!   - Network errors (timeouts, connection failures)
//!   - 5xx server errors (GitHub is having issues)
//!   - 429 rate limit exceeded (respects Retry-After)
//!   - 403 with Retry-After (secondary rate limit)
//!
//! - **Non-Retryable Errors**:
//!   - 4xx client errors (except 429 and some 403s)
//!   - Authentication failures
//!   - Validation errors
//!
//! ```no_run
//! use github_bot_sdk::client::ClientConfig;
//! use std::time::Duration;
//!
//! // Configure retry behavior
//! let config = ClientConfig::default()
//!     .with_max_retries(5)                            // Max attempts
//!     .with_timeout(Duration::from_secs(30));         // Request timeout
//!
//! // Default exponential backoff is applied automatically
//! ```
//!
//! # Pagination
//!
//! GitHub's API uses Link headers for pagination. See specific operation documentation for examples.
//!
//! ```no_run
//! # use github_bot_sdk::client::GitHubClient;
//! # use github_bot_sdk::auth::InstallationId;
//! # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
//! let installation_id = InstallationId::new(12345);
//!
//! // Get installation
//! let installation = client.get_installation(installation_id).await?;
//! println!("Installation: {}", installation.id.as_u64());
//! # Ok(())
//! # }
//! ```
//!
//! # Error Handling
//!
//! All operations return [`Result<T, ApiError>`]:
//!
//! ```no_run
//! # use github_bot_sdk::{client::GitHubClient, error::ApiError, auth::InstallationId};
//! # async fn example(client: &GitHubClient) -> Result<(), ApiError> {
//! let installation_id = InstallationId::new(12345);
//!
//! // Error handling example
//! match client.get_installation(installation_id).await {
//!     Ok(installation) => println!("Found installation: {}", installation.id.as_u64()),
//!     Err(ApiError::NotFound { .. }) => {
//!         println!("Installation doesn't exist or no access");
//!     }
//!     Err(ApiError::RateLimitExceeded { reset_at, .. }) => {
//!         println!("Rate limited, resets at: {:?}", reset_at);
//!     }
//!     Err(ApiError::AuthorizationFailed) => {
//!         println!("Insufficient permissions");
//!     }
//!     Err(e) => return Err(e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # GitHub Enterprise Support
//!
//! To use with GitHub Enterprise Server, configure the API base URL:
//!
//! ```
//! use github_bot_sdk::client::ClientConfig;
//!
//! let config = ClientConfig::default()
//!     .with_github_api_url("https://github.example.com/api/v3");
//! ```
//!
//! # See Also
//!
//! - [`crate::auth`] - Authentication types and providers
//! - [`crate::webhook`] - Webhook validation for incoming events
//! - [GitHub REST API Documentation](https://docs.github.com/en/rest)

mod app;
mod commit;
mod installation;
mod issue;
mod pagination;
mod project;
mod pull_request;
mod rate_limit;
mod release;
mod repository;
mod retry;
mod workflow;

use std::sync::Arc;
use std::time::Duration;

use reqwest;

use crate::auth::{AuthenticationProvider, Installation, InstallationId};
use crate::error::ApiError;

// ============================================================================
// Shared HTTP error mapping
// ============================================================================

/// Map an unsuccessful HTTP response to the canonical `ApiError` variant.
///
/// Called by sub-client modules (`issue`, `pull_request`) and
/// `InstallationClient::fetch_all_pages`. The response body is consumed only
/// for status codes that carry a useful message (422 and any unrecognised
/// error), avoiding unnecessary I/O on 401 / 403 / 404 responses.
pub(in crate::client) async fn map_http_error(
    status: reqwest::StatusCode,
    response: reqwest::Response,
) -> ApiError {
    match status.as_u16() {
        401 => ApiError::AuthenticationFailed,
        403 => ApiError::AuthorizationFailed,
        404 => ApiError::NotFound,
        422 => {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Validation failed".to_string());
            ApiError::InvalidRequest { message }
        }
        _ => {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            ApiError::HttpError {
                status: status.as_u16(),
                message,
            }
        }
    }
}

pub use app::App;
pub use commit::{
    CommitDetails, CommitReference, Comparison, FileChange, FullCommit, GitSignature, Verification,
};
pub use installation::InstallationClient;
pub use issue::{
    Comment, CreateCommentRequest, CreateIssueRequest, CreateLabelRequest, CreateMilestoneRequest,
    Issue, IssueActivityEvent, IssueRename, IssueUser, IssuesClient, Label, LabelsClient,
    LockReason, Milestone, MilestoneSortField, MilestoneState, MilestoneSummary, MilestonesClient,
    Reaction, ReactionContent, SortDirection, TimelineEvent, UpdateCommentRequest,
    UpdateIssueRequest, UpdateLabelRequest, UpdateMilestoneRequest,
};
pub use pagination::{extract_page_number, parse_link_header, PagedResponse, Pagination};
pub use project::{
    AddProjectV2ItemRequest, ProjectOwner, ProjectV2, ProjectV2Item, ProjectsClient,
};
pub use pull_request::{
    CreatePullRequestCommentRequest, CreatePullRequestRequest, CreateReviewRequest,
    DismissReviewRequest, MergePullRequestRequest, MergeResult, PullRequest, PullRequestBranch,
    PullRequestComment, PullRequestRepo, PullRequestsClient, Review,
    UpdatePullRequestCommentRequest, UpdatePullRequestRequest, UpdateReviewRequest,
};
pub use rate_limit::{parse_rate_limit_from_headers, RateLimit, RateLimitContext, RateLimiter};
pub use release::{
    CreateReleaseRequest, Release, ReleaseAsset, ReleasesClient, UpdateReleaseRequest,
};
pub use repository::{
    Branch, Commit, GitRef, OwnerType, RepositoriesClient, Repository, RepositoryOwner, Tag,
};
pub use retry::{
    calculate_rate_limit_delay, detect_secondary_rate_limit, parse_retry_after, RateLimitInfo,
    RetryPolicy,
};
pub use workflow::{
    TriggerWorkflowRequest, Workflow, WorkflowRun, WorkflowRunConclusion, WorkflowRunStatus,
    WorkflowState, WorkflowsClient,
};

/// Configuration for GitHub API client behavior.
///
/// Controls timeouts, retry behavior, rate limiting, and API endpoints.
///
/// # Examples
///
/// ```
/// use github_bot_sdk::client::ClientConfig;
/// use std::time::Duration;
///
/// let config = ClientConfig::default()
///     .with_timeout(Duration::from_secs(60))
///     .with_max_retries(5);
/// ```
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// User agent string for API requests (required by GitHub)
    pub user_agent: String,
    /// Request timeout duration
    pub timeout: Duration,
    /// Maximum number of retry attempts for transient failures
    pub max_retries: u32,
    /// Base delay for exponential backoff retries
    pub initial_retry_delay: Duration,
    /// Maximum delay between retries
    pub max_retry_delay: Duration,
    /// Rate limit safety margin (0.0 to 1.0) - buffer before hitting limits
    pub rate_limit_margin: f64,
    /// GitHub API base URL
    pub github_api_url: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            user_agent: "github-bot-sdk/0.1.0".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            initial_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(60),
            rate_limit_margin: 0.1, // Keep 10% buffer
            github_api_url: "https://api.github.com".to_string(),
        }
    }
}

impl ClientConfig {
    /// Create a new builder for client configuration.
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::new()
    }

    /// Set the user agent string.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum number of retries.
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the rate limit safety margin.
    pub fn with_rate_limit_margin(mut self, margin: f64) -> Self {
        self.rate_limit_margin = margin.clamp(0.0, 1.0);
        self
    }

    /// Set the GitHub API base URL.
    pub fn with_github_api_url(mut self, url: impl Into<String>) -> Self {
        self.github_api_url = url.into();
        self
    }
}

/// Builder for constructing `ClientConfig` instances.
#[derive(Debug)]
pub struct ClientConfigBuilder {
    config: ClientConfig,
}

impl ClientConfigBuilder {
    /// Create a new configuration builder with defaults.
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }

    /// Set the user agent string.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = user_agent.into();
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the maximum number of retries.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Set the rate limit safety margin.
    pub fn rate_limit_margin(mut self, margin: f64) -> Self {
        self.config.rate_limit_margin = margin.clamp(0.0, 1.0);
        self
    }

    /// Set the GitHub API base URL.
    pub fn github_api_url(mut self, url: impl Into<String>) -> Self {
        self.config.github_api_url = url.into();
        self
    }

    /// Build the final configuration.
    pub fn build(self) -> ClientConfig {
        self.config
    }
}

impl Default for ClientConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// GitHub API client for authenticated operations.
///
/// The main client for interacting with GitHub's REST API. Handles authentication,
/// rate limiting, retries, and provides both app-level and installation-level operations.
///
/// # Examples
///
/// ```no_run
/// # use github_bot_sdk::client::{GitHubClient, ClientConfig};
/// # use github_bot_sdk::auth::AuthenticationProvider;
/// # async fn example(auth: impl AuthenticationProvider + 'static) -> Result<(), Box<dyn std::error::Error>> {
/// let client = GitHubClient::builder(auth)
///     .config(ClientConfig::default())
///     .build()?;
///
/// // Get app information
/// let app = client.get_app().await?;
/// println!("App: {}", app.name);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct GitHubClient {
    auth: Arc<dyn AuthenticationProvider>,
    http_client: reqwest::Client,
    config: ClientConfig,
}

impl GitHubClient {
    /// Create a new builder for constructing a GitHub client.
    ///
    /// # Arguments
    ///
    /// * `auth` - Authentication provider for obtaining tokens
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::GitHubClient;
    /// # use github_bot_sdk::auth::AuthenticationProvider;
    /// # async fn example(auth: impl AuthenticationProvider + 'static) {
    /// let client = GitHubClient::builder(auth).build().unwrap();
    /// # }
    /// ```
    pub fn builder(auth: impl AuthenticationProvider + 'static) -> GitHubClientBuilder {
        GitHubClientBuilder::new(auth)
    }

    /// Get the client configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Get the authentication provider.
    pub fn auth_provider(&self) -> &dyn AuthenticationProvider {
        self.auth.as_ref()
    }

    /// Get the HTTP client (internal use by InstallationClient).
    pub(crate) fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    // ========================================================================
    // App-Level Operations (authenticated with JWT)
    // ========================================================================

    /// Get details about the authenticated GitHub App.
    ///
    /// Fetches metadata about the app including ID, name, owner, and permissions.
    ///
    /// # Authentication
    ///
    /// Requires app-level JWT authentication.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::GitHubClient;
    /// # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let app = client.get_app().await?;
    /// println!("App: {} (ID: {})", app.name, app.id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if:
    /// - JWT generation fails
    /// - HTTP request fails
    /// - Response cannot be parsed
    pub async fn get_app(&self) -> Result<App, ApiError> {
        // Get JWT token from auth provider
        let jwt = self
            .auth
            .app_token()
            .await
            .map_err(|e| ApiError::TokenGenerationFailed {
                message: format!("Failed to generate JWT: {}", e),
            })?;

        // Build request URL
        let url = format!("{}/app", self.config.github_api_url);

        // Make authenticated request
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt.token()))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| ApiError::Configuration {
                message: format!("HTTP request failed: {}", e),
            })?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error body".to_string());
            return Err(ApiError::Configuration {
                message: format!("API request failed with status {}: {}", status, error_text),
            });
        }

        // Parse response
        let app = response
            .json::<App>()
            .await
            .map_err(|e| ApiError::Configuration {
                message: format!("Failed to parse App response: {}", e),
            })?;

        Ok(app)
    }

    /// List all installations for the authenticated GitHub App.
    ///
    /// Fetches all installations where this app is installed, including organizations
    /// and user accounts.
    ///
    /// # Authentication
    ///
    /// Requires app-level JWT authentication.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::GitHubClient;
    /// # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let installations = client.list_installations().await?;
    /// for installation in installations {
    ///     println!("Installation ID: {} for {}", installation.id.as_u64(), installation.account.login);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if:
    /// - JWT generation fails
    /// - HTTP request fails
    /// - Response cannot be parsed
    pub async fn list_installations(&self) -> Result<Vec<Installation>, ApiError> {
        // Get JWT token from auth provider
        let jwt = self
            .auth
            .app_token()
            .await
            .map_err(|e| ApiError::TokenGenerationFailed {
                message: format!("Failed to generate JWT: {}", e),
            })?;

        // Build request URL
        let url = format!("{}/app/installations", self.config.github_api_url);

        // Make authenticated request
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt.token()))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| ApiError::Configuration {
                message: format!("HTTP request failed: {}", e),
            })?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error body".to_string());
            return Err(ApiError::Configuration {
                message: format!("API request failed with status {}: {}", status, error_text),
            });
        }

        // Parse response
        let installations =
            response
                .json::<Vec<Installation>>()
                .await
                .map_err(|e| ApiError::Configuration {
                    message: format!("Failed to parse installations response: {}", e),
                })?;

        Ok(installations)
    }

    /// Get a specific installation by ID.
    ///
    /// Fetches detailed information about a specific installation of this GitHub App.
    ///
    /// # Authentication
    ///
    /// Requires app-level JWT authentication.
    ///
    /// # Arguments
    ///
    /// * `installation_id` - The unique identifier for the installation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::GitHubClient;
    /// # use github_bot_sdk::auth::InstallationId;
    /// # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let installation_id = InstallationId::new(12345);
    /// let installation = client.get_installation(installation_id).await?;
    /// println!("Installation for: {}", installation.account.login);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if:
    /// - JWT generation fails
    /// - HTTP request fails
    /// - Installation not found (404)
    /// - Response cannot be parsed
    pub async fn get_installation(
        &self,
        installation_id: InstallationId,
    ) -> Result<Installation, ApiError> {
        // Get JWT token from auth provider
        let jwt = self
            .auth
            .app_token()
            .await
            .map_err(|e| ApiError::TokenGenerationFailed {
                message: format!("Failed to generate JWT: {}", e),
            })?;

        // Build request URL
        let url = format!(
            "{}/app/installations/{}",
            self.config.github_api_url,
            installation_id.as_u64()
        );

        // Make authenticated request
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt.token()))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| ApiError::Configuration {
                message: format!("HTTP request failed: {}", e),
            })?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error body".to_string());
            return Err(ApiError::Configuration {
                message: format!("API request failed with status {}: {}", status, error_text),
            });
        }

        // Parse response
        let installation =
            response
                .json::<Installation>()
                .await
                .map_err(|e| ApiError::Configuration {
                    message: format!("Failed to parse installation response: {}", e),
                })?;

        Ok(installation)
    }

    /// Make a raw authenticated GET request as the GitHub App.
    ///
    /// This is a generic method for making custom API requests that aren't covered
    /// by the specific methods. Returns the raw response for flexible handling by the caller.
    ///
    /// # Authentication
    ///
    /// Requires app-level JWT authentication.
    ///
    /// # Arguments
    ///
    /// * `path` - The API path (e.g., "/app/installations" or "app/installations")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::GitHubClient;
    /// # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // Make a custom GET request
    /// let response = client.get_as_app("/app/hook/config").await?;
    ///
    /// if response.status().is_success() {
    ///     let data: serde_json::Value = response.json().await?;
    ///     println!("Response: {:?}", data);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if:
    /// - JWT generation fails
    /// - HTTP request fails (network error, timeout, etc.)
    ///
    /// Note: Does NOT return an error for non-2xx status codes. The caller is responsible
    /// for checking the response status.
    pub async fn get_as_app(&self, path: &str) -> Result<reqwest::Response, ApiError> {
        // Get JWT token from auth provider
        let jwt = self
            .auth
            .app_token()
            .await
            .map_err(|e| ApiError::TokenGenerationFailed {
                message: format!("Failed to generate JWT: {}", e),
            })?;

        // Normalize path - remove leading slash if present for consistent URL building
        let normalized_path = path.strip_prefix('/').unwrap_or(path);

        // Build request URL
        let url = format!("{}/{}", self.config.github_api_url, normalized_path);

        // Make authenticated request
        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", jwt.token()))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| ApiError::Configuration {
                message: format!("HTTP request failed: {}", e),
            })?;

        Ok(response)
    }

    /// Make a raw authenticated POST request as the GitHub App.
    ///
    /// This is a generic method for making custom API requests that aren't covered
    /// by the specific methods. Returns the raw response for flexible handling by the caller.
    ///
    /// # Authentication
    ///
    /// Requires app-level JWT authentication.
    ///
    /// # Arguments
    ///
    /// * `path` - The API path (e.g., "/app/installations/{id}/suspended")
    /// * `body` - The request body to serialize as JSON
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::GitHubClient;
    /// # async fn example(client: &GitHubClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // Make a custom POST request
    /// let body = serde_json::json!({"reason": "Violation of terms"});
    /// let response = client.post_as_app("/app/installations/123/suspended", &body).await?;
    ///
    /// if response.status().is_success() {
    ///     println!("Installation suspended");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ApiError` if:
    /// - JWT generation fails
    /// - Body serialization fails
    /// - HTTP request fails (network error, timeout, etc.)
    ///
    /// Note: Does NOT return an error for non-2xx status codes. The caller is responsible
    /// for checking the response status.
    pub async fn post_as_app(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, ApiError> {
        // Get JWT token from auth provider
        let jwt = self
            .auth
            .app_token()
            .await
            .map_err(|e| ApiError::TokenGenerationFailed {
                message: format!("Failed to generate JWT: {}", e),
            })?;

        // Normalize path - remove leading slash if present for consistent URL building
        let normalized_path = path.strip_prefix('/').unwrap_or(path);

        // Build request URL
        let url = format!("{}/{}", self.config.github_api_url, normalized_path);

        // Make authenticated request with JSON body
        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt.token()))
            .header("Accept", "application/vnd.github+json")
            .json(body)
            .send()
            .await
            .map_err(|e| ApiError::Configuration {
                message: format!("HTTP request failed: {}", e),
            })?;

        Ok(response)
    }

    // Installation-level operations will be implemented in task 5.0
}

impl std::fmt::Debug for GitHubClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitHubClient")
            .field("config", &self.config)
            .field("auth", &"<AuthenticationProvider>")
            .finish()
    }
}

/// Builder for constructing `GitHubClient` instances.
pub struct GitHubClientBuilder {
    auth: Arc<dyn AuthenticationProvider>,
    config: Option<ClientConfig>,
}

impl GitHubClientBuilder {
    /// Create a new client builder.
    fn new(auth: impl AuthenticationProvider + 'static) -> Self {
        Self {
            auth: Arc::new(auth),
            config: None,
        }
    }

    /// Set the client configuration.
    ///
    /// If not set, uses `ClientConfig::default()`.
    pub fn config(mut self, config: ClientConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Build the GitHub client.
    ///
    /// # Errors
    ///
    /// Returns `ApiError::Configuration` if the HTTP client cannot be created.
    pub fn build(self) -> Result<GitHubClient, ApiError> {
        let config = self.config.unwrap_or_default();

        // Build reqwest client with timeout and user agent
        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| ApiError::Configuration {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(GitHubClient {
            auth: self.auth,
            http_client,
            config,
        })
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
