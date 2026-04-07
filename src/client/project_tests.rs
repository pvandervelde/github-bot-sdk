//! Tests for project operations.

#[path = "test_helpers.rs"]
mod test_helpers;

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

    /// Verify ProjectOwner can be deserialized from JSON.
    ///
    /// Tests that ProjectOwner correctly deserializes all fields including the
    /// `type` rename (serde attribute maps "type" JSON key to `owner_type`).
    #[test]
    fn test_project_owner_deserialize() {
        let json = r#"{
            "login": "octocat",
            "type": "Organization",
            "id": 1,
            "node_id": "MDEyOk9yZ2FuaXphdGlvbjE="
        }"#;

        let owner: ProjectOwner = serde_json::from_str(json).unwrap();

        assert_eq!(owner.login, "octocat");
        assert_eq!(owner.owner_type, "Organization");
        assert_eq!(owner.id, 1);
        assert_eq!(owner.node_id, "MDEyOk9yZ2FuaXphdGlvbjE=");
    }

    /// Verify ProjectV2Item can be deserialized from JSON.
    ///
    /// Tests that ProjectV2Item correctly deserializes all six fields:
    /// id, node_id, content_type, content_node_id, created_at, updated_at.
    #[test]
    fn test_project_v2_item_deserialize() {
        let json = r#"{
            "id": "PVTI_lADOAE1L0M4AA1qJzgDeF2s",
            "node_id": "PVTI_lADOAE1L0M4AA1qJzgDeF2s",
            "content_type": "Issue",
            "content_node_id": "I_kwDOAE1L0M5abc123",
            "created_at": "2022-04-28T16:30:00Z",
            "updated_at": "2022-04-29T08:00:00Z"
        }"#;

        let item: ProjectV2Item = serde_json::from_str(json).unwrap();

        assert_eq!(item.id, "PVTI_lADOAE1L0M4AA1qJzgDeF2s");
        assert_eq!(item.node_id, "PVTI_lADOAE1L0M4AA1qJzgDeF2s");
        assert_eq!(item.content_type, "Issue");
        assert_eq!(item.content_node_id, "I_kwDOAE1L0M5abc123");
        assert!(item.created_at.to_rfc3339().starts_with("2022-04-28"));
        assert!(item.updated_at.to_rfc3339().starts_with("2022-04-29"));
    }

    /// Verify AddProjectV2ItemRequest serializes to the expected JSON shape.
    ///
    /// The field name must remain `content_node_id` in the serialized output
    /// (no rename attribute), matching the GraphQL variable name expected by the
    /// `addProjectV2ItemById` mutation.
    #[test]
    fn test_add_project_item_request_serialize() {
        let request = AddProjectV2ItemRequest {
            content_node_id: "I_kwDOAE1L0M5abc123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["content_node_id"], "I_kwDOAE1L0M5abc123");
        // Confirm exactly one field is serialized.
        assert_eq!(parsed.as_object().unwrap().len(), 1);
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
    use super::test_helpers::MockAuthProvider;
    use crate::auth::InstallationId;
    use crate::client::{ClientConfig, GitHubClient, InstallationClient};
    use crate::error::ApiError;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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

    /// Stub GraphQL response for an issue linked to one project (single page).
    fn linked_projects_response() -> serde_json::Value {
        serde_json::json!({
            "data": {
                "repository": {
                    "issue": {
                        "projectsV2": {
                            "pageInfo": { "hasNextPage": false, "endCursor": null },
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
                            "pageInfo": { "hasNextPage": false, "endCursor": null },
                            "nodes": []
                        }
                    }
                }
            }
        })
    }

    /// First page: one project with hasNextPage true.
    fn first_page_response() -> serde_json::Value {
        serde_json::json!({
            "data": {
                "repository": {
                    "issue": {
                        "projectsV2": {
                            "pageInfo": {
                                "hasNextPage": true,
                                "endCursor": "cursor_page2"
                            },
                            "nodes": [{
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
                            }]
                        }
                    }
                }
            }
        })
    }

    /// Second page: one project with hasNextPage false.
    fn second_page_response() -> serde_json::Value {
        serde_json::json!({
            "data": {
                "repository": {
                    "issue": {
                        "projectsV2": {
                            "pageInfo": {
                                "hasNextPage": false,
                                "endCursor": null
                            },
                            "nodes": [{
                                "id": "PVT_kwDOAE1L0M4AA2rK",
                                "databaseId": 67890,
                                "number": 2,
                                "title": "Backlog",
                                "description": null,
                                "public": false,
                                "url": "https://github.com/orgs/octocat/projects/2",
                                "createdAt": "2022-05-01T10:00:00Z",
                                "updatedAt": "2022-05-02T09:00:00Z",
                                "owner": {
                                    "login": "octocat",
                                    "type": "Organization",
                                    "id": "MDEyOk9yZ2FuaXphdGlvbjE=",
                                    "databaseId": 1
                                }
                            }]
                        }
                    }
                }
            }
        })
    }

    /// Stub GraphQL response where the issue field is null (issue number not found).
    fn null_issue_response() -> serde_json::Value {
        serde_json::json!({
            "data": {
                "repository": {
                    "issue": null
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
            .projects()
            .list_for_issue("octocat", "Hello-World", 42)
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
            .projects()
            .list_for_issue("octocat", "Hello-World", 1)
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
            .projects()
            .list_for_issue("owner", "missing-repo", 1)
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
        let result = client.projects().list_for_issue("owner", "repo", 1).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::AuthenticationFailed
        ));
    }

    /// Verify get_issue_linked_projects fetches all pages when results span multiple pages.
    ///
    /// When `pageInfo.hasNextPage` is true, the function must issue a follow-up query
    /// with the `endCursor`, continuing until `hasNextPage` is false. The combined
    /// results from all pages are returned in a single Vec.
    #[tokio::test]
    async fn test_returns_all_projects_across_multiple_pages() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";

        // First call → page 1, hasNextPage true
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(first_page_response()))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Second call → page 2, hasNextPage false
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(second_page_response()))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .projects()
            .list_for_issue("octocat", "Hello-World", 42)
            .await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let projects = result.unwrap();
        assert_eq!(projects.len(), 2, "expected 2 projects across 2 pages");
        assert_eq!(projects[0].id, 12345);
        assert_eq!(projects[0].title, "Sprint Board");
        assert_eq!(projects[1].id, 67890);
        assert_eq!(projects[1].title, "Backlog");
    }

    /// Verify get_issue_linked_projects returns ApiError::NotFound when the issue is null.
    ///
    /// GitHub GraphQL returns `"issue": null` (not an error in `.errors[]`) when the issue
    /// number does not exist in a repository that does exist. The function must surface
    /// this as ApiError::NotFound, not Ok(vec![]).
    #[tokio::test]
    async fn test_returns_not_found_when_issue_is_null() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(null_issue_response()))
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .projects()
            .list_for_issue("octocat", "Hello-World", 9999)
            .await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ApiError::NotFound),
            "expected ApiError::NotFound when issue field is null"
        );
    }
}

// ============================================================================
// add_item_to_project integration tests
// ============================================================================

mod add_item_to_project_tests {
    use super::test_helpers::MockAuthProvider;
    use crate::auth::InstallationId;
    use crate::client::{ClientConfig, GitHubClient, InstallationClient};
    use crate::error::ApiError;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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

    // -----------------------------------------------------------------------
    // Mock response builders
    // -----------------------------------------------------------------------

    /// org-level project node ID lookup — success.
    fn org_lookup_response(node_id: &str) -> serde_json::Value {
        serde_json::json!({
            "data": {
                "organization": {
                    "projectV2": { "id": node_id }
                }
            }
        })
    }

    /// org-level lookup returns NOT_FOUND → must fall back to user lookup.
    fn org_not_found_response() -> serde_json::Value {
        serde_json::json!({
            "errors": [{ "type": "NOT_FOUND", "message": "org not found" }]
        })
    }

    /// user-level project node ID lookup — success.
    fn user_lookup_response(node_id: &str) -> serde_json::Value {
        serde_json::json!({
            "data": {
                "user": {
                    "projectV2": { "id": node_id }
                }
            }
        })
    }

    /// Both org and user lookups return NOT_FOUND.
    fn user_not_found_response() -> serde_json::Value {
        serde_json::json!({
            "errors": [{ "type": "NOT_FOUND", "message": "user project not found" }]
        })
    }

    /// addProjectV2ItemById mutation — success.
    fn add_item_success_response() -> serde_json::Value {
        serde_json::json!({
            "data": {
                "addProjectV2ItemById": {
                    "item": {
                        "id": "PVTI_lADOAE1L0M4AA1qJzgDeF2s",
                        "type": "ISSUE",
                        "createdAt": "2022-04-28T16:30:00Z",
                        "updatedAt": "2022-04-28T16:31:00Z",
                        "content": {
                            "__typename": "Issue",
                            "id": "I_kwDOAE1L0M5abc123"
                        }
                    }
                }
            }
        })
    }

    /// addProjectV2ItemById mutation — item already exists in project.
    fn add_item_already_exists_response() -> serde_json::Value {
        serde_json::json!({
            "errors": [{
                "message": "Item already added to project",
                "type": "UNPROCESSABLE"
            }]
        })
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// Verify add_item_to_project returns a populated ProjectV2Item on success.
    ///
    /// Two sequential GraphQL calls are mocked: org node-ID lookup then the
    /// addProjectV2ItemById mutation. The returned item must have all fields
    /// populated from the mutation response.
    #[tokio::test]
    async fn test_successful_add_returns_populated_item() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";
        let project_node_id = "PVT_kwDOAE1L0M4AA1qJ";

        // 1st call → org node ID resolution
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(org_lookup_response(project_node_id)),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // 2nd call → addProjectV2ItemById mutation
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(add_item_success_response()))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .projects()
            .add_item("octocat", 1, "I_kwDOAE1L0M5abc123")
            .await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let item = result.unwrap();
        assert_eq!(item.id, "PVTI_lADOAE1L0M4AA1qJzgDeF2s");
        assert_eq!(item.node_id, "PVTI_lADOAE1L0M4AA1qJzgDeF2s");
        assert_eq!(item.content_type, "ISSUE");
        assert_eq!(item.content_node_id, "I_kwDOAE1L0M5abc123");
        assert!(item.created_at.to_rfc3339().starts_with("2022-04-28"));
    }

    /// Verify add_item_to_project falls back to user lookup when org NOT_FOUND.
    ///
    /// When the org-level `projectV2` query returns NOT_FOUND, a second call
    /// using the user-level query must be attempted and the item added.
    #[tokio::test]
    async fn test_falls_back_to_user_when_org_not_found() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";
        let project_node_id = "PVT_kwDOAE1L0M4AA1qJ";

        // 1st call → org lookup → NOT_FOUND
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(org_not_found_response()))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // 2nd call → user lookup → success
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(user_lookup_response(project_node_id)),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // 3rd call → addProjectV2ItemById mutation
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(add_item_success_response()))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .projects()
            .add_item("octocat", 1, "I_kwDOAE1L0M5abc123")
            .await;

        assert!(
            result.is_ok(),
            "expected Ok after user fallback, got {:?}",
            result
        );
        assert_eq!(result.unwrap().id, "PVTI_lADOAE1L0M4AA1qJzgDeF2s");
    }

    /// Verify add_item_to_project returns ApiError::NotFound when project is not found.
    ///
    /// Both the org and user lookups return NOT_FOUND; the function must surface
    /// ApiError::NotFound to the caller rather than panicking or returning a generic error.
    #[tokio::test]
    async fn test_returns_not_found_when_project_does_not_exist() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";

        // 1st call → org lookup → NOT_FOUND
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(org_not_found_response()))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // 2nd call → user lookup → NOT_FOUND
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(user_not_found_response()))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .projects()
            .add_item("unknown-owner", 99, "I_kwDOAE1L0M5abc123")
            .await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ApiError::NotFound),
            "expected ApiError::NotFound"
        );
    }

    /// Verify add_item_to_project returns ApiError::GraphQlError when content is already
    /// in the project.
    ///
    /// The `addProjectV2ItemById` mutation returns an UNPROCESSABLE error when the
    /// content is already present. This must be surfaced as ApiError::GraphQlError with
    /// the original message so callers can handle duplicate additions.
    #[tokio::test]
    async fn test_returns_graphql_error_when_item_already_in_project() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";
        let project_node_id = "PVT_kwDOAE1L0M4AA1qJ";

        // 1st call → org node ID resolution
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(org_lookup_response(project_node_id)),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // 2nd call → mutation returns UNPROCESSABLE
        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(add_item_already_exists_response()),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .projects()
            .add_item("octocat", 1, "I_kwDOAE1L0M5abc123")
            .await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ApiError::GraphQlError { .. }),
            "expected ApiError::GraphQlError for duplicate item"
        );
    }

    /// Verify add_item_to_project returns ApiError::AuthenticationFailed on HTTP 401.
    #[tokio::test]
    async fn test_returns_authentication_failed_on_401() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, "bad-token").await;
        let result = client
            .projects()
            .add_item("octocat", 1, "I_kwDOAE1L0M5abc123")
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApiError::AuthenticationFailed
        ));
    }

    /// Verify add_item_to_project returns ApiError::GraphQlError when the mutation
    /// response is missing the required `type` field on the returned item.
    ///
    /// `type` is always selected by the mutation; its absence indicates an unexpected
    /// API shape change and must be surfaced as an error rather than silently defaulting.
    #[tokio::test]
    async fn test_returns_error_when_item_type_missing() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";
        let project_node_id = "PVT_kwDOAE1L0M4AA1qJ";

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(org_lookup_response(project_node_id)),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "addProjectV2ItemById": {
                        "item": {
                            "id": "PVTI_lADOAE1L0M4AA1qJzgDeF2s",
                            "createdAt": "2022-04-28T16:30:00Z",
                            "updatedAt": "2022-04-28T16:31:00Z",
                            "content": { "id": "I_kwDOAE1L0M5abc123" }
                        }
                    }
                }
            })))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .projects()
            .add_item("octocat", 1, "I_kwDOAE1L0M5abc123")
            .await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ApiError::GraphQlError { .. }),
            "expected GraphQlError for missing item type field"
        );
    }

    /// Verify add_item_to_project returns ApiError::GraphQlError when the mutation
    /// response is missing the `createdAt` timestamp on the returned item.
    ///
    /// Timestamps are always selected by the mutation; their absence indicates an
    /// unexpected API shape change and must be surfaced as an error.
    #[tokio::test]
    async fn test_returns_error_when_item_timestamps_missing() {
        let mock_server = MockServer::start().await;
        let token = "ghs_test_token";
        let project_node_id = "PVT_kwDOAE1L0M4AA1qJ";

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(org_lookup_response(project_node_id)),
            )
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "addProjectV2ItemById": {
                        "item": {
                            "id": "PVTI_lADOAE1L0M4AA1qJzgDeF2s",
                            "type": "ISSUE",
                            "content": { "id": "I_kwDOAE1L0M5abc123" }
                        }
                    }
                }
            })))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let client = make_client(&mock_server, token).await;
        let result = client
            .projects()
            .add_item("octocat", 1, "I_kwDOAE1L0M5abc123")
            .await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ApiError::GraphQlError { .. }),
            "expected GraphQlError for missing item timestamps"
        );
    }
}
