//! Tests for release operations.

use super::*;
use crate::auth::InstallationId;
use crate::client::{ClientConfig, GitHubClient};
use crate::error::ApiError;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[path = "test_helpers.rs"]
mod test_helpers;
use test_helpers::MockAuthProvider;

mod construction {
    use super::*;

    /// Verify CreateReleaseRequest with only required tag_name field.
    ///
    /// Tests minimal release creation request with just the tag name.
    #[test]
    fn test_create_release_request_minimal() {
        let request = CreateReleaseRequest {
            tag_name: "v1.0.0".to_string(),
            target_commitish: None,
            name: None,
            body: None,
            draft: None,
            prerelease: None,
        };

        assert_eq!(request.tag_name, "v1.0.0");
        assert!(request.target_commitish.is_none());
        assert!(request.name.is_none());
        assert!(request.body.is_none());
        assert!(request.draft.is_none());
        assert!(request.prerelease.is_none());

        // Verify minimal JSON serialization
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["tag_name"], "v1.0.0");
        // Optional fields should not be present when None
        assert!(json.get("target_commitish").is_none());
        assert!(json.get("name").is_none());
        assert!(json.get("body").is_none());
    }

    /// Verify CreateReleaseRequest with all fields populated.
    ///
    /// Tests release creation with all optional fields provided.
    #[test]
    fn test_create_release_request_full() {
        let request = CreateReleaseRequest {
            tag_name: "v2.0.0".to_string(),
            target_commitish: Some("develop".to_string()),
            name: Some("Version 2.0.0".to_string()),
            body: Some("Full release notes".to_string()),
            draft: Some(true),
            prerelease: Some(false),
        };

        assert_eq!(request.tag_name, "v2.0.0");
        assert_eq!(request.target_commitish, Some("develop".to_string()));
        assert_eq!(request.name, Some("Version 2.0.0".to_string()));
        assert_eq!(request.body, Some("Full release notes".to_string()));
        assert_eq!(request.draft, Some(true));
        assert_eq!(request.prerelease, Some(false));

        // Verify complete JSON serialization
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["tag_name"], "v2.0.0");
        assert_eq!(json["target_commitish"], "develop");
        assert_eq!(json["name"], "Version 2.0.0");
        assert_eq!(json["body"], "Full release notes");
        assert_eq!(json["draft"], true);
        assert_eq!(json["prerelease"], false);
    }

    /// Verify UpdateReleaseRequest with selective field updates.
    ///
    /// Tests partial update capability with only some fields set.
    #[test]
    fn test_update_release_request_partial() {
        let request = UpdateReleaseRequest {
            tag_name: None,
            target_commitish: None,
            name: Some("Updated Release Name".to_string()),
            body: Some("Updated body".to_string()),
            draft: Some(false),
            prerelease: None,
        };

        // Verify structure
        assert!(request.tag_name.is_none());
        assert!(request.target_commitish.is_none());
        assert_eq!(request.name, Some("Updated Release Name".to_string()));
        assert_eq!(request.body, Some("Updated body".to_string()));
        assert_eq!(request.draft, Some(false));
        assert!(request.prerelease.is_none());

        // Verify selective serialization
        let json = serde_json::to_value(&request).unwrap();
        assert!(json.get("tag_name").is_none());
        assert!(json.get("target_commitish").is_none());
        assert_eq!(json["name"], "Updated Release Name");
        assert_eq!(json["body"], "Updated body");
        assert_eq!(json["draft"], false);
        assert!(json.get("prerelease").is_none());
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

    /// Verify get_release_by_tag handles tags with special characters (URL encoding).
    ///
    /// Tests GET /repos/:owner/:repo/releases/tags/:tag with tags containing slashes.
    #[tokio::test]
    async fn test_get_release_by_tag_with_slash() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let release_json = serde_json::json!({
            "id": 1,
            "node_id": "MDc6UmVsZWFzZTE=",
            "tag_name": "release/v1.0",
            "target_commitish": "main",
            "name": "Release 1.0",
            "body": "Release with slash in tag",
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
            "html_url": "https://github.com/octocat/Hello-World/releases/tag/release/v1.0",
            "assets": []
        });

        // URL-encoded path: release%2Fv1.0
        Mock::given(method("GET"))
            .and(path(
                "/repos/octocat/Hello-World/releases/tags/release%2Fv1.0",
            ))
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
            .get_release_by_tag("octocat", "Hello-World", "release/v1.0")
            .await;

        assert!(result.is_ok());
        let release = result.unwrap();
        assert_eq!(release.tag_name, "release/v1.0");
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
            .and(wiremock::matchers::body_json(serde_json::json!({
                "tag_name": "v1.0.0",
                "target_commitish": "main",
                "name": "v1.0.0",
                "body": "Description of the release"
            })))
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
    use super::*;

    /// Verify Release can be deserialized from GitHub API response.
    ///
    /// Tests that Release struct correctly deserializes from JSON matching
    /// GitHub's release API response format.
    #[test]
    fn test_release_deserialize() {
        let json = r#"{
            "id": 1,
            "node_id": "MDc6UmVsZWFzZTE=",
            "tag_name": "v1.0.0",
            "target_commitish": "main",
            "name": "v1.0.0 Release",
            "body": "Description of the release",
            "draft": false,
            "prerelease": false,
            "author": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2023-01-01T12:00:00Z",
            "published_at": "2023-01-01T13:00:00Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/releases/1",
            "html_url": "https://github.com/octocat/Hello-World/releases/tag/v1.0.0",
            "assets": [
                {
                    "id": 1,
                    "node_id": "MDEyOlJlbGVhc2VBc3NldDE=",
                    "name": "example.zip",
                    "label": "Example Asset",
                    "content_type": "application/zip",
                    "state": "uploaded",
                    "size": 1024,
                    "download_count": 42,
                    "uploader": {
                        "login": "octocat",
                        "id": 1,
                        "node_id": "MDQ6VXNlcjE=",
                        "type": "User"
                    },
                    "created_at": "2023-01-01T12:30:00Z",
                    "updated_at": "2023-01-01T12:30:00Z",
                    "browser_download_url": "https://github.com/octocat/Hello-World/releases/download/v1.0.0/example.zip"
                }
            ]
        }"#;

        let release: Release = serde_json::from_str(json).unwrap();

        assert_eq!(release.id, 1);
        assert_eq!(release.node_id, "MDc6UmVsZWFzZTE=");
        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.target_commitish, "main");
        assert_eq!(release.name, Some("v1.0.0 Release".to_string()));
        assert_eq!(release.body, Some("Description of the release".to_string()));
        assert!(!release.draft);
        assert!(!release.prerelease);
        assert_eq!(release.author.login, "octocat");
        assert_eq!(release.author.id, 1);
        assert!(release.published_at.is_some());
        assert_eq!(release.assets.len(), 1);
        assert_eq!(
            release.url,
            "https://api.github.com/repos/octocat/Hello-World/releases/1"
        );
        assert_eq!(
            release.html_url,
            "https://github.com/octocat/Hello-World/releases/tag/v1.0.0"
        );
    }

    /// Verify ReleaseAsset can be deserialized.
    ///
    /// Tests that ReleaseAsset struct correctly deserializes from JSON.
    #[test]
    fn test_release_asset_deserialize() {
        let json = r#"{
            "id": 1,
            "node_id": "MDEyOlJlbGVhc2VBc3NldDE=",
            "name": "example.zip",
            "label": "Example Asset",
            "content_type": "application/zip",
            "state": "uploaded",
            "size": 2048,
            "download_count": 100,
            "uploader": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2023-01-01T12:30:00Z",
            "updated_at": "2023-01-01T12:35:00Z",
            "browser_download_url": "https://github.com/octocat/Hello-World/releases/download/v1.0.0/example.zip"
        }"#;

        let asset: ReleaseAsset = serde_json::from_str(json).unwrap();

        assert_eq!(asset.id, 1);
        assert_eq!(asset.node_id, "MDEyOlJlbGVhc2VBc3NldDE=");
        assert_eq!(asset.name, "example.zip");
        assert_eq!(asset.label, Some("Example Asset".to_string()));
        assert_eq!(asset.content_type, "application/zip");
        assert_eq!(asset.state, "uploaded");
        assert_eq!(asset.size, 2048);
        assert_eq!(asset.download_count, 100);
        assert_eq!(asset.uploader.login, "octocat");
        assert_eq!(
            asset.browser_download_url,
            "https://github.com/octocat/Hello-World/releases/download/v1.0.0/example.zip"
        );
    }

    /// Verify CreateReleaseRequest serializes correctly.
    ///
    /// Tests that CreateReleaseRequest serializes to JSON with all fields
    /// when provided.
    #[test]
    fn test_create_release_request_serialize() {
        let request = CreateReleaseRequest {
            tag_name: "v1.0.0".to_string(),
            target_commitish: Some("main".to_string()),
            name: Some("Version 1.0.0".to_string()),
            body: Some("Release notes".to_string()),
            draft: Some(false),
            prerelease: Some(false),
        };

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["tag_name"], "v1.0.0");
        assert_eq!(json["target_commitish"], "main");
        assert_eq!(json["name"], "Version 1.0.0");
        assert_eq!(json["body"], "Release notes");
        assert_eq!(json["draft"], false);
        assert_eq!(json["prerelease"], false);
    }

    /// Verify UpdateReleaseRequest skips None fields.
    ///
    /// Tests that UpdateReleaseRequest only serializes fields that are Some,
    /// allowing partial updates without overwriting fields.
    #[test]
    fn test_update_release_request_serialize_partial() {
        let request = UpdateReleaseRequest {
            tag_name: Some("v1.0.1".to_string()),
            name: Some("Updated name".to_string()),
            body: None,
            target_commitish: None,
            draft: None,
            prerelease: None,
        };

        let json = serde_json::to_value(&request).unwrap();

        // Present fields
        assert_eq!(json["tag_name"], "v1.0.1");
        assert_eq!(json["name"], "Updated name");

        // None fields should not be present in JSON
        assert!(json.get("body").is_none());
        assert!(json.get("target_commitish").is_none());
        assert!(json.get("draft").is_none());
        assert!(json.get("prerelease").is_none());
    }
}

mod error_handling {
    use super::*;

    /// Verify get_release returns NotFound error for 404 response.
    ///
    /// Tests that attempting to fetch a non-existent release returns
    /// ApiError::NotFound.
    #[tokio::test]
    async fn test_release_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/releases/999999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest/releases/releases#get-a-release"
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

        let result = client.get_release("octocat", "Hello-World", 999999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    /// Verify create_release returns InvalidRequest for duplicate tag.
    ///
    /// Tests that attempting to create a release with an existing tag
    /// returns ApiError::InvalidRequest with validation error.
    #[tokio::test]
    async fn test_tag_already_exists() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/releases"))
            .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
                "message": "Validation Failed",
                "errors": [
                    {
                        "resource": "Release",
                        "code": "already_exists",
                        "field": "tag_name"
                    }
                ],
                "documentation_url": "https://docs.github.com/rest/releases/releases#create-a-release"
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

        let request = CreateReleaseRequest {
            tag_name: "v1.0.0".to_string(),
            target_commitish: None,
            name: None,
            body: None,
            draft: None,
            prerelease: None,
        };

        let result = client
            .create_release("octocat", "Hello-World", request)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvalidRequest { message } => {
                assert!(message.contains("Validation"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    /// Verify release operations return AuthorizationFailed for 403 response.
    ///
    /// Tests that attempting operations without proper permissions returns
    /// ApiError::AuthorizationFailed.
    #[tokio::test]
    async fn test_forbidden_access() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("DELETE"))
            .and(path("/repos/private-org/private-repo/releases/1"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "message": "Resource not accessible by integration",
                "documentation_url": "https://docs.github.com/rest/releases/releases#delete-a-release"
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

        let result = client
            .delete_release("private-org", "private-repo", 1)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::AuthorizationFailed));
    }

    /// Verify get_release returns AuthenticationFailed for 401 responses.
    ///
    /// Tests that a 401 Unauthorized response is properly mapped to AuthenticationFailed.
    #[tokio::test]
    async fn test_release_unauthorized() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_invalid_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/releases/12345"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "message": "Bad credentials",
                "documentation_url": "https://docs.github.com/rest"
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

        let result = client.get_release("octocat", "Hello-World", 12345).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::AuthenticationFailed
        ));
    }

    /// Verify get_release returns HttpError for 500 server errors.
    ///
    /// Tests that 5xx server errors are properly mapped to HttpError.
    #[tokio::test]
    async fn test_release_server_error() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/releases/12345"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
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

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::HttpError { status, .. } => {
                assert_eq!(status, 503);
            }
            _ => panic!("Expected HttpError"),
        }
    }
}
