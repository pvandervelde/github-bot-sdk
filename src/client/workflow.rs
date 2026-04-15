// Spec: docs/specs/interfaces/additional-operations.md
// Workflow and workflow run operations for GitHub API

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::InstallationClient;
use crate::error::ApiError;

/// State of a GitHub Actions workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    /// Workflow is active.
    Active,
    /// Workflow was disabled manually by a repository admin.
    DisabledManually,
    /// Workflow was disabled automatically due to inactivity.
    DisabledInactivity,
    /// Workflow is disabled because the repo is a fork.
    DisabledFork,
    /// Workflow has been deleted.
    Deleted,
}

impl fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowState::Active => write!(f, "active"),
            WorkflowState::DisabledManually => write!(f, "disabled_manually"),
            WorkflowState::DisabledInactivity => write!(f, "disabled_inactivity"),
            WorkflowState::DisabledFork => write!(f, "disabled_fork"),
            WorkflowState::Deleted => write!(f, "deleted"),
        }
    }
}

/// GitHub Actions workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique workflow identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Workflow name
    pub name: String,

    /// Workflow file path
    pub path: String,

    /// Workflow state
    pub state: WorkflowState,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Workflow URL
    pub url: String,

    /// Workflow HTML URL
    pub html_url: String,

    /// Workflow badge URL
    pub badge_url: String,
}

/// Status of a GitHub Actions workflow run.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunStatus {
    /// Run is queued and waiting to start.
    Queued,
    /// Run is currently executing.
    InProgress,
    /// Run has finished.
    #[default]
    Completed,
    /// Run is waiting on a required check or deployment protection rule.
    Waiting,
    /// Run has been requested.
    Requested,
    /// Run is pending.
    Pending,
}

impl fmt::Display for WorkflowRunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowRunStatus::Queued => write!(f, "queued"),
            WorkflowRunStatus::InProgress => write!(f, "in_progress"),
            WorkflowRunStatus::Completed => write!(f, "completed"),
            WorkflowRunStatus::Waiting => write!(f, "waiting"),
            WorkflowRunStatus::Requested => write!(f, "requested"),
            WorkflowRunStatus::Pending => write!(f, "pending"),
        }
    }
}

/// Conclusion of a completed GitHub Actions workflow run.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunConclusion {
    /// Run completed successfully.
    #[default]
    Success,
    /// Run failed.
    Failure,
    /// Run was cancelled.
    Cancelled,
    /// Run was skipped.
    Skipped,
    /// Run timed out.
    TimedOut,
    /// Run requires manual action.
    ActionRequired,
    /// Run result is stale.
    Stale,
    /// Run completed with a neutral result.
    Neutral,
}

impl fmt::Display for WorkflowRunConclusion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowRunConclusion::Success => write!(f, "success"),
            WorkflowRunConclusion::Failure => write!(f, "failure"),
            WorkflowRunConclusion::Cancelled => write!(f, "cancelled"),
            WorkflowRunConclusion::Skipped => write!(f, "skipped"),
            WorkflowRunConclusion::TimedOut => write!(f, "timed_out"),
            WorkflowRunConclusion::ActionRequired => write!(f, "action_required"),
            WorkflowRunConclusion::Stale => write!(f, "stale"),
            WorkflowRunConclusion::Neutral => write!(f, "neutral"),
        }
    }
}

/// GitHub Actions workflow run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    /// Unique workflow run identifier
    pub id: u64,

    /// Node ID for GraphQL API
    pub node_id: String,

    /// Workflow run name
    pub name: String,

    /// Workflow run number
    pub run_number: u64,

    /// Event that triggered the workflow
    pub event: String,

    /// Workflow run status
    pub status: WorkflowRunStatus,

    /// Workflow run conclusion (if completed)
    pub conclusion: Option<WorkflowRunConclusion>,

    /// Workflow ID
    pub workflow_id: u64,

    /// Head branch
    pub head_branch: String,

    /// Head commit SHA
    pub head_sha: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Workflow run URL
    pub url: String,

    /// Workflow run HTML URL
    pub html_url: String,
}

/// Request to trigger a workflow.
#[derive(Debug, Clone, Serialize)]
pub struct TriggerWorkflowRequest {
    /// Git reference (branch or tag)
    #[serde(rename = "ref")]
    pub git_ref: String,

    /// Workflow inputs (key-value pairs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<std::collections::HashMap<String, String>>,
}

/// Domain client for GitHub Actions workflow and run operations.
///
/// Obtained via [`InstallationClient::workflows()`]. Cheap to clone (Arc-backed).
///
/// See docs/specs/interfaces/additional-operations.md
#[derive(Debug, Clone)]
pub struct WorkflowsClient {
    client: InstallationClient,
}

impl WorkflowsClient {
    pub(crate) fn new(client: InstallationClient) -> Self {
        Self { client }
    }

    /// List workflows in a repository.
    ///
    /// Retrieves all GitHub Actions workflows for a repository.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    ///
    /// # Returns
    ///
    /// Returns vector of workflows.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Repository does not exist
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::WorkflowsClient;
    /// # async fn example(client: &WorkflowsClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let workflows = client.list("owner", "repo").await?;
    /// for workflow in workflows {
    ///     println!("Workflow: {} ({})", workflow.name, workflow.state);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, owner: &str, repo: &str) -> Result<Vec<Workflow>, ApiError> {
        let path = format!("/repos/{}/{}/actions/workflows", owner, repo);
        let response = self.client.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }

        #[derive(Deserialize)]
        struct WorkflowsResponse {
            workflows: Vec<Workflow>,
        }

        let workflows_response: WorkflowsResponse =
            response.json().await.map_err(ApiError::from)?;
        Ok(workflows_response.workflows)
    }

    /// Get a specific workflow by ID.
    ///
    /// Retrieves details about a single workflow.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `workflow_id` - Workflow ID
    ///
    /// # Returns
    ///
    /// Returns the `Workflow` with the specified ID.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Workflow does not exist
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::WorkflowsClient;
    /// # async fn example(client: &WorkflowsClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let workflow = client.get("owner", "repo", 123456).await?;
    /// println!("Workflow: {} at {}", workflow.name, workflow.path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: u64,
    ) -> Result<Workflow, ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/workflows/{}",
            owner, repo, workflow_id
        );
        let response = self.client.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }

        response.json().await.map_err(ApiError::from)
    }

    /// Trigger a workflow run.
    ///
    /// Manually triggers a workflow run using workflow_dispatch event.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `workflow_id` - Workflow ID
    /// * `request` - Trigger parameters (ref and optional inputs)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful trigger.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Workflow does not exist
    /// * `ApiError::InvalidRequest` - Workflow not configured for manual dispatch
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::{WorkflowsClient, TriggerWorkflowRequest};
    /// # async fn example(client: &WorkflowsClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = TriggerWorkflowRequest {
    ///     git_ref: "main".to_string(),
    ///     inputs: None,
    /// };
    /// client.trigger("owner", "repo", 123456, request).await?;
    /// println!("Workflow triggered");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn trigger(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: u64,
        request: TriggerWorkflowRequest,
    ) -> Result<(), ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/workflows/{}/dispatches",
            owner, repo, workflow_id
        );
        let response = self.client.post(&path, &request).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }

        Ok(())
    }

    // ========================================================================
    // Workflow Run Operations
    // ========================================================================

    /// List workflow runs for a workflow.
    ///
    /// Retrieves workflow runs for a specific workflow.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `workflow_id` - Workflow ID
    ///
    /// # Returns
    ///
    /// Returns vector of workflow runs.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Workflow does not exist
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::WorkflowsClient;
    /// # async fn example(client: &WorkflowsClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let runs = client.list_runs("owner", "repo", 123456).await?;
    /// for run in runs {
    ///     println!("Run #{}: {} ({})", run.run_number, run.status, run.conclusion.unwrap_or_default());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_runs(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: u64,
    ) -> Result<Vec<WorkflowRun>, ApiError> {
        let path = format!(
            "/repos/{}/{}/actions/workflows/{}/runs",
            owner, repo, workflow_id
        );
        let response = self.client.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }

        #[derive(Deserialize)]
        struct WorkflowRunsResponse {
            workflow_runs: Vec<WorkflowRun>,
        }

        let runs_response: WorkflowRunsResponse = response.json().await.map_err(ApiError::from)?;
        Ok(runs_response.workflow_runs)
    }

    /// Get a specific workflow run by ID.
    ///
    /// Retrieves details about a single workflow run.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `run_id` - Workflow run ID
    ///
    /// # Returns
    ///
    /// Returns the `WorkflowRun` with the specified ID.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Workflow run does not exist
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::WorkflowsClient;
    /// # async fn example(client: &WorkflowsClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let run = client.get_run("owner", "repo", 987654).await?;
    /// println!("Run #{}: {}", run.run_number, run.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_run(
        &self,
        owner: &str,
        repo: &str,
        run_id: u64,
    ) -> Result<WorkflowRun, ApiError> {
        let path = format!("/repos/{}/{}/actions/runs/{}", owner, repo, run_id);
        let response = self.client.get(&path).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }

        response.json().await.map_err(ApiError::from)
    }

    /// Cancel a workflow run.
    ///
    /// Cancels a workflow run that is in progress.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `run_id` - Workflow run ID
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful cancellation.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Workflow run does not exist
    /// * `ApiError::InvalidRequest` - Workflow run cannot be cancelled
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::WorkflowsClient;
    /// # async fn example(client: &WorkflowsClient) -> Result<(), Box<dyn std::error::Error>> {
    /// client.cancel_run("owner", "repo", 987654).await?;
    /// println!("Workflow run cancelled");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_run(&self, owner: &str, repo: &str, run_id: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/actions/runs/{}/cancel", owner, repo, run_id);
        let response = self.client.post(&path, &serde_json::json!({})).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }

        Ok(())
    }

    /// Re-run a workflow run.
    ///
    /// Re-runs a completed workflow run.
    ///
    /// # Arguments
    ///
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `run_id` - Workflow run ID
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful re-run trigger.
    ///
    /// # Errors
    ///
    /// * `ApiError::NotFound` - Workflow run does not exist
    /// * `ApiError::InvalidRequest` - Workflow run cannot be re-run
    /// * `ApiError::AuthorizationFailed` - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use github_bot_sdk::client::WorkflowsClient;
    /// # async fn example(client: &WorkflowsClient) -> Result<(), Box<dyn std::error::Error>> {
    /// client.rerun_run("owner", "repo", 987654).await?;
    /// println!("Workflow run re-triggered");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rerun_run(&self, owner: &str, repo: &str, run_id: u64) -> Result<(), ApiError> {
        let path = format!("/repos/{}/{}/actions/runs/{}/rerun", owner, repo, run_id);
        let response = self.client.post(&path, &serde_json::json!({})).await?;

        let status = response.status();
        if !status.is_success() {
            return Err(super::map_http_error(status, response).await);
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "workflow_tests.rs"]
mod tests;
