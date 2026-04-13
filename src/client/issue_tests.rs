//! Tests for issue operations.

use super::*;
use crate::auth::InstallationId;
use crate::client::{ClientConfig, GitHubClient};
use crate::error::ApiError;
use wiremock::matchers::{header, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[path = "test_helpers.rs"]
mod test_helpers;
use test_helpers::MockAuthProvider;

mod construction {
    use super::*;

    /// Verify CreateIssueRequest with only required title field.
    #[test]
    fn test_create_issue_request_minimal() {
        let request = CreateIssueRequest {
            title: "Bug report".to_string(),
            body: None,
            assignees: None,
            milestone: None,
            labels: None,
        };

        assert_eq!(request.title, "Bug report");
        assert!(request.body.is_none());
        assert!(request.assignees.is_none());
    }

    /// Verify CreateIssueRequest with all optional fields populated.
    #[test]
    fn test_create_issue_request_full() {
        let request = CreateIssueRequest {
            title: "Bug report".to_string(),
            body: Some("Description of the bug".to_string()),
            assignees: Some(vec!["octocat".to_string()]),
            milestone: Some(1),
            labels: Some(vec!["bug".to_string()]),
        };

        assert_eq!(request.title, "Bug report");
        assert_eq!(request.body, Some("Description of the bug".to_string()));
        assert_eq!(request.assignees.as_ref().unwrap().len(), 1);
        assert_eq!(request.milestone, Some(1));
        assert_eq!(request.labels.as_ref().unwrap().len(), 1);
    }

    /// Verify UpdateIssueRequest with selective field updates.
    #[test]
    fn test_update_issue_request_partial() {
        let request = UpdateIssueRequest {
            title: Some("Updated title".to_string()),
            state: Some("closed".to_string()),
            ..Default::default()
        };

        assert_eq!(request.title, Some("Updated title".to_string()));
        assert_eq!(request.state, Some("closed".to_string()));
        assert!(request.body.is_none());
        assert!(request.assignees.is_none());
    }

    /// Verify CreateLabelRequest construction with all fields.
    #[test]
    fn test_create_label_request() {
        let request = CreateLabelRequest {
            name: "priority-high".to_string(),
            description: Some("High priority issue".to_string()),
            color: "ff0000".to_string(),
        };

        assert_eq!(request.name, "priority-high");
        assert_eq!(request.description, Some("High priority issue".to_string()));
        assert_eq!(request.color, "ff0000");
    }

    /// Verify CreateCommentRequest construction.
    #[test]
    fn test_create_comment_request() {
        let request = CreateCommentRequest {
            body: "This is a comment".to_string(),
        };

        assert_eq!(request.body, "This is a comment");
    }
}

mod issue_operations {
    use super::*;

    /// Verify list_issues returns all repository issues.
    ///
    /// Tests GET /repos/{owner}/{repo}/issues endpoint.
    #[tokio::test]
    async fn test_list_issues_all() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let issues_json = serde_json::json!([
            {
                "id": 1,
                "node_id": "MDU6SXNzdWUx",
                "number": 1347,
                "title": "Found a bug",
                "body": "Bug description",
                "state": "open",
                "user": {
                    "login": "octocat",
                    "id": 1,
                    "node_id": "MDQ6VXNlcjE=",
                    "type": "User"
                },
                "labels": [],
                "assignees": [],
                "milestone": null,
                "comments": 0,
                "created_at": "2011-04-22T13:33:48Z",
                "updated_at": "2011-04-22T13:33:48Z",
                "closed_at": null,
                "html_url": "https://github.com/octocat/Hello-World/issues/1347"
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(200).set_body_json(issues_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list("octocat", "Hello-World", None, None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].number, 1347);
        assert_eq!(response.items[0].title, "Found a bug");
    }

    /// Verify list_issues can filter by state parameter.
    ///
    /// Tests GET /repos/{owner}/{repo}/issues?state=open endpoint.
    #[tokio::test]
    async fn test_list_issues_filtered_by_state() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues"))
            .and(query_param("state", "open"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list("octocat", "Hello-World", Some("open"), None)
            .await;

        assert!(result.is_ok());
    }

    /// Verify get_issue returns a specific issue by number.
    ///
    /// Tests GET /repos/{owner}/{repo}/issues/{number} endpoint.
    #[tokio::test]
    async fn test_get_issue_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let issue_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6SXNzdWUx",
            "number": 1347,
            "title": "Found a bug",
            "body": "Bug description",
            "state": "open",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "labels": [],
            "assignees": [],
            "milestone": null,
            "comments": 0,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-22T13:33:48Z",
            "closed_at": null,
            "html_url": "https://github.com/octocat/Hello-World/issues/1347"
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/1347"))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.issues().get("octocat", "Hello-World", 1347).await;

        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.number, 1347);
        assert_eq!(issue.title, "Found a bug");
    }

    /// Verify get_issue returns NotFound for non-existent issue.
    #[tokio::test]
    async fn test_get_issue_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/9999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.issues().get("octocat", "Hello-World", 9999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    /// Verify create_issue with minimal required fields (title only).
    ///
    /// Tests POST /repos/{owner}/{repo}/issues endpoint.
    #[tokio::test]
    async fn test_create_issue_minimal() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let created_issue_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6SXNzdWUx",
            "number": 1348,
            "title": "New bug",
            "body": null,
            "state": "open",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "labels": [],
            "assignees": [],
            "milestone": null,
            "comments": 0,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-22T13:33:48Z",
            "closed_at": null,
            "html_url": "https://github.com/octocat/Hello-World/issues/1348"
        });

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/issues"))
            .respond_with(ResponseTemplate::new(201).set_body_json(created_issue_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = CreateIssueRequest {
            title: "New bug".to_string(),
            body: None,
            assignees: None,
            milestone: None,
            labels: None,
        };

        let result = client
            .issues()
            .create("octocat", "Hello-World", request)
            .await;

        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.number, 1348);
        assert_eq!(issue.title, "New bug");
    }

    /// Verify create_issue with all optional fields populated.
    #[tokio::test]
    async fn test_create_issue_full() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let created_issue_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6SXNzdWUx",
            "number": 1349,
            "title": "Full bug report",
            "body": "Detailed description",
            "state": "open",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "labels": [{
                "id": 1,
                "node_id": "MDU6TGFiZWwx",
                "name": "bug",
                "description": null,
                "color": "ff0000",
                "default": true
            }],
            "assignees": [{
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            }],
            "milestone": null,
            "comments": 0,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-22T13:33:48Z",
            "closed_at": null,
            "html_url": "https://github.com/octocat/Hello-World/issues/1349"
        });

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/issues"))
            .respond_with(ResponseTemplate::new(201).set_body_json(created_issue_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = CreateIssueRequest {
            title: "Full bug report".to_string(),
            body: Some("Detailed description".to_string()),
            assignees: Some(vec!["octocat".to_string()]),
            milestone: Some(1),
            labels: Some(vec!["bug".to_string()]),
        };

        let result = client
            .issues()
            .create("octocat", "Hello-World", request)
            .await;

        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.number, 1349);
        assert_eq!(issue.labels.len(), 1);
        assert_eq!(issue.assignees.len(), 1);
    }

    /// Verify update_issue modifies issue fields.
    ///
    /// Tests PATCH /repos/{owner}/{repo}/issues/{number} endpoint.
    #[tokio::test]
    async fn test_update_issue() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let updated_issue_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6SXNzdWUx",
            "number": 1347,
            "title": "Updated title",
            "body": "Updated body",
            "state": "closed",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "labels": [],
            "assignees": [],
            "milestone": null,
            "comments": 0,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-22T13:34:00Z",
            "closed_at": "2011-04-22T13:34:00Z",
            "html_url": "https://github.com/octocat/Hello-World/issues/1347"
        });

        Mock::given(method("PATCH"))
            .and(path("/repos/octocat/Hello-World/issues/1347"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_issue_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = UpdateIssueRequest {
            title: Some("Updated title".to_string()),
            body: Some("Updated body".to_string()),
            state: Some("closed".to_string()),
            ..Default::default()
        };

        let result = client
            .issues()
            .update("octocat", "Hello-World", 1347, request)
            .await;

        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.title, "Updated title");
        assert_eq!(issue.state, "closed");
    }

    /// Verify set_issue_milestone sets a milestone on an issue.
    ///
    /// Tests PATCH /repos/{owner}/{repo}/issues/{number} with milestone field.
    #[tokio::test]
    async fn test_set_issue_milestone() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let updated_issue_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6SXNzdWUx",
            "number": 1347,
            "title": "Bug with milestone",
            "body": null,
            "state": "open",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "labels": [],
            "assignees": [],
            "milestone": {
                "id": 1,
                "node_id": "MDk6TWlsZXN0b25lMQ==",
                "number": 1,
                "title": "v1.0",
                "description": null,
                "state": "open",
                "open_issues": 1,
                "closed_issues": 0,
                "due_on": null,
                "created_at": "2011-04-10T20:09:31Z",
                "updated_at": "2011-04-10T20:09:31Z",
                "closed_at": null
            },
            "comments": 0,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-22T13:33:48Z",
            "closed_at": null,
            "html_url": "https://github.com/octocat/Hello-World/issues/1347"
        });

        Mock::given(method("PATCH"))
            .and(path("/repos/octocat/Hello-World/issues/1347"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_issue_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .set_milestone("octocat", "Hello-World", 1347, Some(1))
            .await;

        assert!(result.is_ok());
        let issue = result.unwrap();
        assert!(issue.milestone.is_some());
        assert_eq!(issue.milestone.unwrap().number, 1);
    }

    /// Verify set_issue_milestone with None clears the milestone.
    ///
    /// Tests PATCH /repos/{owner}/{repo}/issues/{number} with milestone=null.
    #[tokio::test]
    async fn test_clear_issue_milestone() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let updated_issue_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6SXNzdWUx",
            "number": 1347,
            "title": "Bug without milestone",
            "body": null,
            "state": "open",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "labels": [],
            "assignees": [],
            "milestone": null,
            "comments": 0,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-22T13:33:48Z",
            "closed_at": null,
            "html_url": "https://github.com/octocat/Hello-World/issues/1347"
        });

        Mock::given(method("PATCH"))
            .and(path("/repos/octocat/Hello-World/issues/1347"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_issue_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .set_milestone("octocat", "Hello-World", 1347, None)
            .await;

        assert!(result.is_ok());
        let issue = result.unwrap();
        assert!(issue.milestone.is_none());
    }
}

mod label_operations {
    use super::*;

    /// Verify list_labels returns all repository labels.
    ///
    /// Tests GET /repos/{owner}/{repo}/labels endpoint.
    #[tokio::test]
    async fn test_list_labels() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let labels_json = serde_json::json!([
            {
                "id": 1,
                "node_id": "MDU6TGFiZWwx",
                "name": "bug",
                "description": "Something isn't working",
                "color": "d73a4a",
                "default": true
            },
            {
                "id": 2,
                "node_id": "MDU6TGFiZWwy",
                "name": "enhancement",
                "description": "New feature or request",
                "color": "a2eeef",
                "default": true
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/labels"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(200).set_body_json(labels_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.labels().list("octocat", "Hello-World").await;

        assert!(result.is_ok());
        let labels = result.unwrap();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "bug");
        assert_eq!(labels[1].name, "enhancement");
    }

    /// Verify get_label returns a specific label by name.
    ///
    /// Tests GET /repos/{owner}/{repo}/labels/{name} endpoint.
    #[tokio::test]
    async fn test_get_label() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let label_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6TGFiZWwx",
            "name": "bug",
            "description": "Something isn't working",
            "color": "d73a4a",
            "default": true
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/labels/bug"))
            .respond_with(ResponseTemplate::new(200).set_body_json(label_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.labels().get("octocat", "Hello-World", "bug").await;

        assert!(result.is_ok());
        let label = result.unwrap();
        assert_eq!(label.name, "bug");
        assert_eq!(label.color, "d73a4a");
    }

    /// Verify get_label returns NotFound for non-existent label.
    #[tokio::test]
    async fn test_get_label_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/labels/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .labels()
            .get("octocat", "Hello-World", "nonexistent")
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    /// Verify create_label creates a new label.
    ///
    /// Tests POST /repos/{owner}/{repo}/labels endpoint.
    #[tokio::test]
    async fn test_create_label() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let created_label_json = serde_json::json!({
            "id": 3,
            "node_id": "MDU6TGFiZWwz",
            "name": "priority-high",
            "description": "High priority issue",
            "color": "ff0000",
            "default": false
        });

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/labels"))
            .respond_with(ResponseTemplate::new(201).set_body_json(created_label_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = CreateLabelRequest {
            name: "priority-high".to_string(),
            description: Some("High priority issue".to_string()),
            color: "ff0000".to_string(),
        };

        let result = client
            .labels()
            .create("octocat", "Hello-World", request)
            .await;

        assert!(result.is_ok());
        let label = result.unwrap();
        assert_eq!(label.name, "priority-high");
        assert_eq!(label.color, "ff0000");
    }

    /// Verify update_label modifies an existing label.
    ///
    /// Tests PATCH /repos/{owner}/{repo}/labels/{name} endpoint.
    #[tokio::test]
    async fn test_update_label() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let updated_label_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6TGFiZWwx",
            "name": "bug-critical",
            "description": "Critical bug requiring immediate attention",
            "color": "ff0000",
            "default": false
        });

        Mock::given(method("PATCH"))
            .and(path("/repos/octocat/Hello-World/labels/bug"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_label_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = UpdateLabelRequest {
            new_name: Some("bug-critical".to_string()),
            description: Some("Critical bug requiring immediate attention".to_string()),
            color: Some("ff0000".to_string()),
        };

        let result = client
            .labels()
            .update("octocat", "Hello-World", "bug", request)
            .await;

        assert!(result.is_ok());
        let label = result.unwrap();
        assert_eq!(label.name, "bug-critical");
        assert_eq!(label.color, "ff0000");
    }

    /// Verify delete_label removes a label.
    ///
    /// Tests DELETE /repos/{owner}/{repo}/labels/{name} endpoint.
    #[tokio::test]
    async fn test_delete_label() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("DELETE"))
            .and(path("/repos/octocat/Hello-World/labels/deprecated"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .labels()
            .delete("octocat", "Hello-World", "deprecated")
            .await;

        assert!(result.is_ok());
    }

    /// Verify add_labels_to_issue adds labels to an issue.
    ///
    /// Tests POST /repos/{owner}/{repo}/issues/{number}/labels endpoint.
    #[tokio::test]
    async fn test_add_labels_to_issue() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let updated_labels_json = serde_json::json!([
            {
                "id": 1,
                "node_id": "MDU6TGFiZWwx",
                "name": "bug",
                "description": "Something isn't working",
                "color": "d73a4a",
                "default": true
            },
            {
                "id": 2,
                "node_id": "MDU6TGFiZWwy",
                "name": "high-priority",
                "description": "High priority",
                "color": "ff0000",
                "default": false
            }
        ]);

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/issues/1347/labels"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_labels_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let labels = vec!["bug".to_string(), "high-priority".to_string()];

        let result = client
            .issues()
            .add_labels("octocat", "Hello-World", 1347, labels)
            .await;

        assert!(result.is_ok());
        let updated_labels = result.unwrap();
        assert_eq!(updated_labels.len(), 2);
        assert_eq!(updated_labels[0].name, "bug");
        assert_eq!(updated_labels[1].name, "high-priority");
    }

    /// Verify remove_label_from_issue removes a specific label from an issue.
    ///
    /// Tests DELETE /repos/{owner}/{repo}/issues/{number}/labels/{name} endpoint.
    #[tokio::test]
    async fn test_remove_label_from_issue() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let remaining_labels_json = serde_json::json!([
            {
                "id": 1,
                "node_id": "MDU6TGFiZWwx",
                "name": "bug",
                "description": "Something isn't working",
                "color": "d73a4a",
                "default": true
            }
        ]);

        Mock::given(method("DELETE"))
            .and(path(
                "/repos/octocat/Hello-World/issues/1347/labels/high-priority",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(remaining_labels_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .remove_label("octocat", "Hello-World", 1347, "high-priority")
            .await;

        assert!(result.is_ok());
        let remaining_labels = result.unwrap();
        assert_eq!(remaining_labels.len(), 1);
        assert_eq!(remaining_labels[0].name, "bug");
    }
}

mod comment_operations {
    use super::*;

    /// Verify list_issue_comments returns all comments on an issue.
    ///
    /// Tests GET /repos/{owner}/{repo}/issues/{number}/comments endpoint.
    #[tokio::test]
    async fn test_list_issue_comments() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let comments_json = serde_json::json!([
            {
                "id": 1,
                "node_id": "MDEyOklzc3VlQ29tbWVudDE=",
                "body": "Great idea!",
                "user": {
                    "login": "octocat",
                    "id": 1,
                    "node_id": "MDQ6VXNlcjE=",
                    "type": "User"
                },
                "created_at": "2011-04-14T16:00:49Z",
                "updated_at": "2011-04-14T16:00:49Z",
                "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-1"
            },
            {
                "id": 2,
                "node_id": "MDEyOklzc3VlQ29tbWVudDI=",
                "body": "I agree!",
                "user": {
                    "login": "hubot",
                    "id": 2,
                    "node_id": "MDQ6VXNlcjI=",
                    "type": "Bot"
                },
                "created_at": "2011-04-14T17:00:49Z",
                "updated_at": "2011-04-14T17:00:49Z",
                "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-2"
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/1347/comments"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(200).set_body_json(comments_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list_comments("octocat", "Hello-World", 1347)
            .await;

        assert!(result.is_ok());
        let comments = result.unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].id, 1);
        assert_eq!(comments[0].body, "Great idea!");
        assert_eq!(comments[1].id, 2);
    }

    /// Verify get_issue_comment returns a specific comment by ID.
    ///
    /// Tests GET /repos/{owner}/{repo}/issues/comments/{id} endpoint.
    #[tokio::test]
    async fn test_get_issue_comment() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let comment_json = serde_json::json!({
            "id": 1,
            "node_id": "MDEyOklzc3VlQ29tbWVudDE=",
            "body": "Great idea!",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2011-04-14T16:00:49Z",
            "updated_at": "2011-04-14T16:00:49Z",
            "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-1"
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/comments/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(comment_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .get_comment("octocat", "Hello-World", 1)
            .await;

        assert!(result.is_ok());
        let comment = result.unwrap();
        assert_eq!(comment.id, 1);
        assert_eq!(comment.body, "Great idea!");
    }

    /// Verify get_issue_comment returns NotFound for non-existent comment.
    #[tokio::test]
    async fn test_get_issue_comment_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/comments/9999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .get_comment("octocat", "Hello-World", 9999)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    /// Verify create_issue_comment creates a new comment.
    ///
    /// Tests POST /repos/{owner}/{repo}/issues/{number}/comments endpoint.
    #[tokio::test]
    async fn test_create_issue_comment() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let created_comment_json = serde_json::json!({
            "id": 3,
            "node_id": "MDEyOklzc3VlQ29tbWVudDM=",
            "body": "This is a new comment",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2011-04-14T18:00:49Z",
            "updated_at": "2011-04-14T18:00:49Z",
            "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-3"
        });

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/issues/1347/comments"))
            .respond_with(ResponseTemplate::new(201).set_body_json(created_comment_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = CreateCommentRequest {
            body: "This is a new comment".to_string(),
        };

        let result = client
            .issues()
            .create_comment("octocat", "Hello-World", 1347, request)
            .await;

        assert!(result.is_ok());
        let comment = result.unwrap();
        assert_eq!(comment.id, 3);
        assert_eq!(comment.body, "This is a new comment");
    }

    /// Verify update_issue_comment modifies an existing comment.
    ///
    /// Tests PATCH /repos/{owner}/{repo}/issues/comments/{id} endpoint.
    #[tokio::test]
    async fn test_update_issue_comment() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let updated_comment_json = serde_json::json!({
            "id": 1,
            "node_id": "MDEyOklzc3VlQ29tbWVudDE=",
            "body": "Updated comment text",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2011-04-14T16:00:49Z",
            "updated_at": "2011-04-14T19:00:49Z",
            "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-1"
        });

        Mock::given(method("PATCH"))
            .and(path("/repos/octocat/Hello-World/issues/comments/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_comment_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = UpdateCommentRequest {
            body: "Updated comment text".to_string(),
        };

        let result = client
            .issues()
            .update_comment("octocat", "Hello-World", 1, request)
            .await;

        assert!(result.is_ok());
        let comment = result.unwrap();
        assert_eq!(comment.body, "Updated comment text");
    }

    /// Verify delete_issue_comment removes a comment.
    ///
    /// Tests DELETE /repos/{owner}/{repo}/issues/comments/{id} endpoint.
    #[tokio::test]
    async fn test_delete_issue_comment() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("DELETE"))
            .and(path("/repos/octocat/Hello-World/issues/comments/1"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .delete_comment("octocat", "Hello-World", 1)
            .await;

        assert!(result.is_ok());
    }

    /// Verify list_comments returns an empty vec when the issue has no comments.
    #[tokio::test]
    async fn test_list_issue_comments_empty() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/1347/comments"))
            .and(query_param("per_page", "100"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list_comments("octocat", "Hello-World", 1347)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Verify list_comments follows Link: rel="next" headers and returns all pages.
    #[tokio::test]
    async fn test_list_issue_comments_multi_page() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let page1_json = serde_json::json!([
            {
                "id": 1,
                "node_id": "MDEyOklzc3VlQ29tbWVudDE=",
                "body": "First comment",
                "user": { "login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User" },
                "created_at": "2011-04-14T16:00:49Z",
                "updated_at": "2011-04-14T16:00:49Z",
                "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-1"
            },
            {
                "id": 2,
                "node_id": "MDEyOklzc3VlQ29tbWVudDI=",
                "body": "Second comment",
                "user": { "login": "hubot", "id": 2, "node_id": "MDQ6VXNlcjI=", "type": "Bot" },
                "created_at": "2011-04-14T17:00:49Z",
                "updated_at": "2011-04-14T17:00:49Z",
                "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-2"
            }
        ]);

        let page2_json = serde_json::json!([
            {
                "id": 3,
                "node_id": "MDEyOklzc3VlQ29tbWVudDM=",
                "body": "Third comment",
                "user": { "login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User" },
                "created_at": "2011-04-14T18:00:49Z",
                "updated_at": "2011-04-14T18:00:49Z",
                "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-3"
            }
        ]);

        let link_header = r#"<https://api.github.com/repos/octocat/Hello-World/issues/1347/comments?per_page=100&page=2>; rel="next""#;

        // Page 1: return two comments with a Link header pointing to page 2.
        // `query_param_is_missing("page")` prevents this mock from also matching
        // page-2 requests (which would create an infinite pagination loop).
        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/1347/comments"))
            .and(query_param("per_page", "100"))
            .and(query_param_is_missing("page"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Link", link_header)
                    .set_body_json(page1_json),
            )
            .mount(&mock_server)
            .await;

        // Page 2: return one comment with no further Link header.
        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/1347/comments"))
            .and(query_param("per_page", "100"))
            .and(query_param("page", "2"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(200).set_body_json(page2_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list_comments("octocat", "Hello-World", 1347)
            .await;

        assert!(result.is_ok());
        let comments = result.unwrap();
        assert_eq!(comments.len(), 3);
        assert_eq!(comments[0].id, 1);
        assert_eq!(comments[1].id, 2);
        assert_eq!(comments[2].id, 3);
        // Verify ascending created_at order (GitHub default).
        assert!(comments[0].created_at < comments[1].created_at);
        assert!(comments[1].created_at < comments[2].created_at);
    }

    /// Verify list_comments returns NotFound when the issue does not exist.
    #[tokio::test]
    async fn test_list_issue_comments_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/9999/comments"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest/issues/comments"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list_comments("octocat", "Hello-World", 9999)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }
}

mod serialization {
    use super::*;

    /// Verify Issue can be deserialized from GitHub API response.
    ///
    /// Tests that the Issue type correctly deserializes from GitHub's API JSON format
    /// including all fields like id, number, title, state, labels, etc.
    #[test]
    fn test_issue_deserialize() {
        let json = r#"{
            "id": 1,
            "node_id": "MDU6SXNzdWUx",
            "number": 1347,
            "title": "Found a bug",
            "body": "I'm having a problem with this.",
            "state": "open",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "labels": [
                {
                    "id": 208045946,
                    "node_id": "MDU6TGFiZWwyMDgwNDU5NDY=",
                    "name": "bug",
                    "description": "Something isn't working",
                    "color": "d73a4a",
                    "default": true
                }
            ],
            "assignees": [],
            "milestone": null,
            "comments": 0,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-22T13:33:48Z",
            "closed_at": null,
            "html_url": "https://github.com/octocat/Hello-World/issues/1347"
        }"#;

        let issue: Issue = serde_json::from_str(json).unwrap();

        assert_eq!(issue.id, 1);
        assert_eq!(issue.node_id, "MDU6SXNzdWUx");
        assert_eq!(issue.number, 1347);
        assert_eq!(issue.title, "Found a bug");
        assert_eq!(
            issue.body,
            Some("I'm having a problem with this.".to_string())
        );
        assert_eq!(issue.state, "open");
        assert_eq!(issue.user.login, "octocat");
        assert_eq!(issue.labels.len(), 1);
        assert_eq!(issue.labels[0].name, "bug");
        assert_eq!(issue.comments, 0);
        assert!(issue.closed_at.is_none());
    }

    /// Verify Issue deserializes with closed state and timestamp.
    #[test]
    fn test_issue_deserialize_closed() {
        let json = r#"{
            "id": 2,
            "node_id": "MDU6SXNzdWUy",
            "number": 1348,
            "title": "Fixed bug",
            "body": null,
            "state": "closed",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "labels": [],
            "assignees": [],
            "milestone": null,
            "comments": 5,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-23T13:33:48Z",
            "closed_at": "2011-04-23T13:33:48Z",
            "html_url": "https://github.com/octocat/Hello-World/issues/1348"
        }"#;

        let issue: Issue = serde_json::from_str(json).unwrap();

        assert_eq!(issue.state, "closed");
        assert_eq!(issue.body, None);
        assert!(issue.closed_at.is_some());
        assert_eq!(issue.comments, 5);
    }

    /// Verify Label can be deserialized from GitHub API response.
    #[test]
    fn test_label_deserialize() {
        let json = r#"{
            "id": 208045946,
            "node_id": "MDU6TGFiZWwyMDgwNDU5NDY=",
            "name": "bug",
            "description": "Something isn't working",
            "color": "d73a4a",
            "default": true
        }"#;

        let label: Label = serde_json::from_str(json).unwrap();

        assert_eq!(label.id, 208045946);
        assert_eq!(label.node_id, "MDU6TGFiZWwyMDgwNDU5NDY=");
        assert_eq!(label.name, "bug");
        assert_eq!(
            label.description,
            Some("Something isn't working".to_string())
        );
        assert_eq!(label.color, "d73a4a");
        assert!(label.default);
    }

    /// Verify Comment can be deserialized from GitHub API response.
    #[test]
    fn test_comment_deserialize() {
        let json = r#"{
            "id": 1,
            "node_id": "MDEyOklzc3VlQ29tbWVudDE=",
            "body": "Great idea!",
            "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "type": "User"
            },
            "created_at": "2011-04-14T16:00:49Z",
            "updated_at": "2011-04-14T16:00:49Z",
            "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-1"
        }"#;

        let comment: Comment = serde_json::from_str(json).unwrap();

        assert_eq!(comment.id, 1);
        assert_eq!(comment.node_id, "MDEyOklzc3VlQ29tbWVudDE=");
        assert_eq!(comment.body, "Great idea!");
        assert_eq!(comment.user.login, "octocat");
        assert_eq!(
            comment.html_url,
            "https://github.com/octocat/Hello-World/issues/1347#issuecomment-1"
        );
    }

    /// Verify Milestone can be deserialized from GitHub API response.
    #[test]
    fn test_milestone_deserialize() {
        let json = r#"{
            "id": 1002604,
            "node_id": "MDk6TWlsZXN0b25lMTAwMjYwNA==",
            "number": 1,
            "title": "v1.0",
            "description": "Tracking milestone for version 1.0",
            "state": "open",
            "open_issues": 4,
            "closed_issues": 8,
            "due_on": "2012-10-09T23:39:01Z",
            "created_at": "2011-04-10T20:09:31Z",
            "updated_at": "2014-03-03T18:58:10Z",
            "closed_at": null
        }"#;

        let milestone: Milestone = serde_json::from_str(json).unwrap();

        assert_eq!(milestone.id, 1002604);
        assert_eq!(milestone.node_id, "MDk6TWlsZXN0b25lMTAwMjYwNA==");
        assert_eq!(milestone.number, 1);
        assert_eq!(milestone.title, "v1.0");
        assert_eq!(
            milestone.description,
            Some("Tracking milestone for version 1.0".to_string())
        );
        assert!(matches!(milestone.state, MilestoneState::Open));
        assert_eq!(milestone.open_issues, 4);
        assert_eq!(milestone.closed_issues, 8);
        assert!(milestone.due_on.is_some());
        assert!(milestone.closed_at.is_none());
    }

    /// Verify CreateIssueRequest serializes correctly.
    #[test]
    fn test_create_issue_request_serialize() {
        let request = CreateIssueRequest {
            title: "Found a bug".to_string(),
            body: Some("I'm having a problem".to_string()),
            assignees: Some(vec!["octocat".to_string()]),
            milestone: Some(1),
            labels: Some(vec!["bug".to_string(), "high-priority".to_string()]),
        };

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["title"], "Found a bug");
        assert_eq!(json["body"], "I'm having a problem");
        assert_eq!(json["assignees"][0], "octocat");
        assert_eq!(json["milestone"], 1);
        assert_eq!(json["labels"].as_array().unwrap().len(), 2);
    }

    /// Verify UpdateIssueRequest skips None fields.
    #[test]
    fn test_update_issue_request_serialize_partial() {
        let request = UpdateIssueRequest {
            title: Some("Updated title".to_string()),
            state: Some("closed".to_string()),
            body: None,
            assignees: None,
            milestone: None,
            labels: None,
        };

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["title"], "Updated title");
        assert_eq!(json["state"], "closed");
        // None fields should not be present
        assert!(json.get("body").is_none());
        assert!(json.get("assignees").is_none());
        assert!(json.get("milestone").is_none());
        assert!(json.get("labels").is_none());
    }
}

mod error_handling {
    use super::*;

    /// Verify get_issue returns NotFound error for 404 response.
    #[tokio::test]
    async fn test_issue_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/issues/999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest/issues/issues#get-an-issue"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.issues().get("owner", "repo", 999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    /// Verify issue operations return AuthorizationFailed error for 403 response.
    #[tokio::test]
    async fn test_forbidden_access() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/owner/private-repo/issues/1"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "message": "Resource not accessible by integration",
                "documentation_url": "https://docs.github.com/rest/issues/issues#get-an-issue"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.issues().get("owner", "private-repo", 1).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::AuthorizationFailed));
    }

    /// Verify create_issue returns InvalidRequest error for 422 response.
    #[tokio::test]
    async fn test_validation_error() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path("/repos/owner/repo/issues"))
            .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
                "message": "Validation Failed",
                "errors": [
                    {
                        "resource": "Issue",
                        "field": "title",
                        "code": "missing_field"
                    }
                ],
                "documentation_url": "https://docs.github.com/rest/issues/issues#create-an-issue"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = CreateIssueRequest {
            title: "".to_string(), // Invalid: empty title
            body: None,
            assignees: None,
            milestone: None,
            labels: None,
        };

        let result = client.issues().create("owner", "repo", request).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ApiError::InvalidRequest { message } => {
                assert!(message.contains("Validation Failed"));
            }
            _ => panic!("Expected InvalidRequest error, got: {:?}", error),
        }
    }
}

mod reaction_operations {
    use super::*;

    /// Verify list_reactions returns all reactions on an issue.
    ///
    /// Tests GET /repos/{owner}/{repo}/issues/{number}/reactions endpoint.
    #[tokio::test]
    async fn test_list_reactions() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let reactions_json = serde_json::json!([
            {
                "id": 1,
                "node_id": "MDg6UmVhY3Rpb24x",
                "user": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
                "content": "+1",
                "created_at": "2016-05-20T20:09:31Z"
            },
            {
                "id": 2,
                "node_id": "MDg6UmVhY3Rpb24y",
                "user": {"login": "hubot", "id": 2, "node_id": "MDQ6VXNlcjI=", "type": "Bot"},
                "content": "laugh",
                "created_at": "2016-05-20T20:09:32Z"
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/1347/reactions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(reactions_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list_reactions("octocat", "Hello-World", 1347)
            .await;

        assert!(result.is_ok());
        let reactions = result.unwrap();
        assert_eq!(reactions.len(), 2);
        assert_eq!(reactions[0].id, 1);
        assert!(matches!(reactions[0].content, ReactionContent::PlusOne));
    }

    /// Verify create_reaction adds a reaction to an issue and returns it.
    ///
    /// Tests POST /repos/{owner}/{repo}/issues/{number}/reactions endpoint.
    #[tokio::test]
    async fn test_create_reaction() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let reaction_json = serde_json::json!({
            "id": 1,
            "node_id": "MDg6UmVhY3Rpb24x",
            "user": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
            "content": "+1",
            "created_at": "2016-05-20T20:09:31Z"
        });

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/issues/1347/reactions"))
            .respond_with(ResponseTemplate::new(201).set_body_json(reaction_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .create_reaction("octocat", "Hello-World", 1347, ReactionContent::PlusOne)
            .await;

        assert!(result.is_ok());
        let reaction = result.unwrap();
        assert_eq!(reaction.id, 1);
        assert!(matches!(reaction.content, ReactionContent::PlusOne));
    }

    /// Verify delete_reaction removes a reaction from an issue.
    ///
    /// Tests DELETE /repos/{owner}/{repo}/issues/{number}/reactions/{id} endpoint.
    #[tokio::test]
    async fn test_delete_reaction() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("DELETE"))
            .and(path("/repos/octocat/Hello-World/issues/1347/reactions/1"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .delete_reaction("octocat", "Hello-World", 1347, 1)
            .await;

        assert!(result.is_ok());
    }

    /// Verify list_comment_reactions returns all reactions on an issue comment.
    ///
    /// Tests GET /repos/{owner}/{repo}/issues/comments/{id}/reactions endpoint.
    #[tokio::test]
    async fn test_list_comment_reactions() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let reactions_json = serde_json::json!([
            {
                "id": 3,
                "node_id": "MDg6UmVhY3Rpb24z",
                "user": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
                "content": "heart",
                "created_at": "2016-05-20T20:09:33Z"
            }
        ]);

        Mock::given(method("GET"))
            .and(path(
                "/repos/octocat/Hello-World/issues/comments/42/reactions",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(reactions_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list_comment_reactions("octocat", "Hello-World", 42)
            .await;

        assert!(result.is_ok());
        let reactions = result.unwrap();
        assert_eq!(reactions.len(), 1);
        assert!(matches!(reactions[0].content, ReactionContent::Heart));
    }

    /// Verify create_comment_reaction adds a reaction to an issue comment.
    ///
    /// Tests POST /repos/{owner}/{repo}/issues/comments/{id}/reactions endpoint.
    #[tokio::test]
    async fn test_create_comment_reaction() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let reaction_json = serde_json::json!({
            "id": 3,
            "node_id": "MDg6UmVhY3Rpb24z",
            "user": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
            "content": "heart",
            "created_at": "2016-05-20T20:09:33Z"
        });

        Mock::given(method("POST"))
            .and(path(
                "/repos/octocat/Hello-World/issues/comments/42/reactions",
            ))
            .respond_with(ResponseTemplate::new(201).set_body_json(reaction_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .create_comment_reaction("octocat", "Hello-World", 42, ReactionContent::Heart)
            .await;

        assert!(result.is_ok());
        let reaction = result.unwrap();
        assert_eq!(reaction.id, 3);
        assert!(matches!(reaction.content, ReactionContent::Heart));
    }

    /// Verify delete_comment_reaction removes a reaction from an issue comment.
    ///
    /// Tests DELETE /repos/{owner}/{repo}/issues/comments/{id}/reactions/{reaction_id} endpoint.
    #[tokio::test]
    async fn test_delete_comment_reaction() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("DELETE"))
            .and(path(
                "/repos/octocat/Hello-World/issues/comments/42/reactions/3",
            ))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .delete_comment_reaction("octocat", "Hello-World", 42, 3)
            .await;

        assert!(result.is_ok());
    }
}

mod assignee_operations {
    use super::*;

    /// Verify list_available_assignees returns users eligible to be assigned in the repository.
    ///
    /// Tests GET /repos/{owner}/{repo}/assignees endpoint.
    #[tokio::test]
    async fn test_list_available_assignees() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let assignees_json = serde_json::json!([
            {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
            {"login": "hubot", "id": 2, "node_id": "MDQ6VXNlcjI=", "type": "Bot"}
        ]);

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/assignees"))
            .respond_with(ResponseTemplate::new(200).set_body_json(assignees_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list_available_assignees("octocat", "Hello-World")
            .await;

        assert!(result.is_ok());
        let assignees = result.unwrap();
        assert_eq!(assignees.len(), 2);
        assert_eq!(assignees[0].login, "octocat");
    }

    /// Verify add_assignees assigns users to an issue and returns the updated issue.
    ///
    /// Tests POST /repos/{owner}/{repo}/issues/{number}/assignees endpoint.
    #[tokio::test]
    async fn test_add_assignees() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let issue_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6SXNzdWUx",
            "number": 1347,
            "title": "Found a bug",
            "body": "I'm having a problem with this.",
            "state": "open",
            "user": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
            "labels": [],
            "assignees": [
                {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"}
            ],
            "milestone": null,
            "comments": 0,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-22T13:33:48Z",
            "closed_at": null,
            "html_url": "https://github.com/octocat/Hello-World/issues/1347"
        });

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/issues/1347/assignees"))
            .respond_with(ResponseTemplate::new(201).set_body_json(issue_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .add_assignees("octocat", "Hello-World", 1347, vec!["octocat".to_string()])
            .await;

        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.assignees.len(), 1);
        assert_eq!(issue.assignees[0].login, "octocat");
    }

    /// Verify remove_assignees unassigns users from an issue and returns the updated issue.
    ///
    /// Tests DELETE /repos/{owner}/{repo}/issues/{number}/assignees endpoint.
    #[tokio::test]
    async fn test_remove_assignees() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let issue_json = serde_json::json!({
            "id": 1,
            "node_id": "MDU6SXNzdWUx",
            "number": 1347,
            "title": "Found a bug",
            "body": "I'm having a problem with this.",
            "state": "open",
            "user": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
            "labels": [],
            "assignees": [],
            "milestone": null,
            "comments": 0,
            "created_at": "2011-04-22T13:33:48Z",
            "updated_at": "2011-04-22T13:33:48Z",
            "closed_at": null,
            "html_url": "https://github.com/octocat/Hello-World/issues/1347"
        });

        Mock::given(method("DELETE"))
            .and(path("/repos/octocat/Hello-World/issues/1347/assignees"))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .remove_assignees("octocat", "Hello-World", 1347, vec!["octocat".to_string()])
            .await;

        assert!(result.is_ok());
        let issue = result.unwrap();
        assert!(issue.assignees.is_empty());
    }
}

mod lock_operations {
    use super::*;

    /// Verify lock locks an issue without a reason.
    ///
    /// Tests PUT /repos/{owner}/{repo}/issues/{number}/lock endpoint.
    #[tokio::test]
    async fn test_lock_issue() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("PUT"))
            .and(path("/repos/octocat/Hello-World/issues/1347/lock"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .lock("octocat", "Hello-World", 1347, None)
            .await;

        assert!(result.is_ok());
    }

    /// Verify lock locks an issue with a specific lock reason.
    ///
    /// Tests PUT /repos/{owner}/{repo}/issues/{number}/lock endpoint with reason body.
    #[tokio::test]
    async fn test_lock_issue_with_reason() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("PUT"))
            .and(path("/repos/octocat/Hello-World/issues/1347/lock"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .lock("octocat", "Hello-World", 1347, Some(LockReason::TooHeated))
            .await;

        assert!(result.is_ok());
    }

    /// Verify unlock removes the lock from an issue.
    ///
    /// Tests DELETE /repos/{owner}/{repo}/issues/{number}/lock endpoint.
    #[tokio::test]
    async fn test_unlock_issue() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("DELETE"))
            .and(path("/repos/octocat/Hello-World/issues/1347/lock"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.issues().unlock("octocat", "Hello-World", 1347).await;

        assert!(result.is_ok());
    }

    /// Verify lock returns AuthorizationFailed when the caller lacks permission.
    ///
    /// Tests PUT /repos/{owner}/{repo}/issues/{number}/lock returning 403.
    #[tokio::test]
    async fn test_lock_unauthorized() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("PUT"))
            .and(path("/repos/octocat/Hello-World/issues/1347/lock"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "message": "Resource not accessible by integration",
                "documentation_url": "https://docs.github.com/rest/issues/issues#lock-an-issue"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .lock("octocat", "Hello-World", 1347, None)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::AuthorizationFailed));
    }
}

mod activity_operations {
    use super::*;

    /// Verify list_activity_events returns discrete activity events on an issue.
    ///
    /// Tests GET /repos/{owner}/{repo}/issues/{number}/events endpoint.
    #[tokio::test]
    async fn test_list_activity_events() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let events_json = serde_json::json!([
            {
                "id": 6430295168u64,
                "event": "labeled",
                "actor": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
                "label": {
                    "id": 1,
                    "node_id": "MDU6TGFiZWwx",
                    "name": "bug",
                    "description": null,
                    "color": "d73a4a",
                    "default": false
                },
                "created_at": "2022-03-10T14:00:00Z"
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/1347/events"))
            .respond_with(ResponseTemplate::new(200).set_body_json(events_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list_activity_events("octocat", "Hello-World", 1347)
            .await;

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, "labeled");
    }

    /// Verify list_timeline returns the full timeline including unknown event kinds.
    ///
    /// Tests GET /repos/{owner}/{repo}/issues/{number}/timeline endpoint.
    /// Unknown event kinds must deserialize to TimelineEvent::Unknown without error.
    #[tokio::test]
    async fn test_list_timeline() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let timeline_json = serde_json::json!([
            {
                "event": "labeled",
                "id": 1,
                "actor": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
                "label": {
                    "id": 1,
                    "node_id": "MDU6TGFiZWwx",
                    "name": "bug",
                    "description": null,
                    "color": "d73a4a",
                    "default": false
                },
                "created_at": "2022-03-10T14:00:00Z"
            },
            {
                "event": "commented",
                "id": 2,
                "user": {"login": "octocat", "id": 1, "node_id": "MDQ6VXNlcjE=", "type": "User"},
                "body": "This is a comment",
                "created_at": "2022-03-10T15:00:00Z",
                "updated_at": "2022-03-10T15:00:00Z",
                "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-2"
            },
            {
                "event": "commented",
                "id": 3,
                "user": {"login": "monalisa", "id": 2, "node_id": "MDQ6VXNlcjI=", "type": "User"},
                "body": null,
                "created_at": "2022-03-10T16:00:00Z",
                "updated_at": "2022-03-10T16:00:00Z",
                "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-3"
            },
            {
                "event": "totally_unknown_future_event"
            }
        ]);

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/1347/timeline"))
            .respond_with(ResponseTemplate::new(200).set_body_json(timeline_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .issues()
            .list_timeline("octocat", "Hello-World", 1347)
            .await;

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 4);

        // Verify the commented event with body
        if let crate::client::issue::TimelineEvent::Commented {
            id,
            user,
            body,
            html_url,
            ..
        } = &events[1]
        {
            assert_eq!(*id, 2);
            assert_eq!(user.login, "octocat");
            assert_eq!(body.as_deref(), Some("This is a comment"));
            assert_eq!(
                html_url,
                "https://github.com/octocat/Hello-World/issues/1347#issuecomment-2"
            );
        } else {
            panic!("Expected Commented event at index 1");
        }

        // Verify the commented event with null body
        if let crate::client::issue::TimelineEvent::Commented { body, .. } = &events[2] {
            assert!(body.is_none());
        } else {
            panic!("Expected Commented event at index 2");
        }
    }
}

mod milestone_operations {
    use super::*;

    fn milestone_json() -> serde_json::Value {
        serde_json::json!({
            "id": 1,
            "node_id": "MDk6TWlsZXN0b25lMQ==",
            "number": 1,
            "title": "v1.0",
            "description": "Tracking milestone for v1.0 release",
            "state": "open",
            "due_on": null,
            "open_issues": 5,
            "closed_issues": 2,
            "created_at": "2013-02-12T13:22:01Z",
            "updated_at": "2013-02-12T13:22:01Z",
            "closed_at": null
        })
    }

    /// Verify list returns all milestones in a repository.
    ///
    /// Tests GET /repos/{owner}/{repo}/milestones endpoint.
    #[tokio::test]
    async fn test_list_milestones() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/milestones"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!([milestone_json()])),
            )
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .milestones()
            .list("octocat", "Hello-World", None)
            .await;

        assert!(result.is_ok());
        let milestones = result.unwrap();
        assert_eq!(milestones.len(), 1);
        assert_eq!(milestones[0].title, "v1.0");
    }

    /// Verify get returns a single milestone by its number.
    ///
    /// Tests GET /repos/{owner}/{repo}/milestones/{number} endpoint.
    #[tokio::test]
    async fn test_get_milestone() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/milestones/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(milestone_json()))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.milestones().get("octocat", "Hello-World", 1).await;

        assert!(result.is_ok());
        let milestone = result.unwrap();
        assert_eq!(milestone.number, 1);
        assert_eq!(milestone.title, "v1.0");
        assert!(matches!(milestone.state, MilestoneState::Open));
    }

    /// Verify get returns NotFound error for a non-existent milestone.
    ///
    /// Tests GET /repos/{owner}/{repo}/milestones/{number} returning 404.
    #[tokio::test]
    async fn test_get_milestone_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/milestones/999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest/issues/milestones#get-a-milestone"
            })))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client.milestones().get("octocat", "Hello-World", 999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    /// Verify create creates a new milestone and returns it.
    ///
    /// Tests POST /repos/{owner}/{repo}/milestones endpoint.
    #[tokio::test]
    async fn test_create_milestone() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path("/repos/octocat/Hello-World/milestones"))
            .respond_with(ResponseTemplate::new(201).set_body_json(milestone_json()))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = CreateMilestoneRequest {
            title: "v1.0".to_string(),
            state: None,
            description: None,
            due_on: None,
        };

        let result = client
            .milestones()
            .create("octocat", "Hello-World", request)
            .await;

        assert!(result.is_ok());
        let milestone = result.unwrap();
        assert_eq!(milestone.title, "v1.0");
    }

    /// Verify update patches an existing milestone and returns the updated version.
    ///
    /// Tests PATCH /repos/{owner}/{repo}/milestones/{number} endpoint.
    #[tokio::test]
    async fn test_update_milestone() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let updated_json = serde_json::json!({
            "id": 1,
            "node_id": "MDk6TWlsZXN0b25lMQ==",
            "number": 1,
            "title": "v1.1",
            "description": "Tracking milestone for v1.0 release",
            "state": "open",
            "due_on": null,
            "open_issues": 5,
            "closed_issues": 2,
            "created_at": "2013-02-12T13:22:01Z",
            "updated_at": "2013-02-12T13:22:01Z",
            "closed_at": null
        });

        Mock::given(method("PATCH"))
            .and(path("/repos/octocat/Hello-World/milestones/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_json))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let request = UpdateMilestoneRequest {
            title: Some("v1.1".to_string()),
            ..Default::default()
        };

        let result = client
            .milestones()
            .update("octocat", "Hello-World", 1, request)
            .await;

        assert!(result.is_ok());
        let milestone = result.unwrap();
        assert_eq!(milestone.title, "v1.1");
    }

    /// Verify delete removes a milestone.
    ///
    /// Tests DELETE /repos/{owner}/{repo}/milestones/{number} endpoint.
    #[tokio::test]
    async fn test_delete_milestone() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("DELETE"))
            .and(path("/repos/octocat/Hello-World/milestones/1"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let auth = MockAuthProvider::new_with_token(test_token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();

        let client = github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap();

        let result = client
            .milestones()
            .delete("octocat", "Hello-World", 1)
            .await;

        assert!(result.is_ok());
    }
}
