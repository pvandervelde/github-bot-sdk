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
            maintainer_can_modify: None,
        };

        assert_eq!(request.title, "Test PR");
        assert_eq!(request.head, "feature-branch");
        assert_eq!(request.base, "main");
        assert!(request.body.is_none());
        assert!(request.draft.is_none());
    }

    /// Verify CreatePullRequestRequest with all fields populated.
    ///
    /// Ensures PR creation request supports all optional fields, including
    /// maintainer_can_modify for fork-sourced pull requests.
    #[test]
    fn test_create_pull_request_request_full() {
        let request = CreatePullRequestRequest {
            title: "Test PR".to_string(),
            head: "contributor:feature-branch".to_string(),
            base: "main".to_string(),
            body: Some("Detailed description".to_string()),
            draft: Some(true),
            milestone: Some(5),
            maintainer_can_modify: Some(true),
        };

        assert_eq!(request.title, "Test PR");
        assert_eq!(request.head, "contributor:feature-branch");
        assert_eq!(request.base, "main");
        assert_eq!(request.body, Some("Detailed description".to_string()));
        assert_eq!(request.draft, Some(true));
        assert_eq!(request.milestone, Some(5));
        assert_eq!(request.maintainer_can_modify, Some(true));
    }

    /// Verify CreatePullRequestRequest serializes maintainer_can_modify as boolean when Some.
    ///
    /// When maintainer_can_modify is Some(true) or Some(false), the field must be
    /// present in JSON with the correct boolean value so GitHub respects the caller's
    /// explicit preference.
    #[test]
    fn test_create_pr_request_with_maintainer_modify() {
        let request_true = CreatePullRequestRequest {
            title: "Test PR".to_string(),
            head: "contributor:feature".to_string(),
            base: "main".to_string(),
            body: None,
            draft: None,
            milestone: None,
            maintainer_can_modify: Some(true),
        };

        let json_true = serde_json::to_value(&request_true).unwrap();
        assert_eq!(json_true["maintainer_can_modify"], true);

        let request_false = CreatePullRequestRequest {
            title: "Test PR".to_string(),
            head: "contributor:feature".to_string(),
            base: "main".to_string(),
            body: None,
            draft: None,
            milestone: None,
            maintainer_can_modify: Some(false),
        };

        let json_false = serde_json::to_value(&request_false).unwrap();
        assert_eq!(json_false["maintainer_can_modify"], false);
    }

    /// Verify CreatePullRequestRequest omits maintainer_can_modify from JSON when None.
    ///
    /// When maintainer_can_modify is None, the field must be absent from serialized
    /// JSON so that the GitHub API applies its own default without explicit override.
    #[test]
    fn test_create_pr_request_without_maintainer_modify() {
        let request = CreatePullRequestRequest {
            title: "Test PR".to_string(),
            head: "feature-branch".to_string(),
            base: "main".to_string(),
            body: None,
            draft: None,
            milestone: None,
            maintainer_can_modify: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json.get("maintainer_can_modify").is_none());
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
    /// Tests: docs/spec/assertions.md #10
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
            .pull_requests()
            .list("owner", "repo", None, None)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].number, 42);
        assert_eq!(response.items[0].title, "Test PR");
        assert_eq!(response.items[0].state, "open");
    }

    /// Verify get_pull_request returns single PR from GitHub API.
    ///
    /// Tests: docs/spec/assertions.md #10
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

        let pr = client
            .pull_requests()
            .get("owner", "repo", 42)
            .await
            .unwrap();

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

        let result = client.pull_requests().get("owner", "repo", 999).await;

        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    /// Verify create_pull_request creates new PR via GitHub API.
    ///
    /// Tests: docs/spec/assertions.md #10
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
            maintainer_can_modify: None,
        };

        let pr = client
            .pull_requests()
            .create("owner", "repo", request)
            .await
            .unwrap();

        assert_eq!(pr.number, 42);
        assert_eq!(pr.title, "New Feature");
    }
}

mod comment_operations {
    use super::*;

    /// Verify update_comment patches an existing pull request comment and returns it.
    ///
    /// Tests PATCH /repos/{owner}/{repo}/issues/comments/{id} endpoint.
    #[tokio::test]
    async fn test_update_comment() {
        let mock_server = MockServer::start().await;

        let updated_comment_json = serde_json::json!({
            "id": 1,
            "node_id": "MDEyOklzc3VlQ29tbWVudDE=",
            "body": "Updated comment",
            "user": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
            "created_at": "2011-04-14T16:00:49Z",
            "updated_at": "2011-04-14T17:00:49Z",
            "html_url": "https://github.com/octocat/Hello-World/pull/1#issuecomment-1"
        });

        Mock::given(method("PATCH"))
            .and(path("/repos/octocat/Hello-World/issues/comments/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_comment_json))
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

        let request = UpdatePullRequestCommentRequest {
            body: "Updated comment".to_string(),
        };

        let result = client
            .pull_requests()
            .update_comment("octocat", "Hello-World", 1, request)
            .await;

        assert!(result.is_ok());
        let comment = result.unwrap();
        assert_eq!(comment.id, 1);
        assert_eq!(comment.body, "Updated comment");
    }

    /// Verify delete_comment removes a pull request comment.
    ///
    /// Tests DELETE /repos/{owner}/{repo}/issues/comments/{id} endpoint.
    #[tokio::test]
    async fn test_delete_comment() {
        let mock_server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/repos/octocat/Hello-World/issues/comments/1"))
            .respond_with(ResponseTemplate::new(204))
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

        let result = client
            .pull_requests()
            .delete_comment("octocat", "Hello-World", 1)
            .await;

        assert!(result.is_ok());
    }

    /// Verify update_comment returns NotFound for a non-existent comment.
    ///
    /// Tests PATCH /repos/{owner}/{repo}/issues/comments/{id} returning 404.
    #[tokio::test]
    async fn test_update_comment_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/repos/octocat/Hello-World/issues/comments/999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest/issues/comments#update-an-issue-comment"
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

        let request = UpdatePullRequestCommentRequest {
            body: "text".to_string(),
        };

        let result = client
            .pull_requests()
            .update_comment("octocat", "Hello-World", 999, request)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }
}
