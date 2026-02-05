//! Tests for release operations.

use super::*;
use crate::auth::{
    AuthenticationProvider, InstallationId, InstallationPermissions, InstallationToken,
    JsonWebToken,
};
use crate::client::{ClientConfig, GitHubClient};
use crate::error::{ApiError, AuthError};
use chrono::{Duration, Utc};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ============================================================================
// Mock AuthenticationProvider for Testing
// ============================================================================

#[derive(Clone)]
struct MockAuthProvider {
    installation_token: Result<InstallationToken, String>,
}

impl MockAuthProvider {
    fn new_with_token(token: &str) -> Self {
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

mod construction {

    #[test]
    #[ignore = "TODO: Verify CreateReleaseRequest with only tag_name"]
    fn test_create_release_request_minimal() {
        todo!("Verify CreateReleaseRequest with only tag_name")
    }

    #[test]
    #[ignore = "TODO: Verify CreateReleaseRequest with all fields"]
    fn test_create_release_request_full() {
        todo!("Verify CreateReleaseRequest with all fields")
    }

    #[test]
    #[ignore = "TODO: Verify UpdateReleaseRequest with selective updates"]
    fn test_update_release_request_partial() {
        todo!("Verify UpdateReleaseRequest with selective updates")
    }
}

mod release_operations {
    use super::*;

    /// Verify list_releases fetches all releases from repository.
    ///
    /// Tests GET /repos/:owner/:repo/releases endpoint.
    #[tokio::test]
    async fn test_list_releases() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let releases_json = serde_json::json!([
            {
                "id": 1,
                "node_id": "MDc6UmVsZWFzZTE=",
                "tag_name": "v1.0.0",
                "target_commitish": "main",
                "name": "v1.0.0",
                "body": "First release",
                "draft": false,
                "prerelease": false,
                "author": {
                    "login": "octocat",
                    "id": 1,
                    "node_id": "MDQ6VXNlcjE=",
                    "type": "User"
                },
                "created_at": "2023-01-01T00:00:00Z",
                "published_at": "2023-01-01T00:00:00Z",
                "url": "https://api.github.com/repos/octocat/Hello-World/releases/1",
                "html_url": "https://github.com/octocat/Hello-World/releases/tag/v1.0.0",
                "assets": []
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/releases"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(releases_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let result = client.list_releases("octocat", "Hello-World").await;

        assert!(result.is_ok());
        let releases = result.unwrap();
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].tag_name, "v1.0.0");
        assert_eq!(releases[0].name, Some("v1.0.0".to_string()));
    }

    /// Verify get_latest_release fetches the latest published release.
    ///
    /// Tests GET /repos/:owner/:repo/releases/latest endpoint.
    #[tokio::test]
    async fn test_get_latest_release() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let release_json = serde_json::json!({
            "id": 1,
            "node_id": "MDc6UmVsZWFzZTE=",
            "tag_name": "v1.0.0",
            "target_commitish": "main",
            "name": "v1.0.0",
            "body": "Latest release",
            "draft": false,
            "prerelease": false,
            "author": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2023-01-01T00:00:00Z",
            "published_at": "2023-01-01T00:00:00Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/releases/1",
            "html_url": "https://github.com/octocat/Hello-World/releases/tag/v1.0.0",
            "assets": []
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/releases/latest"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(release_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let result = client.get_latest_release("octocat", "Hello-World").await;

        assert!(result.is_ok());
        let release = result.unwrap();
        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.name, Some("v1.0.0".to_string()));
        assert!(!release.draft);
        assert!(!release.prerelease);
    }

    /// Verify get_latest_release returns NotFound when no published releases exist.
    ///
    /// Tests 404 response from GET /repos/:owner/:repo/releases/latest.
    #[tokio::test]
    async fn test_get_latest_release_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/releases/latest"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest/releases/releases#get-the-latest-release"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let result = client.get_latest_release("octocat", "Hello-World").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    /// Verify get_release_by_tag fetches a release by tag name.
    ///
    /// Tests GET /repos/:owner/:repo/releases/tags/:tag endpoint.
    #[tokio::test]
    async fn test_get_release_by_tag() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let release_json = serde_json::json!({
            "id": 1,
            "node_id": "MDc6UmVsZWFzZTE=",
            "tag_name": "v1.0.0",
            "target_commitish": "main",
            "name": "v1.0.0",
            "body": "Release by tag",
            "draft": false,
            "prerelease": false,
            "author": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2023-01-01T00:00:00Z",
            "published_at": "2023-01-01T00:00:00Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/releases/1",
            "html_url": "https://github.com/octocat/Hello-World/releases/tag/v1.0.0",
            "assets": []
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/releases/tags/v1.0.0"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(release_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let result = client
            .get_release_by_tag("octocat", "Hello-World", "v1.0.0")
            .await;

        assert!(result.is_ok());
        let release = result.unwrap();
        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.id, 1);
    }

    /// Verify get_release fetches a release by ID.
    ///
    /// Tests GET /repos/:owner/:repo/releases/:id endpoint.
    #[tokio::test]
    async fn test_get_release() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let release_json = serde_json::json!({
            "id": 12345,
            "node_id": "MDc6UmVsZWFzZTEyMzQ1",
            "tag_name": "v2.0.0",
            "target_commitish": "main",
            "name": "v2.0.0",
            "body": "Release by ID",
            "draft": false,
            "prerelease": false,
            "author": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2023-01-01T00:00:00Z",
            "published_at": "2023-01-01T00:00:00Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/releases/12345",
            "html_url": "https://github.com/octocat/Hello-World/releases/tag/v2.0.0",
            "assets": []
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/releases/12345"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(release_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let result = client.get_release("octocat", "Hello-World", 12345).await;

        assert!(result.is_ok());
        let release = result.unwrap();
        assert_eq!(release.id, 12345);
        assert_eq!(release.tag_name, "v2.0.0");
    }

    /// Verify create_release creates a new release.
    ///
    /// Tests POST /repos/:owner/:repo/releases endpoint.
    #[tokio::test]
    async fn test_create_release() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let created_release_json = serde_json::json!({
            "id": 1,
            "node_id": "MDc6UmVsZWFzZTE=",
            "tag_name": "v1.0.0",
            "target_commitish": "main",
            "name": "v1.0.0",
            "body": "Description of the release",
            "draft": false,
            "prerelease": false,
            "author": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2023-01-01T00:00:00Z",
            "published_at": "2023-01-01T00:00:00Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/releases/1",
            "html_url": "https://github.com/octocat/Hello-World/releases/tag/v1.0.0",
            "assets": []
        });

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/releases"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(201).set_body_json(created_release_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let request = CreateReleaseRequest {
            tag_name: "v1.0.0".to_string(),
            target_commitish: Some("main".to_string()),
            name: Some("v1.0.0".to_string()),
            body: Some("Description of the release".to_string()),
            draft: None,
            prerelease: None,
        };

        let result = client
            .create_release("octocat", "Hello-World", request)
            .await;

        assert!(result.is_ok());
        let release = result.unwrap();
        assert_eq!(release.tag_name, "v1.0.0");
        assert!(!release.draft);
        assert!(!release.prerelease);
    }

    /// Verify create_release can create a draft release.
    ///
    /// Tests POST /repos/:owner/:repo/releases with draft=true.
    #[tokio::test]
    async fn test_create_release_draft() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let draft_release_json = serde_json::json!({
            "id": 2,
            "node_id": "MDc6UmVsZWFzZTI=",
            "tag_name": "v2.0.0",
            "target_commitish": "main",
            "name": "v2.0.0 Draft",
            "body": "Draft release",
            "draft": true,
            "prerelease": false,
            "author": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2023-01-01T00:00:00Z",
            "published_at": null,
            "url": "https://api.github.com/repos/octocat/Hello-World/releases/2",
            "html_url": "https://github.com/octocat/Hello-World/releases/tag/v2.0.0",
            "assets": []
        });

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/releases"))
            .respond_with(ResponseTemplate::new(201).set_body_json(draft_release_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let request = CreateReleaseRequest {
            tag_name: "v2.0.0".to_string(),
            target_commitish: None,
            name: Some("v2.0.0 Draft".to_string()),
            body: Some("Draft release".to_string()),
            draft: Some(true),
            prerelease: None,
        };

        let result = client
            .create_release("octocat", "Hello-World", request)
            .await;

        assert!(result.is_ok());
        let release = result.unwrap();
        assert_eq!(release.tag_name, "v2.0.0");
        assert!(release.draft);
        assert!(!release.prerelease);
    }

    /// Verify create_release can create a prerelease.
    ///
    /// Tests POST /repos/:owner/:repo/releases with prerelease=true.
    #[tokio::test]
    async fn test_create_release_prerelease() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let prerelease_json = serde_json::json!({
            "id": 3,
            "node_id": "MDc6UmVsZWFzZTM=",
            "tag_name": "v3.0.0-beta",
            "target_commitish": "main",
            "name": "v3.0.0 Beta",
            "body": "Beta release",
            "draft": false,
            "prerelease": true,
            "author": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2023-01-01T00:00:00Z",
            "published_at": "2023-01-01T00:00:00Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/releases/3",
            "html_url": "https://github.com/octocat/Hello-World/releases/tag/v3.0.0-beta",
            "assets": []
        });

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/releases"))
            .respond_with(ResponseTemplate::new(201).set_body_json(prerelease_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let request = CreateReleaseRequest {
            tag_name: "v3.0.0-beta".to_string(),
            target_commitish: None,
            name: Some("v3.0.0 Beta".to_string()),
            body: Some("Beta release".to_string()),
            draft: None,
            prerelease: Some(true),
        };

        let result = client
            .create_release("octocat", "Hello-World", request)
            .await;

        assert!(result.is_ok());
        let release = result.unwrap();
        assert_eq!(release.tag_name, "v3.0.0-beta");
        assert!(!release.draft);
        assert!(release.prerelease);
    }

    /// Verify update_release updates an existing release.
    ///
    /// Tests PATCH /repos/:owner/:repo/releases/:id endpoint.
    #[tokio::test]
    async fn test_update_release() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let updated_release_json = serde_json::json!({
            "id": 1,
            "node_id": "MDc6UmVsZWFzZTE=",
            "tag_name": "v1.0.1",
            "target_commitish": "main",
            "name": "v1.0.1",
            "body": "Updated release description",
            "draft": false,
            "prerelease": false,
            "author": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2023-01-01T00:00:00Z",
            "published_at": "2023-01-01T00:00:00Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/releases/1",
            "html_url": "https://github.com/octocat/Hello-World/releases/tag/v1.0.1",
            "assets": []
        });

        Mock::given(method("PATCH"))
            .and(path("/repos/octocat/Hello-World/releases/1"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_release_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let request = UpdateReleaseRequest {
            tag_name: Some("v1.0.1".to_string()),
            name: Some("v1.0.1".to_string()),
            body: Some("Updated release description".to_string()),
            ..Default::default()
        };

        let result = client
            .update_release("octocat", "Hello-World", 1, request)
            .await;

        assert!(result.is_ok());
        let release = result.unwrap();
        assert_eq!(release.tag_name, "v1.0.1");
        assert_eq!(
            release.body,
            Some("Updated release description".to_string())
        );
    }

    /// Verify delete_release deletes an existing release.
    ///
    /// Tests DELETE /repos/:owner/:repo/releases/:id endpoint.
    #[tokio::test]
    async fn test_delete_release() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("DELETE"))
            .and(path("/repos/octocat/Hello-World/releases/1"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        let client = github_client
            .installation_by_id(installation_id)
            .await
            .unwrap();

        let result = client.delete_release("octocat", "Hello-World", 1).await;

        assert!(result.is_ok());
    }
}

mod serialization {

    #[test]
    #[ignore = "TODO: Verify Release can be deserialized from GitHub API response"]
    fn test_release_deserialize() {
        todo!("Verify Release can be deserialized from GitHub API response")
    }

    #[test]
    #[ignore = "TODO: Verify ReleaseAsset can be deserialized"]
    fn test_release_asset_deserialize() {
        todo!("Verify ReleaseAsset can be deserialized")
    }

    #[test]
    #[ignore = "TODO: Verify CreateReleaseRequest serializes correctly"]
    fn test_create_release_request_serialize() {
        todo!("Verify CreateReleaseRequest serializes correctly")
    }

    #[test]
    #[ignore = "TODO: Verify UpdateReleaseRequest skips None fields"]
    fn test_update_release_request_serialize_partial() {
        todo!("Verify UpdateReleaseRequest skips None fields")
    }
}

mod error_handling {

    #[tokio::test]
    #[ignore = "TODO: Mock: 404 response returns ApiError::NotFound"]
    async fn test_release_not_found() {
        todo!("Mock: 404 response returns ApiError::NotFound")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: 422 validation error for duplicate tag"]
    async fn test_tag_already_exists() {
        todo!("Mock: 422 validation error for duplicate tag")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: 403 response returns ApiError::Forbidden"]
    async fn test_forbidden_access() {
        todo!("Mock: 403 response returns ApiError::Forbidden")
    }
}
