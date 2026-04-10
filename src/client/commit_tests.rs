//! Tests for commit operations.

use super::*;
use crate::auth::InstallationId;
use crate::client::{ClientConfig, GitHubClient};
use crate::error::ApiError;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[path = "test_helpers.rs"]
mod test_helpers;
use test_helpers::MockAuthProvider;

// ============================================================================
// Shared test fixtures
// ============================================================================

/// Returns a minimal valid IssueUser JSON object for embedding in commit JSON.
fn octocat_user_json() -> serde_json::Value {
    serde_json::json!({
        "login": "octocat",
        "id": 1,
        "node_id": "MDQ6VXNlcjE=",
        "type": "User"
    })
}

/// Builds a complete FullCommit JSON for a single-parent commit.
fn full_commit_json(sha: &str) -> serde_json::Value {
    serde_json::json!({
        "sha": sha,
        "node_id": "MDY6Q29tbWl0MQ==",
        "commit": {
            "author": {
                "name": "Monalisa Octocat",
                "email": "octocat@github.com",
                "date": "2026-01-15T12:00:00Z"
            },
            "committer": {
                "name": "GitHub",
                "email": "noreply@github.com",
                "date": "2026-01-15T12:00:00Z"
            },
            "message": "feat: add new feature\n\nLonger description here.",
            "tree": {
                "sha": "abc123tree",
                "url": "https://api.github.com/repos/octocat/Hello-World/git/trees/abc123tree"
            },
            "comment_count": 0,
            "verification": {
                "verified": false,
                "reason": "unsigned",
                "signature": null,
                "payload": null
            }
        },
        "author": octocat_user_json(),
        "committer": octocat_user_json(),
        "parents": [
            {
                "sha": "parentsha1",
                "url": "https://api.github.com/repos/octocat/Hello-World/commits/parentsha1"
            }
        ],
        "url": format!("https://api.github.com/repos/octocat/Hello-World/commits/{}", sha),
        "html_url": format!("https://github.com/octocat/Hello-World/commit/{}", sha),
        "comment_count": 0
    })
}

// ============================================================================
// Deserialization unit tests
// ============================================================================

mod deserialization {
    use super::*;

    /// Deserialize a FullCommit with all optional fields (author, committer,
    /// verification) present.
    #[test]
    fn test_deserialize_full_commit_with_all_fields() {
        let json = full_commit_json("abc123sha456");

        let commit: FullCommit = serde_json::from_value(json).unwrap();

        assert_eq!(commit.sha, "abc123sha456");
        assert_eq!(commit.node_id, "MDY6Q29tbWl0MQ==");
        assert_eq!(
            commit.commit.message,
            "feat: add new feature\n\nLonger description here."
        );
        assert_eq!(commit.commit.author.name, "Monalisa Octocat");
        assert_eq!(commit.commit.author.email, "octocat@github.com");
        assert_eq!(commit.commit.committer.name, "GitHub");
        assert_eq!(commit.commit.tree.sha, "abc123tree");
        assert_eq!(commit.commit.comment_count, 0);
        assert!(commit.author.is_some());
        assert_eq!(commit.author.as_ref().unwrap().login, "octocat");
        assert!(commit.committer.is_some());
        assert_eq!(commit.parents.len(), 1);
        assert_eq!(commit.parents[0].sha, "parentsha1");
        assert_eq!(commit.comment_count, 0);

        // Verification present but unsigned
        let v = commit.commit.verification.as_ref().unwrap();
        assert!(!v.verified);
        assert_eq!(v.reason, "unsigned");
        assert!(v.signature.is_none());
        assert!(v.payload.is_none());
    }

    /// Deserialize a FullCommit where author and committer GitHub users are
    /// absent (no matching GitHub account for those emails).
    #[test]
    fn test_deserialize_full_commit_optional_fields_absent() {
        let json = serde_json::json!({
            "sha": "nousers123",
            "node_id": "node1",
            "commit": {
                "author": {
                    "name": "Git Author",
                    "email": "author@example.com",
                    "date": "2026-02-01T08:00:00Z"
                },
                "committer": {
                    "name": "Git Committer",
                    "email": "committer@example.com",
                    "date": "2026-02-01T08:00:00Z"
                },
                "message": "chore: miscellaneous",
                "tree": {
                    "sha": "treesha",
                    "url": "https://api.github.com/repos/o/r/git/trees/treesha"
                },
                "comment_count": 2
            },
            "author": null,
            "committer": null,
            "parents": [
                { "sha": "parentA", "url": "https://api.github.com/repos/o/r/commits/parentA" }
            ],
            "url": "https://api.github.com/repos/o/r/commits/nousers123",
            "html_url": "https://github.com/o/r/commit/nousers123",
            "comment_count": 2
        });

        let commit: FullCommit = serde_json::from_value(json).unwrap();

        assert_eq!(commit.sha, "nousers123");
        assert!(commit.author.is_none());
        assert!(commit.committer.is_none());
        assert!(commit.commit.verification.is_none());
        assert_eq!(commit.commit.comment_count, 2);
        assert_eq!(commit.comment_count, 2);
    }

    /// Deserialize a merge commit that has two parent entries.
    #[test]
    fn test_deserialize_merge_commit() {
        let json = serde_json::json!({
            "sha": "mergesha",
            "node_id": "nodemerge",
            "commit": {
                "author": { "name": "Alice", "email": "alice@example.com", "date": "2026-02-10T10:00:00Z" },
                "committer": { "name": "Alice", "email": "alice@example.com", "date": "2026-02-10T10:00:00Z" },
                "message": "Merge pull request #42",
                "tree": { "sha": "treesha", "url": "https://api.github.com/repos/o/r/git/trees/treesha" },
                "comment_count": 0
            },
            "author": octocat_user_json(),
            "committer": octocat_user_json(),
            "parents": [
                { "sha": "parent1", "url": "https://api.github.com/repos/o/r/commits/parent1" },
                { "sha": "parent2", "url": "https://api.github.com/repos/o/r/commits/parent2" }
            ],
            "url": "https://api.github.com/repos/o/r/commits/mergesha",
            "html_url": "https://github.com/o/r/commit/mergesha",
            "comment_count": 0
        });

        let commit: FullCommit = serde_json::from_value(json).unwrap();

        assert_eq!(commit.sha, "mergesha");
        assert_eq!(commit.parents.len(), 2);
        assert_eq!(commit.parents[0].sha, "parent1");
        assert_eq!(commit.parents[1].sha, "parent2");
    }

    /// Deserialize the initial commit of a repository (no parents).
    #[test]
    fn test_deserialize_initial_commit() {
        let json = serde_json::json!({
            "sha": "initialsha",
            "node_id": "nodeinitial",
            "commit": {
                "author": { "name": "Bob", "email": "bob@example.com", "date": "2026-01-01T00:00:00Z" },
                "committer": { "name": "Bob", "email": "bob@example.com", "date": "2026-01-01T00:00:00Z" },
                "message": "Initial commit",
                "tree": { "sha": "treetop", "url": "https://api.github.com/repos/o/r/git/trees/treetop" },
                "comment_count": 0
            },
            "author": null,
            "committer": null,
            "parents": [],
            "url": "https://api.github.com/repos/o/r/commits/initialsha",
            "html_url": "https://github.com/o/r/commit/initialsha",
            "comment_count": 0
        });

        let commit: FullCommit = serde_json::from_value(json).unwrap();

        assert_eq!(commit.sha, "initialsha");
        assert!(commit.parents.is_empty());
    }

    /// Deserialize a GPG-verified commit (verification.verified = true).
    #[test]
    fn test_deserialize_verified_commit() {
        let json = serde_json::json!({
            "sha": "signedsha",
            "node_id": "nodesigned",
            "commit": {
                "author": { "name": "Charlie", "email": "charlie@example.com", "date": "2026-03-01T09:00:00Z" },
                "committer": { "name": "Charlie", "email": "charlie@example.com", "date": "2026-03-01T09:00:00Z" },
                "message": "fix: security patch",
                "tree": { "sha": "treesign", "url": "https://api.github.com/repos/o/r/git/trees/treesign" },
                "comment_count": 0,
                "verification": {
                    "verified": true,
                    "reason": "valid",
                    "signature": "-----BEGIN PGP SIGNATURE-----\nData\n-----END PGP SIGNATURE-----",
                    "payload": "tree treesign\nparent parentsha\nauthor Charlie..."
                }
            },
            "author": null,
            "committer": null,
            "parents": [{ "sha": "parentsha", "url": "https://api.github.com/repos/o/r/commits/parentsha" }],
            "url": "https://api.github.com/repos/o/r/commits/signedsha",
            "html_url": "https://github.com/o/r/commit/signedsha",
            "comment_count": 0
        });

        let commit: FullCommit = serde_json::from_value(json).unwrap();

        let v = commit.commit.verification.as_ref().unwrap();
        assert!(v.verified);
        assert_eq!(v.reason, "valid");
        assert!(v.signature.is_some());
        assert!(v.payload.is_some());
    }

    /// Deserialize a Comparison response with all fields present.
    #[test]
    fn test_deserialize_comparison_all_fields() {
        let commit_a = full_commit_json("sha_base");
        let commit_b = full_commit_json("sha_head");
        let commit_c = full_commit_json("sha_merge");

        let json = serde_json::json!({
            "base_commit": commit_a,
            "merge_base_commit": commit_c,
            "head_commit": commit_b,
            "status": "ahead",
            "ahead_by": 3,
            "behind_by": 0,
            "total_commits": 3,
            "commits": [commit_a, commit_b],
            "files": [
                {
                    "filename": "src/lib.rs",
                    "status": "modified",
                    "additions": 10,
                    "deletions": 2,
                    "changes": 12,
                    "blob_url": "https://github.com/o/r/blob/sha_head/src/lib.rs",
                    "raw_url": "https://github.com/o/r/raw/sha_head/src/lib.rs",
                    "contents_url": "https://api.github.com/repos/o/r/contents/src/lib.rs",
                    "patch": "@@ -1,2 +1,10 @@\n+new content"
                }
            ],
            "html_url": "https://github.com/o/r/compare/sha_base...sha_head",
            "permalink_url": "https://github.com/o/r/compare/sha_base...sha_head",
            "diff_url": "https://github.com/o/r/compare/sha_base...sha_head.diff",
            "patch_url": "https://github.com/o/r/compare/sha_base...sha_head.patch",
            "url": "https://api.github.com/repos/o/r/compare/sha_base...sha_head"
        });

        let cmp: Comparison = serde_json::from_value(json).unwrap();

        assert_eq!(cmp.base_commit.sha, "sha_base");
        assert_eq!(cmp.head_commit.sha, "sha_head");
        assert_eq!(cmp.merge_base_commit.sha, "sha_merge");
        assert_eq!(cmp.status, "ahead");
        assert_eq!(cmp.ahead_by, 3);
        assert_eq!(cmp.behind_by, 0);
        assert_eq!(cmp.total_commits, 3);
        assert_eq!(cmp.commits.len(), 2);
        assert_eq!(cmp.files.len(), 1);
        assert_eq!(cmp.files[0].filename, "src/lib.rs");
        assert_eq!(cmp.files[0].additions, 10);
        assert!(cmp.files[0].patch.is_some());
        assert_eq!(
            cmp.url,
            "https://api.github.com/repos/o/r/compare/sha_base...sha_head"
        );
    }

    /// Deserialize a FileChange for a renamed file (previous_filename present).
    #[test]
    fn test_deserialize_file_change_renamed() {
        let json = serde_json::json!({
            "filename": "new_name.rs",
            "status": "renamed",
            "additions": 0,
            "deletions": 0,
            "changes": 0,
            "blob_url": "https://github.com/o/r/blob/sha/new_name.rs",
            "raw_url": "https://github.com/o/r/raw/sha/new_name.rs",
            "contents_url": "https://api.github.com/repos/o/r/contents/new_name.rs",
            "previous_filename": "old_name.rs"
        });

        let file: FileChange = serde_json::from_value(json).unwrap();

        assert_eq!(file.filename, "new_name.rs");
        assert_eq!(file.status, "renamed");
        assert_eq!(file.previous_filename.as_deref(), Some("old_name.rs"));
        assert!(file.patch.is_none());
    }

    /// Deserialize a FileChange for a binary file where the patch is absent.
    #[test]
    fn test_deserialize_file_change_no_patch() {
        let json = serde_json::json!({
            "filename": "image.png",
            "status": "added",
            "additions": 0,
            "deletions": 0,
            "changes": 0,
            "blob_url": "https://github.com/o/r/blob/sha/image.png",
            "raw_url": "https://github.com/o/r/raw/sha/image.png",
            "contents_url": "https://api.github.com/repos/o/r/contents/image.png"
        });

        let file: FileChange = serde_json::from_value(json).unwrap();

        assert_eq!(file.filename, "image.png");
        assert_eq!(file.status, "added");
        assert!(file.patch.is_none());
        assert!(file.previous_filename.is_none());
    }

    /// Verify FileChange serialization omits optional fields when None.
    #[test]
    fn test_serialize_file_change_omits_optional_none_fields() {
        let file = FileChange {
            filename: "README.md".to_string(),
            status: "modified".to_string(),
            additions: 5,
            deletions: 1,
            changes: 6,
            blob_url: "https://github.com/o/r/blob/sha/README.md".to_string(),
            raw_url: "https://github.com/o/r/raw/sha/README.md".to_string(),
            contents_url: "https://api.github.com/repos/o/r/contents/README.md".to_string(),
            patch: None,
            previous_filename: None,
        };

        let json = serde_json::to_value(&file).unwrap();
        assert!(
            json.get("patch").is_none(),
            "patch should be absent when None"
        );
        assert!(
            json.get("previous_filename").is_none(),
            "previous_filename should be absent when None"
        );
    }
}

// ============================================================================
// Integration tests (mock server)
// ============================================================================

mod commit_operations {
    use super::*;
    use wiremock::matchers::path_regex;

    // -------------------------------------------------------------------------
    // get_commit
    // -------------------------------------------------------------------------

    /// get_commit by SHA returns the deserialized FullCommit.
    #[tokio::test]
    async fn test_get_commit_by_sha() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";
        let sha = "abc123def456abc123def456abc123def456abc1";

        Mock::given(method("GET"))
            .and(path(format!("/repos/octocat/Hello-World/commits/{}", sha)))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .and(header("Accept", "application/vnd.github+json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(full_commit_json(sha)))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client.get_commit("octocat", "Hello-World", sha).await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let commit = result.unwrap();
        assert_eq!(commit.sha, sha);
        assert_eq!(commit.commit.author.name, "Monalisa Octocat");
    }

    /// get_commit by branch name returns the deserialized FullCommit.
    #[tokio::test]
    async fn test_get_commit_by_branch() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits/main"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(200).set_body_json(full_commit_json("headsha")))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client.get_commit("octocat", "Hello-World", "main").await;

        assert!(result.is_ok());
        let commit = result.unwrap();
        assert_eq!(commit.sha, "headsha");
    }

    /// get_commit by tag name returns the deserialized FullCommit.
    #[tokio::test]
    async fn test_get_commit_by_tag() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits/v1.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(full_commit_json("tagsha")))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client.get_commit("octocat", "Hello-World", "v1.0.0").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().sha, "tagsha");
    }

    /// get_commit URL-encodes the ref_name so branch names with slashes are
    /// transmitted as a single path segment.
    #[tokio::test]
    async fn test_get_commit_url_encodes_ref_name() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        // refs/heads/feature/x → refs%2Fheads%2Ffeature%2Fx
        Mock::given(method("GET"))
            .and(path(
                "/repos/octocat/Hello-World/commits/refs%2Fheads%2Ffeature%2Fx",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(full_commit_json("encsha")))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .get_commit("octocat", "Hello-World", "refs/heads/feature/x")
            .await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        assert_eq!(result.unwrap().sha, "encsha");
    }

    /// get_commit returns ApiError::InvalidRequest for a 422 response.
    #[tokio::test]
    async fn test_get_commit_422_returns_invalid_request() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path_regex(r"/repos/octocat/Hello-World/commits/.*"))
            .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
                "message": "No commit found for SHA: invalidref"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .get_commit("octocat", "Hello-World", "invalidref")
            .await;

        assert!(
            matches!(result, Err(ApiError::InvalidRequest { .. })),
            "Expected InvalidRequest, got: {:?}",
            result
        );
    }
    #[tokio::test]
    async fn test_get_commit_404_returns_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path_regex(r"/repos/octocat/Hello-World/commits/.*"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .get_commit("octocat", "Hello-World", "nonexistent")
            .await;

        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    /// get_commit returns ApiError::AuthorizationFailed for a 403 response.
    #[tokio::test]
    async fn test_get_commit_403_returns_authorization_failed() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path_regex(r"/repos/octocat/Hello-World/commits/.*"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "message": "Forbidden"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client.get_commit("octocat", "Hello-World", "sha1").await;

        assert!(matches!(result, Err(ApiError::AuthorizationFailed)));
    }

    /// get_commit returns ApiError::AuthenticationFailed for a 401 response.
    #[tokio::test]
    async fn test_get_commit_401_returns_authentication_failed() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path_regex(r"/repos/octocat/Hello-World/commits/.*"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "message": "Requires authentication"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client.get_commit("octocat", "Hello-World", "sha1").await;

        assert!(matches!(result, Err(ApiError::AuthenticationFailed)));
    }

    /// get_commit returns ApiError::HttpError for unexpected server errors (5xx).
    #[tokio::test]
    async fn test_get_commit_server_error() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path_regex(r"/repos/octocat/Hello-World/commits/.*"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client.get_commit("octocat", "Hello-World", "sha1").await;

        match result.unwrap_err() {
            ApiError::HttpError { status, .. } => assert_eq!(status, 503),
            e => panic!("Expected HttpError, got: {:?}", e),
        }
    }

    // -------------------------------------------------------------------------
    // list_commits
    // -------------------------------------------------------------------------

    /// list_commits with no filters calls the correct endpoint and returns
    /// commits in the response order.
    #[tokio::test]
    async fn test_list_commits_default_branch() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let commits_json = serde_json::json!([
            full_commit_json("sha3"),
            full_commit_json("sha2"),
            full_commit_json("sha1")
        ]);

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(200).set_body_json(commits_json))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let commits = result.unwrap();
        assert_eq!(commits.len(), 3);
        // Response order is preserved (newest first as returned by GitHub)
        assert_eq!(commits[0].sha, "sha3");
        assert_eq!(commits[2].sha, "sha1");
    }

    /// list_commits with a path filter passes the `path` query parameter.
    #[tokio::test]
    async fn test_list_commits_path_filter() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .and(query_param("path", "README.md"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([full_commit_json("readmesha")])),
            )
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                Some("README.md"),
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    /// list_commits with an author filter passes the `author` query parameter.
    #[tokio::test]
    async fn test_list_commits_author_filter() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .and(query_param("author", "alice"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([full_commit_json("alicesha")])),
            )
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                Some("alice"),
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0].sha, "alicesha");
    }

    /// list_commits with since/until passes ISO 8601 date query parameters.
    #[tokio::test]
    async fn test_list_commits_date_range() {
        use chrono::TimeZone;
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let since = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let until = chrono::Utc
            .with_ymd_and_hms(2026, 1, 31, 23, 59, 59)
            .unwrap();

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .and(query_param("since", since.to_rfc3339()))
            .and(query_param("until", until.to_rfc3339()))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([full_commit_json("rangesha")])),
            )
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                None,
                Some(since),
                Some(until),
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0].sha, "rangesha");
    }

    /// list_commits clamps per_page to 100 when a larger value is given.
    #[tokio::test]
    async fn test_list_commits_per_page_clamped_to_100() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        // The mock only matches when per_page is exactly "100"
        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .and(query_param("per_page", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        // Request 200, expect clamped to 100
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                None,
                None,
                None,
                Some(200),
                None,
            )
            .await;

        assert!(
            result.is_ok(),
            "Expected Ok after clamping, got: {:?}",
            result
        );
    }

    /// list_commits with per_page <= 100 passes the value unchanged.
    #[tokio::test]
    async fn test_list_commits_per_page_not_clamped() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .and(query_param("per_page", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                None,
                None,
                None,
                Some(50),
                None,
            )
            .await;

        assert!(result.is_ok());
    }

    /// list_commits on an empty repository returns ApiError::InvalidRequest (422).
    ///
    /// Covers assertion 36.
    #[tokio::test]
    async fn test_list_commits_empty_repo_returns_invalid_request() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/empty-repo/commits"))
            .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
                "message": "Git Repository is empty."
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "empty-repo",
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(
            matches!(result, Err(ApiError::InvalidRequest { .. })),
            "Expected InvalidRequest, got: {:?}",
            result
        );
    }

    /// list_commits returns ApiError::NotFound for a 404 response.
    #[tokio::test]
    async fn test_list_commits_404_returns_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/missing-repo/commits"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "missing-repo",
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    /// list_commits returns ApiError::AuthorizationFailed for a 403 response.
    #[tokio::test]
    async fn test_list_commits_403_returns_authorization_failed() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "message": "Forbidden"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(matches!(result, Err(ApiError::AuthorizationFailed)));
    }

    /// list_commits returns ApiError::AuthenticationFailed for a 401 response.
    #[tokio::test]
    async fn test_list_commits_401_returns_authentication_failed() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "message": "Requires authentication"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(matches!(result, Err(ApiError::AuthenticationFailed)));
    }

    /// list_commits returns ApiError::HttpError for unexpected server errors (5xx).
    #[tokio::test]
    async fn test_list_commits_server_error() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        match result.unwrap_err() {
            ApiError::HttpError { status, .. } => assert_eq!(status, 503),
            e => panic!("Expected HttpError, got: {:?}", e),
        }
    }

    /// list_commits with per_page=0 clamps the value to 1, not 0.
    #[tokio::test]
    async fn test_list_commits_per_page_zero_clamped_to_one() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        // The mock only matches when per_page is exactly "1" (clamped from 0)
        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .and(query_param("per_page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                None,
                None,
                None,
                Some(0),
                None,
            )
            .await;

        assert!(
            result.is_ok(),
            "Expected Ok after clamping 0 to 1, got: {:?}",
            result
        );
    }

    /// list_commits with a sha filter passes the `sha` query parameter.
    #[tokio::test]
    async fn test_list_commits_sha_filter() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .and(query_param("sha", "feature-branch"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([full_commit_json("branchsha")])),
            )
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                Some("feature-branch"),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0].sha, "branchsha");
    }

    /// list_commits with a page number sends the correct page query parameter.
    #[tokio::test]
    async fn test_list_commits_page_parameter() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/commits"))
            .and(query_param("page", "3"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([full_commit_json("pagesha")])),
            )
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .list_commits(
                "octocat",
                "Hello-World",
                None,
                None,
                None,
                None,
                None,
                None,
                Some(3),
            )
            .await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        assert_eq!(result.unwrap()[0].sha, "pagesha");
    }

    // -------------------------------------------------------------------------
    // compare_commits
    // -------------------------------------------------------------------------

    /// compare_commits returns the deserialized Comparison.
    ///
    /// Covers assertion 34.
    #[tokio::test]
    async fn test_compare_commits_ahead() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let base = full_commit_json("base_sha");
        let head = full_commit_json("head_sha");
        let merge_base = full_commit_json("merge_base_sha");

        let comparison_json = serde_json::json!({
            "base_commit": base,
            "merge_base_commit": merge_base,
            "head_commit": head,
            "status": "ahead",
            "ahead_by": 5,
            "behind_by": 0,
            "total_commits": 5,
            "commits": [
                full_commit_json("c1"),
                full_commit_json("c2"),
                full_commit_json("c3"),
                full_commit_json("c4"),
                full_commit_json("c5")
            ],
            "files": [
                {
                    "filename": "src/main.rs",
                    "status": "modified",
                    "additions": 20,
                    "deletions": 5,
                    "changes": 25,
                    "blob_url": "https://github.com/o/r/blob/head_sha/src/main.rs",
                    "raw_url": "https://github.com/o/r/raw/head_sha/src/main.rs",
                    "contents_url": "https://api.github.com/repos/o/r/contents/src/main.rs"
                }
            ],
            "html_url": "https://github.com/octocat/Hello-World/compare/v1.0.0...v1.1.0",
            "permalink_url": "https://github.com/octocat/Hello-World/compare/v1.0.0...v1.1.0",
            "diff_url": "https://github.com/octocat/Hello-World/compare/v1.0.0...v1.1.0.diff",
            "patch_url": "https://github.com/octocat/Hello-World/compare/v1.0.0...v1.1.0.patch",
            "url": "https://api.github.com/repos/octocat/Hello-World/compare/v1.0.0...v1.1.0"
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/compare/v1.0.0...v1.1.0"))
            .and(header("Authorization", format!("Bearer {}", test_token)))
            .respond_with(ResponseTemplate::new(200).set_body_json(comparison_json))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .compare("octocat", "Hello-World", "v1.0.0", "v1.1.0")
            .await;

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let cmp = result.unwrap();
        assert_eq!(cmp.status, "ahead");
        assert_eq!(cmp.ahead_by, 5);
        assert_eq!(cmp.behind_by, 0);
        assert_eq!(cmp.total_commits, 5);
        assert_eq!(cmp.commits.len(), 5);
        assert_eq!(cmp.files.len(), 1);
        assert_eq!(cmp.files[0].filename, "src/main.rs");
        assert_eq!(
            cmp.url,
            "https://api.github.com/repos/octocat/Hello-World/compare/v1.0.0...v1.1.0"
        );
    }

    /// compare_commits with identical refs returns "identical" status.
    ///
    /// Covers assertion 33.
    #[tokio::test]
    async fn test_compare_commits_identical() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";
        let sha = "abc123sha";

        let the_commit = full_commit_json(sha);

        let comparison_json = serde_json::json!({
            "base_commit": the_commit,
            "merge_base_commit": the_commit,
            "head_commit": the_commit,
            "status": "identical",
            "ahead_by": 0,
            "behind_by": 0,
            "total_commits": 0,
            "commits": [],
            "files": [],
            "html_url": "https://github.com/octocat/Hello-World/compare/v1.0.0...v1.0.0",
            "permalink_url": "https://github.com/octocat/Hello-World/compare/v1.0.0...v1.0.0",
            "diff_url": "https://github.com/octocat/Hello-World/compare/v1.0.0...v1.0.0.diff",
            "patch_url": "https://github.com/octocat/Hello-World/compare/v1.0.0...v1.0.0.patch",
            "url": "https://api.github.com/repos/octocat/Hello-World/compare/v1.0.0...v1.0.0"
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/compare/v1.0.0...v1.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(comparison_json))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .compare("octocat", "Hello-World", "v1.0.0", "v1.0.0")
            .await;

        assert!(result.is_ok());
        let cmp = result.unwrap();
        assert_eq!(cmp.status, "identical");
        assert_eq!(cmp.ahead_by, 0);
        assert_eq!(cmp.behind_by, 0);
        assert_eq!(cmp.total_commits, 0);
        assert!(cmp.commits.is_empty());
        assert!(cmp.files.is_empty());
    }

    /// compare_commits includes file changes with correct statistics.
    ///
    /// Covers assertion 35.
    #[tokio::test]
    async fn test_compare_commits_with_file_changes() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        let commit = full_commit_json("diffsha");

        let comparison_json = serde_json::json!({
            "base_commit": commit,
            "merge_base_commit": commit,
            "head_commit": commit,
            "status": "ahead",
            "ahead_by": 1,
            "behind_by": 0,
            "total_commits": 1,
            "commits": [commit],
            "files": [
                {
                    "filename": "src/new.rs",
                    "status": "added",
                    "additions": 50,
                    "deletions": 0,
                    "changes": 50,
                    "blob_url": "https://github.com/o/r/blob/diffsha/src/new.rs",
                    "raw_url": "https://github.com/o/r/raw/diffsha/src/new.rs",
                    "contents_url": "https://api.github.com/repos/o/r/contents/src/new.rs",
                    "patch": "@@ -0,0 +1,50 @@\n+new file content"
                },
                {
                    "filename": "src/renamed.rs",
                    "status": "renamed",
                    "additions": 0,
                    "deletions": 0,
                    "changes": 0,
                    "blob_url": "https://github.com/o/r/blob/diffsha/src/renamed.rs",
                    "raw_url": "https://github.com/o/r/raw/diffsha/src/renamed.rs",
                    "contents_url": "https://api.github.com/repos/o/r/contents/src/renamed.rs",
                    "previous_filename": "src/old.rs"
                },
                {
                    "filename": "deleted.rs",
                    "status": "removed",
                    "additions": 0,
                    "deletions": 100,
                    "changes": 100,
                    "blob_url": "https://github.com/o/r/blob/diffsha/deleted.rs",
                    "raw_url": "https://github.com/o/r/raw/diffsha/deleted.rs",
                    "contents_url": "https://api.github.com/repos/o/r/contents/deleted.rs"
                }
            ],
            "html_url": "https://github.com/octocat/Hello-World/compare/base...head",
            "permalink_url": "https://github.com/octocat/Hello-World/compare/base...head",
            "diff_url": "https://github.com/octocat/Hello-World/compare/base...head.diff",
            "patch_url": "https://github.com/octocat/Hello-World/compare/base...head.patch",
            "url": "https://api.github.com/repos/octocat/Hello-World/compare/base...head"
        });

        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/compare/base...head"))
            .respond_with(ResponseTemplate::new(200).set_body_json(comparison_json))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .compare("octocat", "Hello-World", "base", "head")
            .await;

        assert!(result.is_ok());
        let cmp = result.unwrap();
        assert_eq!(cmp.files.len(), 3);

        let added = &cmp.files[0];
        assert_eq!(added.status, "added");
        assert_eq!(added.additions, 50);
        assert!(added.patch.is_some());

        let renamed = &cmp.files[1];
        assert_eq!(renamed.status, "renamed");
        assert_eq!(renamed.previous_filename.as_deref(), Some("src/old.rs"));

        let removed = &cmp.files[2];
        assert_eq!(removed.status, "removed");
        assert_eq!(removed.deletions, 100);
        assert!(removed.patch.is_none());
    }

    /// compare_commits returns ApiError::NotFound for a 404 response.
    #[tokio::test]
    async fn test_compare_commits_404_returns_not_found() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path_regex(r"/repos/octocat/Hello-World/compare/.*"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "message": "Not Found"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .compare("octocat", "Hello-World", "bad-base", "bad-head")
            .await;

        assert!(matches!(result, Err(ApiError::NotFound)));
    }

    /// compare_commits returns ApiError::AuthorizationFailed for a 403 response.
    #[tokio::test]
    async fn test_compare_commits_403_returns_authorization_failed() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path_regex(r"/repos/octocat/Hello-World/compare/.*"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "message": "Forbidden"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .compare("octocat", "Hello-World", "base", "head")
            .await;

        assert!(matches!(result, Err(ApiError::AuthorizationFailed)));
    }

    /// compare_commits returns ApiError::AuthenticationFailed for a 401 response.
    #[tokio::test]
    async fn test_compare_commits_401_returns_authentication_failed() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path_regex(r"/repos/octocat/Hello-World/compare/.*"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "message": "Requires authentication"
            })))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .compare("octocat", "Hello-World", "base", "head")
            .await;

        assert!(matches!(result, Err(ApiError::AuthenticationFailed)));
    }

    /// compare_commits returns ApiError::HttpError for unexpected server errors (5xx).
    #[tokio::test]
    async fn test_compare_commits_server_error() {
        let mock_server = MockServer::start().await;
        let test_token = "ghs_test_token";

        Mock::given(method("GET"))
            .and(path_regex(r"/repos/octocat/Hello-World/compare/.*"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .mount(&mock_server)
            .await;

        let client = build_client(test_token, &mock_server.uri()).await;
        let result = client
            .compare("octocat", "Hello-World", "base", "head")
            .await;

        match result.unwrap_err() {
            ApiError::HttpError { status, .. } => assert_eq!(status, 503),
            e => panic!("Expected HttpError, got: {:?}", e),
        }
    }

    // -------------------------------------------------------------------------
    // Helper
    // -------------------------------------------------------------------------

    async fn build_client(token: &str, mock_server_uri: &str) -> RepositoriesClient {
        let auth = MockAuthProvider::new_with_token(token);
        let github_client = GitHubClient::builder(auth)
            .config(ClientConfig::default().with_github_api_url(mock_server_uri.to_string()))
            .build()
            .unwrap();

        let installation_id = InstallationId::new(12345);
        github_client
            .installation_by_id(installation_id)
            .await
            .unwrap()
            .repositories()
    }
}
