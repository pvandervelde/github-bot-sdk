//! Tests for pull request operations.

use super::*;
use crate::auth::InstallationId;
use crate::client::{ClientConfig, GitHubClient};
use crate::error::ApiError;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[path = "test_helpers.rs"]
mod test_helpers;
use test_helpers::MockAuthProvider;

// ============================================================================
// Type Construction Tests
// ============================================================================

mod construction {
    use super::*;

    /// Verify CreatePullRequestRequest with required fields only.
    ///
    /// Ensures minimal PR creation request can be constructed.
    #[test]
    fn test_create_pull_request_request_minimal() {
        let request = CreatePullRequestRequest {
            title: "Test PR".to_string(),
            head: "feature-branch".to_string(),
            base: "main".to_string(),
            body: None,
            draft: None,
            milestone: None,
        };

        assert_eq!(request.title, "Test PR");
        assert_eq!(request.head, "feature-branch");
        assert_eq!(request.base, "main");
        assert!(request.body.is_none());
        assert!(request.draft.is_none());
    }

    /// Verify CreatePullRequestRequest with all fields.
    ///
    /// Ensures PR creation request supports optional fields.
    #[test]
    fn test_create_pull_request_request_full() {
        let request = CreatePullRequestRequest {
            title: "Test PR".to_string(),
            head: "feature-branch".to_string(),
            base: "main".to_string(),
            body: Some("Detailed description".to_string()),
            draft: Some(true),
            milestone: Some(5),
        };

        assert_eq!(request.title, "Test PR");
        assert_eq!(request.head, "feature-branch");
        assert_eq!(request.base, "main");
        assert_eq!(request.body, Some("Detailed description".to_string()));
        assert_eq!(request.draft, Some(true));
        assert_eq!(request.milestone, Some(5));
    }

    /// Verify UpdatePullRequestRequest with selective updates.
    ///
    /// Ensures PR update request supports partial field updates.
    #[test]
    fn test_update_pull_request_request_partial() {
        let request = UpdatePullRequestRequest {
            title: Some("Updated title".to_string()),
            body: None,
            state: None,
            base: None,
            milestone: None,
        };

        assert_eq!(request.title, Some("Updated title".to_string()));
        assert!(request.body.is_none());
        assert!(request.state.is_none());
    }

    /// Verify MergePullRequestRequest with merge method.
    ///
    /// Ensures merge request supports different merge strategies.
    #[test]
    fn test_merge_pull_request_request() {
        let request = MergePullRequestRequest {
            commit_title: Some("Merge feature".to_string()),
            commit_message: Some("Closes #123".to_string()),
            sha: None,
            merge_method: Some("squash".to_string()),
        };

        assert_eq!(request.commit_title, Some("Merge feature".to_string()));
        assert_eq!(request.merge_method, Some("squash".to_string()));
    }

    /// Verify CreateReviewRequest with event type.
    ///
    /// Ensures review request supports different review types.
    #[test]
    fn test_create_review_request() {
        let request = CreateReviewRequest {
            commit_id: Some("abc123".to_string()),
            body: Some("Looks good!".to_string()),
            event: "APPROVE".to_string(),
        };

        assert_eq!(request.event, "APPROVE");
        assert_eq!(request.body, Some("Looks good!".to_string()));
    }
}

// ============================================================================
// Pull Request Operations Tests
// ============================================================================

mod pull_request_operations {
    use super::*;

    /// Verify list_pull_requests returns PRs from GitHub API.
    ///
    /// Tests: github-bot-sdk-specs/assertions.md #10
    #[tokio::test]
    async fn test_list_pull_requests() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/pulls"))
            .and(header("Authorization", "Bearer test-token"))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "id": 1,
                    "node_id": "PR_1",
                    "number": 42,
                    "title": "Test PR",
                    "body": "Description",
                    "state": "open",
                    "user": {
                        "login": "testuser",
                        "id": 123,
                        "node_id": "U_123",
                        "type": "User"
                    },
                    "head": {
                        "ref": "feature-branch",
                        "sha": "abc123",
                        "repo": {
                            "id": 456,
                            "name": "repo",
                            "full_name": "owner/repo"
                        }
                    },
                    "base": {
                        "ref": "main",
                        "sha": "def456",
                        "repo": {
                            "id": 456,
                            "name": "repo",
                            "full_name": "owner/repo"
                        }
                    },
                    "draft": false,
                    "merged": false,
                    "mergeable": true,
                    "merge_commit_sha": null,
                    "assignees": [],
                    "requested_reviewers": [],
                    "labels": [],
                    "milestone": null,
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z",
                    "closed_at": null,
                    "merged_at": null,
                    "html_url": "https://github.com/owner/repo/pull/42"
                }
            ])))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token("test-token");
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();
        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let response = client
            .list_pull_requests("owner", "repo", None, None)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].number, 42);
        assert_eq!(response.items[0].title, "Test PR");
        assert_eq!(response.items[0].state, "open");
    }

    /// Verify get_pull_request returns single PR from GitHub API.
    ///
    /// Tests: github-bot-sdk-specs/assertions.md #10
    #[tokio::test]
    async fn test_get_pull_request() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/pulls/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 1,
                "node_id": "PR_1",
                "number": 42,
                "title": "Test PR",
                "body": "Description",
                "state": "open",
                "user": {
                    "login": "testuser",
                    "id": 123,
                    "node_id": "U_123",
                    "type": "User"
                },
                "head": {
                    "ref": "feature-branch",
                    "sha": "abc123",
                    "repo": {
                        "id": 456,
                        "name": "repo",
                        "full_name": "owner/repo"
                    }
                },
                "base": {
                    "ref": "main",
                    "sha": "def456",
                    "repo": {
                        "id": 456,
                        "name": "repo",
                        "full_name": "owner/repo"
                    }
                },
                "draft": false,
                "merged": false,
                "mergeable": true,
                "merge_commit_sha": null,
                "assignees": [],
                "requested_reviewers": [],
                "labels": [],
                "milestone": null,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "closed_at": null,
                "merged_at": null,
                "html_url": "https://github.com/owner/repo/pull/42"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token("test-token");
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();
        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let pr = client.get_pull_request("owner", "repo", 42).await.unwrap();

        assert_eq!(pr.number, 42);
        assert_eq!(pr.title, "Test PR");
    }

    /// Verify get_pull_request returns NotFound for non-existent PR.
    ///
    /// Tests: Error handling for missing resources
    #[tokio::test]
    async fn test_get_pull_request_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/pulls/999"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token("test-token");
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();
        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.get_pull_request("owner", "repo", 999).await;

        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    /// Verify create_pull_request creates new PR via GitHub API.
    ///
    /// Tests: github-bot-sdk-specs/assertions.md #10
    #[tokio::test]
    async fn test_create_pull_request() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/repos/owner/repo/pulls"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 1,
                "node_id": "PR_1",
                "number": 42,
                "title": "New Feature",
                "body": "Feature description",
                "state": "open",
                "user": {
                    "login": "testuser",
                    "id": 123,
                    "node_id": "U_123",
                    "type": "User"
                },
                "head": {
                    "ref": "feature-branch",
                    "sha": "abc123",
                    "repo": {
                        "id": 456,
                        "name": "repo",
                        "full_name": "owner/repo"
                    }
                },
                "base": {
                    "ref": "main",
                    "sha": "def456",
                    "repo": {
                        "id": 456,
                        "name": "repo",
                        "full_name": "owner/repo"
                    }
                },
                "draft": false,
                "merged": false,
                "mergeable": null,
                "merge_commit_sha": null,
                "assignees": [],
                "requested_reviewers": [],
                "labels": [],
                "milestone": null,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "closed_at": null,
                "merged_at": null,
                "html_url": "https://github.com/owner/repo/pull/42"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token("test-token");
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();
        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = CreatePullRequestRequest {
            title: "New Feature".to_string(),
            head: "feature-branch".to_string(),
            base: "main".to_string(),
            body: Some("Feature description".to_string()),
            draft: None,
            milestone: None,
        };

        let pr = client
            .create_pull_request("owner", "repo", request)
            .await
            .unwrap();

        assert_eq!(pr.number, 42);
        assert_eq!(pr.title, "New Feature");
    }
}
