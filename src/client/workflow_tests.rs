//! Tests for workflow operations.

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
    use super::*;

    /// Verify TriggerWorkflowRequest constructs correctly with only ref.
    ///
    /// Tests that TriggerWorkflowRequest can be created with minimal required
    /// fields (ref only, no inputs).
    #[test]
    fn test_trigger_workflow_request_minimal() {
        let request = TriggerWorkflowRequest {
            git_ref: "main".to_string(),
            inputs: None,
        };

        assert_eq!(request.git_ref, "main");
        assert!(request.inputs.is_none());
    }

    /// Verify TriggerWorkflowRequest constructs correctly with inputs.
    ///
    /// Tests that TriggerWorkflowRequest can be created with inputs map
    /// for passing parameters to the workflow.
    #[test]
    fn test_trigger_workflow_request_with_inputs() {
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("environment".to_string(), "production".to_string());
        inputs.insert("version".to_string(), "v1.2.3".to_string());

        let request = TriggerWorkflowRequest {
            git_ref: "release-branch".to_string(),
            inputs: Some(inputs.clone()),
        };

        assert_eq!(request.git_ref, "release-branch");
        assert!(request.inputs.is_some());
        let request_inputs = request.inputs.unwrap();
        assert_eq!(
            request_inputs.get("environment"),
            Some(&"production".to_string())
        );
        assert_eq!(request_inputs.get("version"), Some(&"v1.2.3".to_string()));
    }
}

mod workflow_operations {
    use super::*;

    /// Verify list_workflows fetches all workflows from repository.
    ///
    /// Tests GET /repos/:owner/:repo/actions/workflows endpoint.
    #[tokio::test]
    async fn test_list_workflows() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let workflows_json = serde_json::json!({
            "total_count": 2,
            "workflows": [
                {
                    "id": 161335,
                    "node_id": "MDg6V29ya2Zsb3cxNjEzMzU=",
                    "name": "CI",
                    "path": ".github/workflows/ci.yml",
                    "state": "active",
                    "created_at": "2020-01-08T23:48:37.000-08:00",
                    "updated_at": "2020-01-08T23:50:21.000-08:00",
                    "url": "https://api.github.com/repos/octocat/Hello-World/actions/workflows/161335",
                    "html_url": "https://github.com/octocat/Hello-World/blob/master/.github/workflows/ci.yml",
                    "badge_url": "https://github.com/octocat/Hello-World/workflows/CI/badge.svg"
                },
                {
                    "id": 269289,
                    "node_id": "MDE4OldvcmtmbG93MjY5Mjg5",
                    "name": "Deploy",
                    "path": ".github/workflows/deploy.yml",
                    "state": "active",
                    "created_at": "2020-02-15T10:30:00.000-08:00",
                    "updated_at": "2020-02-15T10:30:00.000-08:00",
                    "url": "https://api.github.com/repos/octocat/Hello-World/actions/workflows/269289",
                    "html_url": "https://github.com/octocat/Hello-World/blob/master/.github/workflows/deploy.yml",
                    "badge_url": "https://github.com/octocat/Hello-World/workflows/Deploy/badge.svg"
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/actions/workflows"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(workflows_json))
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

        let result = client.list_workflows("octocat", "Hello-World").await;

        assert!(result.is_ok());
        let workflows = result.unwrap();
        assert_eq!(workflows.len(), 2);
        assert_eq!(workflows[0].id, 161335);
        assert_eq!(workflows[0].name, "CI");
        assert_eq!(workflows[0].state, "active");
        assert_eq!(workflows[1].id, 269289);
        assert_eq!(workflows[1].name, "Deploy");
    }

    /// Verify get_workflow fetches a specific workflow by ID.
    ///
    /// Tests GET /repos/:owner/:repo/actions/workflows/:id endpoint.
    #[tokio::test]
    async fn test_get_workflow() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let workflow_json = serde_json::json!({
            "id": 161335,
            "node_id": "MDg6V29ya2Zsb3cxNjEzMzU=",
            "name": "CI",
            "path": ".github/workflows/ci.yml",
            "state": "active",
            "created_at": "2020-01-08T23:48:37.000-08:00",
            "updated_at": "2020-01-08T23:50:21.000-08:00",
            "url": "https://api.github.com/repos/octocat/Hello-World/actions/workflows/161335",
            "html_url": "https://github.com/octocat/Hello-World/blob/master/.github/workflows/ci.yml",
            "badge_url": "https://github.com/octocat/Hello-World/workflows/CI/badge.svg"
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/actions/workflows/161335"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(workflow_json))
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

        let result = client.get_workflow("octocat", "Hello-World", 161335).await;

        assert!(result.is_ok());
        let workflow = result.unwrap();
        assert_eq!(workflow.id, 161335);
        assert_eq!(workflow.name, "CI");
        assert_eq!(workflow.path, ".github/workflows/ci.yml");
        assert_eq!(workflow.state, "active");
    }

    /// Verify get_workflow returns NotFound for non-existent workflow.
    ///
    /// Tests 404 response from GET /repos/:owner/:repo/actions/workflows/:id.
    #[tokio::test]
    async fn test_get_workflow_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/actions/workflows/999999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest/actions/workflows#get-a-workflow"
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

        let result = client.get_workflow("octocat", "Hello-World", 999999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    /// Verify trigger_workflow dispatches a workflow run.
    ///
    /// Tests POST /repos/:owner/:repo/actions/workflows/:id/dispatches endpoint.
    #[tokio::test]
    async fn test_trigger_workflow() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path(
                "/repos/octocat/Hello-World/actions/workflows/161335/dispatches",
            ))
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

        let request = TriggerWorkflowRequest {
            git_ref: "main".to_string(),
            inputs: None,
        };

        let result = client
            .trigger_workflow("octocat", "Hello-World", 161335, request)
            .await;

        assert!(result.is_ok());
    }

    /// Verify trigger_workflow with inputs parameter.
    ///
    /// Tests POST /repos/:owner/:repo/actions/workflows/:id/dispatches with inputs.
    #[tokio::test]
    async fn test_trigger_workflow_with_inputs() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path(
                "/repos/octocat/Hello-World/actions/workflows/161335/dispatches",
            ))
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

        let mut inputs = std::collections::HashMap::new();
        inputs.insert("name".to_string(), "Mona the Octocat".to_string());
        inputs.insert("home".to_string(), "San Francisco, CA".to_string());

        let request = TriggerWorkflowRequest {
            git_ref: "main".to_string(),
            inputs: Some(inputs),
        };

        let result = client
            .trigger_workflow("octocat", "Hello-World", 161335, request)
            .await;

        assert!(result.is_ok());
    }
}

mod workflow_run_operations {
    use super::*;

    /// Verify list_workflow_runs fetches workflow runs for a workflow.
    ///
    /// Tests GET /repos/:owner/:repo/actions/workflows/:id/runs endpoint.
    #[tokio::test]
    async fn test_list_workflow_runs() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let workflow_runs_json = serde_json::json!({
            "total_count": 1,
            "workflow_runs": [
                {
                    "id": 30433642,
                    "node_id": "MDEyOldvcmtmbG93IFJ1bjI2OTI4OQ==",
                    "name": "CI",
                    "run_number": 562,
                    "event": "push",
                    "status": "completed",
                    "conclusion": "success",
                    "workflow_id": 161335,
                    "head_branch": "main",
                    "head_sha": "acb5820ced9479c074f688cc328bf03f341a511d",
                    "created_at": "2020-01-22T19:33:08Z",
                    "updated_at": "2020-01-22T19:33:08Z",
                    "url": "https://api.github.com/repos/octocat/Hello-World/actions/runs/30433642",
                    "html_url": "https://github.com/octocat/Hello-World/actions/runs/30433642"
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path(
                "/repos/octocat/Hello-World/actions/workflows/161335/runs",
            ))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(workflow_runs_json))
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
            .list_workflow_runs("octocat", "Hello-World", 161335)
            .await;

        assert!(result.is_ok());
        let runs = result.unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].id, 30433642);
        assert_eq!(runs[0].name, "CI");
        assert_eq!(runs[0].status, "completed");
        assert_eq!(runs[0].conclusion, Some("success".to_string()));
    }

    /// Verify get_workflow_run fetches a specific workflow run by ID.
    ///
    /// Tests GET /repos/:owner/:repo/actions/runs/:id endpoint.
    #[tokio::test]
    async fn test_get_workflow_run() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let workflow_run_json = serde_json::json!({
            "id": 30433642,
            "node_id": "MDEyOldvcmtmbG93IFJ1bjI2OTI4OQ==",
            "name": "CI",
            "run_number": 562,
            "event": "push",
            "status": "completed",
            "conclusion": "success",
            "workflow_id": 161335,
            "head_branch": "main",
            "head_sha": "acb5820ced9479c074f688cc328bf03f341a511d",
            "created_at": "2020-01-22T19:33:08Z",
            "updated_at": "2020-01-22T19:33:08Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/actions/runs/30433642",
            "html_url": "https://github.com/octocat/Hello-World/actions/runs/30433642"
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/actions/runs/30433642"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(workflow_run_json))
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
            .get_workflow_run("octocat", "Hello-World", 30433642)
            .await;

        assert!(result.is_ok());
        let run = result.unwrap();
        assert_eq!(run.id, 30433642);
        assert_eq!(run.name, "CI");
        assert_eq!(run.run_number, 562);
        assert_eq!(run.event, "push");
        assert_eq!(run.status, "completed");
        assert_eq!(run.conclusion, Some("success".to_string()));
    }

    /// Verify cancel_workflow_run cancels a running workflow.
    ///
    /// Tests POST /repos/:owner/:repo/actions/runs/:id/cancel endpoint.
    #[tokio::test]
    async fn test_cancel_workflow_run() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path(
                "/repos/octocat/Hello-World/actions/runs/30433642/cancel",
            ))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(202))
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
            .cancel_workflow_run("octocat", "Hello-World", 30433642)
            .await;

        assert!(result.is_ok());
    }

    /// Verify rerun_workflow_run re-runs a workflow.
    ///
    /// Tests POST /repos/:owner/:repo/actions/runs/:id/rerun endpoint.
    #[tokio::test]
    async fn test_rerun_workflow_run() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path(
                "/repos/octocat/Hello-World/actions/runs/30433642/rerun",
            ))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(201))
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
            .rerun_workflow_run("octocat", "Hello-World", 30433642)
            .await;

        assert!(result.is_ok());
    }
}

mod serialization {
    use super::*;

    /// Verify Workflow can be deserialized from GitHub API response.
    ///
    /// Tests that Workflow struct correctly deserializes from JSON matching
    /// GitHub's workflow API response format.
    #[test]
    fn test_workflow_deserialize() {
        let json = r#"{
            "id": 161335,
            "node_id": "MDg6V29ya2Zsb3cxNjEzMzU=",
            "name": "CI",
            "path": ".github/workflows/ci.yml",
            "state": "active",
            "created_at": "2020-01-08T23:48:37.000Z",
            "updated_at": "2020-01-08T23:50:21.000Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/actions/workflows/161335",
            "html_url": "https://github.com/octocat/Hello-World/blob/master/.github/workflows/ci.yml",
            "badge_url": "https://github.com/octocat/Hello-World/workflows/CI/badge.svg"
        }"#;

        let workflow: Workflow = serde_json::from_str(json).unwrap();

        assert_eq!(workflow.id, 161335);
        assert_eq!(workflow.node_id, "MDg6V29ya2Zsb3cxNjEzMzU=");
        assert_eq!(workflow.name, "CI");
        assert_eq!(workflow.path, ".github/workflows/ci.yml");
        assert_eq!(workflow.state, "active");
        assert_eq!(
            workflow.url,
            "https://api.github.com/repos/octocat/Hello-World/actions/workflows/161335"
        );
        assert_eq!(
            workflow.html_url,
            "https://github.com/octocat/Hello-World/blob/master/.github/workflows/ci.yml"
        );
        assert_eq!(
            workflow.badge_url,
            "https://github.com/octocat/Hello-World/workflows/CI/badge.svg"
        );
    }

    /// Verify WorkflowRun can be deserialized from GitHub API response.
    ///
    /// Tests that WorkflowRun struct correctly deserializes from JSON matching
    /// GitHub's workflow run API response format.
    #[test]
    fn test_workflow_run_deserialize() {
        let json = r#"{
            "id": 30433642,
            "node_id": "MDEyOldvcmtmbG93IFJ1bjI2OTI4OQ==",
            "name": "Build",
            "run_number": 42,
            "event": "push",
            "status": "completed",
            "conclusion": "success",
            "workflow_id": 159038,
            "head_branch": "master",
            "head_sha": "acb5820ced9479c074f688cc328bf03f341a511d",
            "created_at": "2020-01-22T19:33:08Z",
            "updated_at": "2020-01-22T19:33:08Z",
            "url": "https://api.github.com/repos/octocat/Hello-World/actions/runs/30433642",
            "html_url": "https://github.com/octocat/Hello-World/actions/runs/30433642"
        }"#;

        let run: WorkflowRun = serde_json::from_str(json).unwrap();

        assert_eq!(run.id, 30433642);
        assert_eq!(run.node_id, "MDEyOldvcmtmbG93IFJ1bjI2OTI4OQ==");
        assert_eq!(run.name, "Build");
        assert_eq!(run.run_number, 42);
        assert_eq!(run.event, "push");
        assert_eq!(run.status, "completed");
        assert_eq!(run.conclusion, Some("success".to_string()));
        assert_eq!(run.workflow_id, 159038);
        assert_eq!(run.head_branch, "master");
        assert_eq!(run.head_sha, "acb5820ced9479c074f688cc328bf03f341a511d");
        assert_eq!(
            run.url,
            "https://api.github.com/repos/octocat/Hello-World/actions/runs/30433642"
        );
        assert_eq!(
            run.html_url,
            "https://github.com/octocat/Hello-World/actions/runs/30433642"
        );
    }

    /// Verify TriggerWorkflowRequest serializes correctly.
    ///
    /// Tests that TriggerWorkflowRequest serializes to JSON with ref field
    /// when inputs are not provided.
    #[test]
    fn test_trigger_workflow_request_serialize() {
        let request = TriggerWorkflowRequest {
            git_ref: "main".to_string(),
            inputs: None,
        };

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["ref"], "main");
        // Verify inputs field is not present when None
        assert!(json.get("inputs").is_none());
    }

    /// Verify TriggerWorkflowRequest serializes correctly with inputs.
    ///
    /// Tests that TriggerWorkflowRequest serializes to JSON including
    /// inputs when provided.
    #[test]
    fn test_trigger_workflow_request_serialize_with_inputs() {
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("name".to_string(), "Mona the Octocat".to_string());
        inputs.insert("home".to_string(), "San Francisco, CA".to_string());

        let request = TriggerWorkflowRequest {
            git_ref: "main".to_string(),
            inputs: Some(inputs),
        };

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["ref"], "main");
        assert!(json["inputs"].is_object());
        assert_eq!(json["inputs"]["name"], "Mona the Octocat");
        assert_eq!(json["inputs"]["home"], "San Francisco, CA");
    }
}

mod error_handling {
    use super::*;

    /// Verify get_workflow returns NotFound error for 404 response.
    ///
    /// Tests that attempting to fetch a non-existent workflow returns
    /// ApiError::NotFound.
    #[tokio::test]
    async fn test_workflow_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/actions/workflows/999999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found",
                "documentation_url": "https://docs.github.com/rest/actions/workflows#get-a-workflow"
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

        let result = client.get_workflow("octocat", "Hello-World", 999999).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    /// Verify workflow operations return AuthorizationFailed for 403 response.
    ///
    /// Tests that attempting operations without proper permissions returns
    /// ApiError::AuthorizationFailed.
    #[tokio::test]
    async fn test_forbidden_access() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path(
                "/repos/private-org/private-repo/actions/workflows/123/dispatches",
            ))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(serde_json::json!({
                    "message": "Resource not accessible by integration",
                    "documentation_url": "https://docs.github.com/rest/actions/workflows#create-a-workflow-dispatch-event"
                })),
            )
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

        let request = TriggerWorkflowRequest {
            git_ref: "main".to_string(),
            inputs: None,
        };

        let result = client
            .trigger_workflow("private-org", "private-repo", 123, request)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::AuthorizationFailed));
    }

    /// Verify trigger_workflow returns InvalidRequest when workflow is disabled.
    ///
    /// Tests that attempting to trigger a disabled workflow returns
    /// ApiError::InvalidRequest with appropriate error message.
    #[tokio::test]
    async fn test_workflow_disabled() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path(
                "/repos/octocat/Hello-World/actions/workflows/123/dispatches",
            ))
            .respond_with(
                ResponseTemplate::new(422).set_body_json(serde_json::json!({
                    "message": "Workflow is disabled",
                    "documentation_url": "https://docs.github.com/rest/actions/workflows#create-a-workflow-dispatch-event"
                })),
            )
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

        let request = TriggerWorkflowRequest {
            git_ref: "main".to_string(),
            inputs: None,
        };

        let result = client
            .trigger_workflow("octocat", "Hello-World", 123, request)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::InvalidRequest { message } => {
                assert!(message.contains("Workflow is disabled"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }
}
