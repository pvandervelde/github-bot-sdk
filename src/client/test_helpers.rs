//! Shared test utilities for client module tests.
//!
//! This module provides common test helpers used across client test files,
//! including mock authentication providers and test client setup utilities.

use crate::auth::{
    AuthenticationProvider, InstallationId, InstallationPermissions, InstallationToken,
    JsonWebToken,
};
use crate::client::{ClientConfig, GitHubClient, InstallationClient};
use crate::error::{ApiError, AuthError};
use chrono::{Duration, Utc};

/// Mock authentication provider for testing.
///
/// Provides a simple mock implementation of `AuthenticationProvider` that returns
/// a pre-configured installation token. Useful for testing client operations without
/// requiring actual GitHub authentication.
#[derive(Clone)]
pub struct MockAuthProvider {
    installation_token: Result<InstallationToken, String>,
}

impl MockAuthProvider {
    /// Creates a new mock auth provider with a valid installation token.
    ///
    /// # Arguments
    ///
    /// * `token` - The token string to use for authentication
    ///
    /// # Example
    ///
    /// ```ignore
    /// let auth = MockAuthProvider::new_with_token("ghs_test_token");
    /// ```
    pub fn new_with_token(token: &str) -> Self {
        let installation_id = InstallationId::new(12345);
        let expires_at = Utc::now() + Duration::hours(1);
        let permissions = InstallationPermissions::default();
        let repositories = Vec::new();

        Self {
            installation_token: Ok(InstallationToken::new(
                token.to_string(),
                installation_id,
                expires_at,
                permissions,
                repositories,
            )),
        }
    }

    /// Creates a new mock auth provider that returns an error.
    ///
    /// Useful for testing error handling paths in client code.
    ///
    /// # Arguments
    ///
    /// * `error_message` - The error message to return
    ///
    /// # Example
    ///
    /// ```ignore
    /// let auth = MockAuthProvider::new_with_error("Token expired");
    /// ```
    pub fn new_with_error(error_message: &str) -> Self {
        Self {
            installation_token: Err(error_message.to_string()),
        }
    }
}

#[async_trait::async_trait]
impl AuthenticationProvider for MockAuthProvider {
    async fn app_token(&self) -> Result<JsonWebToken, AuthError> {
        Err(AuthError::TokenGenerationFailed {
            message: "Not implemented for mock".to_string(),
        })
    }

    async fn installation_token(
        &self,
        _installation_id: InstallationId,
    ) -> Result<InstallationToken, AuthError> {
        self.installation_token
            .clone()
            .map_err(|msg| AuthError::TokenGenerationFailed { message: msg })
    }

    async fn refresh_installation_token(
        &self,
        installation_id: InstallationId,
    ) -> Result<InstallationToken, AuthError> {
        self.installation_token(installation_id).await
    }

    async fn list_installations(&self) -> Result<Vec<crate::auth::Installation>, AuthError> {
        Err(AuthError::TokenGenerationFailed {
            message: "Not implemented for mock".to_string(),
        })
    }

    async fn get_installation_repositories(
        &self,
        _installation_id: InstallationId,
    ) -> Result<Vec<crate::auth::Repository>, AuthError> {
        Err(AuthError::TokenGenerationFailed {
            message: "Not implemented for mock".to_string(),
        })
    }
}

/// Creates a test client with mock authentication.
///
/// Helper function that sets up a complete test client connected to a mock server.
/// Reduces boilerplate in tests by handling the common setup pattern.
///
/// # Arguments
///
/// * `token` - The authentication token to use
/// * `mock_server_uri` - The URI of the wiremock server
///
/// # Returns
///
/// An `InstallationClient` configured for testing
///
/// # Example
///
/// ```ignore
/// let mock_server = MockServer::start().await;
/// let client = create_test_client("ghs_token", &mock_server.uri()).await.unwrap();
/// ```
pub async fn create_test_client(
    token: &str,
    mock_server_uri: &str,
) -> Result<InstallationClient, ApiError> {
    let auth = MockAuthProvider::new_with_token(token);
    let github_client = GitHubClient::builder(auth)
        .config(ClientConfig::default().with_github_api_url(mock_server_uri.to_string()))
        .build()?;

    let installation_id = InstallationId::new(12345);
    github_client.installation_by_id(installation_id).await
}
