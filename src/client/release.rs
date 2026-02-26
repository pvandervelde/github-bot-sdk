// Release and release asset operations for GitHub API

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::issue::IssueUser;
use crate::client::InstallationClient;
use crate::error::ApiError;

/// GitHub release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// Unique release identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Release tag name
    pub tag_name: String,

    /// Target commitish (branch or commit SHA)
    pub target_commitish: String,

    /// Release name
    pub name: Option<String>,

    /// Release body (Markdown)
    pub body: Option<String>,

    /// Whether this is a draft release
    pub draft: bool,

    /// Whether this is a prerelease
    pub prerelease: bool,

    /// User who created the release
    pub author: IssueUser,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Publication timestamp
    pub published_at: Option<DateTime<Utc>>,

    /// Release URL
    pub url: String,

    /// Release HTML URL
    pub html_url: String,

    /// Release assets
    pub assets: Vec<ReleaseAsset>,
}

/// Asset attached to a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    /// Unique asset identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Asset filename
    pub name: String,

    /// Asset label
    pub label: Option<String>,

    /// Asset content type
    pub content_type: String,

    /// Asset state
    pub state: String, // "uploaded", "open"

    /// Asset size in bytes
    pub size: u64,

    /// Download count
    pub download_count: u64,

    /// User who uploaded the asset
    pub uploader: IssueUser,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Asset download URL
    pub browser_download_url: String,
}

/// Request to create a release.
#[derive(Debug, Clone, Serialize)]
pub struct CreateReleaseRequest {
    /// Tag name (required)
    pub tag_name: String,

    /// Target commitish (branch or commit SHA)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_commitish: Option<String>,

    /// Release name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Release body (Markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Whether to create as draft
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,

    /// Whether to mark as prerelease
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<bool>,

    /// Whether to automatically generate the name and body for this release.
    ///
    /// When set to `true`, GitHub will auto-generate the release name (if `name`
    /// is not provided) and the release notes body from merged pull requests and
    /// other repository activity since the previous release. If `name` is provided
    /// it is used as-is; if `body` is provided it is pre-pended to the generated
    /// notes. Defaults to `false`.
    ///
    /// # Example
    ///
    /// ```
    /// use github_bot_sdk::client::CreateReleaseRequest;
    ///
    /// let request = CreateReleaseRequest {
    ///     tag_name: "v1.2.0".to_string(),
    ///     target_commitish: None,
    ///     name: None,
    ///     body: None,
    ///     draft: None,
    ///     prerelease: None,
    ///     generate_release_notes: Some(true),
    /// };
    /// ```
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_release_notes: Option<bool>,
}

/// Request to update a release.
///
/// Note: `generate_release_notes` is intentionally absent from this type.
/// The GitHub Update Release endpoint does not accept that parameter — it is
/// only valid on the Create Release endpoint.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateReleaseRequest {
    /// Tag name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,

    /// Target commitish
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_commitish: Option<String>,

    /// Release name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Release body (Markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Whether this is a draft
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,

    /// Whether this is a prerelease
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerelease: Option<bool>,
}

impl InstallationClient {
    // ========================================================================
    // Release Operations
    // ========================================================================

    /// List releases in a repository.
    ///
    /// Retrieves all releases for a repository, including drafts and prereleases.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    ///
    /// Returns vector of releases ordered by creation date (newest first).
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Repository does not exist
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let releases = client.list_releases("owner", "repo").await?;
    /// for release in releases {
    ///     println!("Release: {} ({})", release.name.unwrap_or_default(), release.tag_name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_releases(&self, owner: &str, repo: &str) -> Result<Vec<Release>, ApiError> {
        let path = format!("/repos/{}/{}/releases", owner, repo);
        let response = self.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
                _ => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    ApiError::HttpError {
                        status: status.as_u16(),
                        message,
                    }
                }
            });
        }

        response.json().await.map_err(ApiError::from)
    }

    /// Get the latest published release.
    ///
    /// Returns the most recent non-draft, non-prerelease release.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    ///
    /// Returns the latest published `Release`.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Repository or no published releases exist
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let release = client.get_latest_release("owner", "repo").await?;
    /// println!("Latest: {} ({})", release.name.unwrap_or_default(), release.tag_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_latest_release(&self, owner: &str, repo: &str) -> Result<Release, ApiError> {
        let path = format!("/repos/{}/{}/releases/latest", owner, repo);
        let response = self.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
                _ => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    ApiError::HttpError {
                        status: status.as_u16(),
                        message,
                    }
                }
            });
        }

        response.json().await.map_err(ApiError::from)
    }

    /// Get a release by tag name.
    ///
    /// Retrieves a release by its git tag name.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `tag` - Git tag name
    ///
    /// # Returns
    ///
    /// Returns the `Release` with the specified tag.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Release with tag does not exist
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let release = client.get_release_by_tag("owner", "repo", "v1.0.0").await?;
    /// println!("Release: {}", release.tag_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_release_by_tag(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
    ) -> Result<Release, ApiError> {
        let encoded_tag = urlencoding::encode(tag);
        let path = format!("/repos/{}/{}/releases/tags/{}", owner, repo, encoded_tag);
        let response = self.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
                _ => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    ApiError::HttpError {
                        status: status.as_u16(),
                        message,
                    }
                }
            });
        }

        response.json().await.map_err(ApiError::from)
    }

    /// Get a release by ID.
    ///
    /// Retrieves a release by its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `release_id` - Release ID
    ///
    /// # Returns
    ///
    /// Returns the `Release` with the specified ID.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Release does not exist
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let release = client.get_release("owner", "repo", 12345).await?;
    /// println!("Release: {}", release.tag_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_release(
        &self,
        owner: &str,
        repo: &str,
        release_id: u64,
    ) -> Result<Release, ApiError> {
        let path = format!("/repos/{}/{}/releases/{}", owner, repo, release_id);
        let response = self.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
                _ => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    ApiError::HttpError {
                        status: status.as_u16(),
                        message,
                    }
                }
            });
        }

        response.json().await.map_err(ApiError::from)
    }

    /// Create a new release.
    ///
    /// Creates a new release for a repository. Can create published releases,
    /// drafts, or prereleases.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `request` - Release creation parameters
    ///
    /// # Returns
    ///
    /// Returns the created `Release`.
    ///
    /// # Errors
    ///
    /// * `ApiError::InvalidRequest` - Tag already exists or invalid parameters
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::{InstallationClient, CreateReleaseRequest};
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = CreateReleaseRequest {
    ///     tag_name: "v1.0.0".to_string(),
    ///     name: Some("Version 1.0.0".to_string()),
    ///     body: Some("Release notes".to_string()),
    ///     draft: Some(false),
    ///     prerelease: Some(false),
    ///     target_commitish: None,
    ///     generate_release_notes: None,
    /// };
    /// let release = client.create_release("owner", "repo", request).await?;
    /// println!("Created release: {}", release.tag_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_release(
        &self,
        owner: &str,
        repo: &str,
        request: CreateReleaseRequest,
    ) -> Result<Release, ApiError> {
        let path = format!("/repos/{}/{}/releases", owner, repo);
        let response = self.post(&path, &request).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
                422 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Validation error".to_string());
                    ApiError::InvalidRequest { message }
                }
                _ => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    ApiError::HttpError {
                        status: status.as_u16(),
                        message,
                    }
                }
            });
        }

        response.json().await.map_err(ApiError::from)
    }

    /// Update an existing release.
    ///
    /// Updates release properties. Only specified fields are modified.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `release_id` - Release ID
    /// * `request` - Fields to update
    ///
    /// # Returns
    ///
    /// Returns the updated `Release`.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Release does not exist
    /// * `ApiError::InvalidRequest` - Invalid parameters
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::{InstallationClient, UpdateReleaseRequest};
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = UpdateReleaseRequest {
    ///     name: Some("Updated name".to_string()),
    ///     body: Some("Updated notes".to_string()),
    ///     ..Default::default()
    /// };
    /// let release = client.update_release("owner", "repo", 12345, request).await?;
    /// println!("Updated release: {}", release.tag_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_release(
        &self,
        owner: &str,
        repo: &str,
        release_id: u64,
        request: UpdateReleaseRequest,
    ) -> Result<Release, ApiError> {
        let path = format!("/repos/{}/{}/releases/{}", owner, repo, release_id);
        let response = self.patch(&path, &request).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
                422 => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Validation error".to_string());
                    ApiError::InvalidRequest { message }
                }
                _ => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    ApiError::HttpError {
                        status: status.as_u16(),
                        message,
                    }
                }
            });
        }

        response.json().await.map_err(ApiError::from)
    }

    /// Delete a release.
    ///
    /// Deletes a release. Does not delete the associated git tag.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `release_id` - Release ID
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful deletion.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Release does not exist
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::InstallationClient;
    /// # async fn example(client: &InstallationClient) -> Result<(), Box<dyn std::error::Error>> {
    /// client.delete_release("owner", "repo", 12345).await?;
    /// println!("Release deleted");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_release(
        &self,
        owner: &str,
        repo: &str,
        release_id: u64,
    ) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/releases/{}", owner, repo, release_id);
        let response = self.delete(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => ApiError::NotFound,
                403 => ApiError::AuthorizationFailed,
                401 => ApiError::AuthenticationFailed,
                _ => {
                    let message = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    ApiError::HttpError {
                        status: status.as_u16(),
                        message,
                    }
                }
            });
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "release_tests.rs"]
mod tests;
