//! Tests for project operations.

mod construction {
    use super::super::*;

    /// Verify AddProjectV2ItemRequest structure with content node ID.
    ///
    /// Tests that AddProjectV2ItemRequest can be constructed with a valid
    /// content node ID (issue or pull request node ID).
    #[test]
    fn test_add_project_item_request() {
        // Arrange: Create request with issue node ID
        let content_node_id = "MDU6SXNzdWUx".to_string();

        // Act: Create the request
        let request = AddProjectV2ItemRequest {
            content_node_id: content_node_id.clone(),
        };

        // Assert: Verify structure
        assert_eq!(request.content_node_id, content_node_id);

        // Verify serialization
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("content_node_id"));
        assert!(json.contains("MDU6SXNzdWUx"));
    }
}

mod project_operations {

    #[tokio::test]
    #[ignore = "TODO: Mock: GET /orgs/:org/projects"]
    async fn test_list_organization_projects() {
        todo!("Mock: GET /orgs/:org/projects")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: GET /users/:username/projects"]
    async fn test_list_user_projects() {
        todo!("Mock: GET /users/:username/projects")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: GET /orgs/:owner/projects/:number"]
    async fn test_get_project_organization() {
        todo!("Mock: GET /orgs/:owner/projects/:number")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: GET /users/:owner/projects/:number with fallback"]
    async fn test_get_project_user() {
        todo!("Mock: GET /users/:owner/projects/:number with fallback")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: 404 response"]
    async fn test_get_project_not_found() {
        todo!("Mock: 404 response")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: POST /projects/:id/items"]
    async fn test_add_item_to_project() {
        todo!("Mock: POST /projects/:id/items")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: 422 validation error"]
    async fn test_add_item_already_in_project() {
        todo!("Mock: 422 validation error")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: DELETE /projects/:id/items/:item_id"]
    async fn test_remove_item_from_project() {
        todo!("Mock: DELETE /projects/:id/items/:item_id")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: 404 response"]
    async fn test_remove_item_not_found() {
        todo!("Mock: 404 response")
    }
}

mod serialization {
    use super::super::*;

    /// Verify ProjectV2 can be deserialized from GitHub API response.
    ///
    /// Tests that ProjectV2 correctly deserializes from JSON matching
    /// GitHub's Projects v2 API response format.
    #[test]
    fn test_project_v2_deserialize() {
        // Arrange: GitHub Projects v2 API response JSON
        let json = r#"{
            "id": 12345,
            "node_id": "PVT_kwDOAE1L0M4AA1qJ",
            "number": 1,
            "title": "My Project",
            "description": "Project description",
            "owner": {
                "login": "octocat",
                "type": "Organization",
                "id": 1,
                "node_id": "MDEyOk9yZ2FuaXphdGlvbjE="
            },
            "public": true,
            "created_at": "2022-04-28T16:30:00Z",
            "updated_at": "2022-04-28T16:30:00Z",
            "url": "https://github.com/orgs/octocat/projects/1"
        }"#;

        // Act: Deserialize
        let project: ProjectV2 = serde_json::from_str(json).unwrap();

        // Assert: Verify all fields
        assert_eq!(project.id, 12345);
        assert_eq!(project.node_id, "PVT_kwDOAE1L0M4AA1qJ");
        assert_eq!(project.number, 1);
        assert_eq!(project.title, "My Project");
        assert_eq!(project.description, Some("Project description".to_string()));
        assert_eq!(project.owner.login, "octocat");
        assert_eq!(project.owner.owner_type, "Organization");
        assert_eq!(project.owner.id, 1);
        assert_eq!(project.owner.node_id, "MDEyOk9yZ2FuaXphdGlvbjE=");
        assert!(project.public);
        assert_eq!(project.url, "https://github.com/orgs/octocat/projects/1");
        assert!(project.created_at.to_rfc3339().starts_with("2022-04-28"));
        assert!(project.updated_at.to_rfc3339().starts_with("2022-04-28"));
    }

    #[test]
    #[ignore = "TODO: Verify ProjectOwner can be deserialized"]
    fn test_project_owner_deserialize() {
        todo!("Verify ProjectOwner can be deserialized")
    }

    #[test]
    #[ignore = "TODO: Verify ProjectV2Item can be deserialized"]
    fn test_project_v2_item_deserialize() {
        todo!("Verify ProjectV2Item can be deserialized")
    }

    #[test]
    #[ignore = "TODO: Verify AddProjectV2ItemRequest serializes correctly"]
    fn test_add_project_item_request_serialize() {
        todo!("Verify AddProjectV2ItemRequest serializes correctly")
    }
}

mod error_handling {

    #[tokio::test]
    #[ignore = "TODO: Mock: 404 response returns ApiError::NotFound"]
    async fn test_project_not_found() {
        todo!("Mock: 404 response returns ApiError::NotFound")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: 403 response returns ApiError::Forbidden"]
    async fn test_forbidden_access() {
        todo!("Mock: 403 response returns ApiError::Forbidden")
    }

    #[tokio::test]
    #[ignore = "TODO: Mock: 404 when org doesn't exist"]
    async fn test_organization_not_found() {
        todo!("Mock: 404 when org doesn't exist")
    }
}

// ============================================================================
// get_issue_linked_projects integration tests
// ============================================================================

mod get_issue_linked_projects_tests {
    use crate::auth::InstallationId;
    use crate::client::{ClientConfig, GitHubClient, InstallationClient};
    use crate::error::ApiError;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[path = "test_helpers.rs"]
    mod test_helpers;
    use test_helpers::MockAuthProvider;

    async fn make_client(mock_server: &MockServer, token: &str) -> InstallationClient {
        let auth = MockAuthProvider::new_with_token(token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server.uri()))
            .build()
            .unwrap();
        github_client
            .installation_by_id(InstallationId::new(12345))
            .await
            .unwrap()
    }

    /// Stub GraphQL response for an issue linked to one project.
    fn linked_projects_response() -> serde_json::Value {
        serde_json::json!({
            "data": {
                "repository": {
                    "issue": {
                        "projectsV2": {
                            "nodes": [
                                {
                                    "id": "PVT_kwDOAE1L0M4AA1qJ",
                                    "databaseId": 12345,
                                    "number": 1,
                                    "title": "Sprint Board",
                                    "description": "Sprint tracking project",
                                    "public": true,
                                    "url": "https://github.com/orgs/octocat/projects/1",
                                    "createdAt": "2022-04-28T16:30:00Z",
                                    "updatedAt": "2022-04-29T08:00:00Z",
                                    "owner": {
                                        "login": "octocat",
                                        "type": "Organization",
                                        "id": "MDEyOk9yZ2FuaXphdGlvbjE=",
                                        "databaseId": 1
                                    }
                                }
                            ]
                        }
                    }
                }
            }
        })
    }

    /// Stub GraphQL response for an issue with no linked projects.
    fn no_projects_response() -> serde_json::Value {
        serde_json::json!({
            "data": {
                "repository": {
                    "issue": {
                        "projectsV2": {
                            "nodes": []
                        }
                    }
                }
            }
        })
    }

    /// Stub GraphQL NOT_FOUND error response.
    fn not_found_response() -> serde_json::Value {
        serde_json::json!({
            "errors": [
                {
                    "type": "NOT_FOUND",
                    "message": "Could not resolve to a Repository with the name 'owner/missing-repo'."
                }
            ]
        })
    }

    /// Verify get_issue_linked_projects returns a populated Vec when the issue has projects.
    ///
    /// The GraphQL response includes one project node; the function must map all fields
    /// of ProjectV2 and ProjectOwner correctly and return a non-empty Vec.
    #[tokio::test]
    async fn test_returns_projects_when_issue_has_linked_projects() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(linked_projects_response()))
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .get_issue_linked_projects("octocat", "Hello-World", 42)
            .await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let projects = result.unwrap();
        assert_eq!(projects.len(), 1);
        let project = &projects[0];
        assert_eq!(project.id, 12345);
        assert_eq!(project.node_id, "PVT_kwDOAE1L0M4AA1qJ");
        assert_eq!(project.number, 1);
        assert_eq!(project.title, "Sprint Board");
        assert_eq!(
            project.description,
            Some("Sprint tracking project".to_string())
        );
        assert!(project.public);
        assert_eq!(project.url, "https://github.com/orgs/octocat/projects/1");
        assert_eq!(project.owner.login, "octocat");
        assert_eq!(project.owner.owner_type, "Organization");
        assert_eq!(project.owner.id, 1);
        assert_eq!(project.owner.node_id, "MDEyOk9yZ2FuaXphdGlvbjE=");
    }

    /// Verify get_issue_linked_projects returns an empty Vec when the issue has no projects.
    ///
    /// When `projectsV2.nodes` is an empty array, the function must return Ok(vec![])
    /// rather than an error — the issue exists, it just isn't linked to any project.
    #[tokio::test]
    async fn test_returns_empty_vec_when_issue_has_no_linked_projects() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(no_projects_response()))
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .get_issue_linked_projects("octocat", "Hello-World", 1)
            .await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        assert!(result.unwrap().is_empty());
    }

    /// Verify get_issue_linked_projects returns ApiError::NotFound for missing repo/issue.
    ///
    /// GitHub GraphQL returns `type: "NOT_FOUND"` in the errors array when the repository
    /// or issue does not exist. The function must surface this as ApiError::NotFound.
    #[tokio::test]
    async fn test_returns_not_found_for_missing_repository_or_issue() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(not_found_response()))
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .get_issue_linked_projects("owner", "missing-repo", 1)
            .await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ApiError::NotFound),
            "expected ApiError::NotFound"
        );
    }

    /// Verify get_issue_linked_projects returns ApiError::AuthenticationFailed on HTTP 401.
    ///
    /// Even though the GraphQL endpoint normally returns 200, an invalid token produces
    /// a 401 that must be propagated as ApiError::AuthenticationFailed.
    #[tokio::test]
    async fn test_returns_authentication_failed_on_401() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, "bad-token").await;
        let result = client.get_issue_linked_projects("owner", "repo", 1).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::AuthenticationFailed
        ));
    }
}
