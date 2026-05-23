use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHubRepo {
    pub full_name: String,
    pub name: String,
    pub description: Option<String>,
    pub private: bool,
    pub html_url: String,
    pub default_branch: String,
    pub stargazers_count: u64,
    pub language: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub user_login: String,
    pub created_at: String,
    pub updated_at: String,
    pub html_url: String,
    pub head_ref: String,
    pub base_ref: String,
    pub draft: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActionRun {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub head_branch: String,
    pub event: String,
}

/// GitHub's list pull requests API nests the user login inside a `user` object.
#[derive(Deserialize)]
struct PullRequestApiResponse {
    number: u64,
    title: String,
    state: String,
    user: PullRequestUser,
    created_at: String,
    updated_at: String,
    html_url: String,
    head: PullRequestRef,
    base: PullRequestRef,
    #[serde(default)]
    draft: bool,
}

#[derive(Deserialize)]
struct PullRequestUser {
    login: String,
}

#[derive(Deserialize)]
struct PullRequestRef {
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Deserialize)]
struct WorkflowRunsResponse {
    workflow_runs: Vec<ActionRun>,
}

fn build_authenticated_client(token: &str) -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("Palimpsest")
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            let auth_value = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|error| format!("Invalid token format: {error}"))?;
            headers.insert(reqwest::header::AUTHORIZATION, auth_value);

            let accept_value =
                reqwest::header::HeaderValue::from_static("application/vnd.github.v3+json");
            headers.insert(reqwest::header::ACCEPT, accept_value);

            headers
        })
        .build()
        .map_err(|error| format!("Failed to build HTTP client: {error}"))
}

pub fn list_user_repos(token: &str) -> Result<Vec<GitHubRepo>, String> {
    let http_client = build_authenticated_client(token)?;

    let response = http_client
        .get("https://api.github.com/user/repos?sort=updated&per_page=30&affiliation=owner,collaborator")
        .send()
        .map_err(|error| format!("Failed to fetch repos: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned status {} when fetching repos",
            response.status()
        ));
    }

    response
        .json::<Vec<GitHubRepo>>()
        .map_err(|error| format!("Failed to parse repos response: {error}"))
}

pub fn list_pull_requests(
    token: &str,
    owner: &str,
    repo: &str,
) -> Result<Vec<PullRequest>, String> {
    let http_client = build_authenticated_client(token)?;

    let url = format!("https://api.github.com/repos/{owner}/{repo}/pulls?state=open");
    let response = http_client
        .get(&url)
        .send()
        .map_err(|error| format!("Failed to fetch pull requests: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned status {} when fetching pull requests for {owner}/{repo}",
            response.status()
        ));
    }

    let api_responses: Vec<PullRequestApiResponse> = response
        .json()
        .map_err(|error| format!("Failed to parse pull requests response: {error}"))?;

    let pull_requests = api_responses
        .into_iter()
        .map(|api_pull_request| PullRequest {
            number: api_pull_request.number,
            title: api_pull_request.title,
            state: api_pull_request.state,
            user_login: api_pull_request.user.login,
            created_at: api_pull_request.created_at,
            updated_at: api_pull_request.updated_at,
            html_url: api_pull_request.html_url,
            head_ref: api_pull_request.head.ref_name,
            base_ref: api_pull_request.base.ref_name,
            draft: api_pull_request.draft,
        })
        .collect();

    Ok(pull_requests)
}

pub fn list_action_runs(token: &str, owner: &str, repo: &str) -> Result<Vec<ActionRun>, String> {
    let http_client = build_authenticated_client(token)?;

    let url = format!("https://api.github.com/repos/{owner}/{repo}/actions/runs?per_page=10");
    let response = http_client
        .get(&url)
        .send()
        .map_err(|error| format!("Failed to fetch action runs: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned status {} when fetching action runs for {owner}/{repo}",
            response.status()
        ));
    }

    let workflow_response: WorkflowRunsResponse = response
        .json()
        .map_err(|error| format!("Failed to parse action runs response: {error}"))?;

    Ok(workflow_response.workflow_runs)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHubRelease {
    pub name: Option<String>,
    pub tag_name: String,
    pub body: Option<String>,
    pub html_url: String,
    pub draft: bool,
    pub prerelease: bool,
    pub published_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHubPackage {
    pub name: String,
    pub package_type: String,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
struct RepoOwner {
    #[serde(rename = "type")]
    owner_type: String,
}

#[derive(Deserialize)]
struct RepoResponse {
    owner: RepoOwner,
}

pub fn get_repo_owner_type(token: &str, owner: &str, repo: &str) -> Result<String, String> {
    let http_client = build_authenticated_client(token)?;
    let url = format!("https://api.github.com/repos/{owner}/{repo}");
    let response = http_client
        .get(&url)
        .send()
        .map_err(|error| format!("Failed to fetch repo owner type: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned status {} when fetching repo details for {owner}/{repo}",
            response.status()
        ));
    }

    let repo_res: RepoResponse = response
        .json()
        .map_err(|error| format!("Failed to parse repo response: {error}"))?;

    Ok(repo_res.owner.owner_type)
}

pub fn list_releases(token: &str, owner: &str, repo: &str) -> Result<Vec<GitHubRelease>, String> {
    let http_client = build_authenticated_client(token)?;
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=20");
    let response = http_client
        .get(&url)
        .send()
        .map_err(|error| format!("Failed to fetch releases: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned status {} when fetching releases for {owner}/{repo}",
            response.status()
        ));
    }

    response
        .json::<Vec<GitHubRelease>>()
        .map_err(|error| format!("Failed to parse releases response: {error}"))
}

pub fn list_packages(token: &str, owner: &str, is_org: bool) -> Result<Vec<GitHubPackage>, String> {
    let http_client = build_authenticated_client(token)?;
    let prefix = if is_org { "orgs" } else { "users" };

    let mut packages = Vec::new();

    for pkg_type in &["container", "npm"] {
        let url = format!(
            "https://api.github.com/{prefix}/{owner}/packages?package_type={pkg_type}&per_page=20"
        );
        let response = http_client.get(&url).send();
        if let Ok(res) = response {
            if res.status().is_success() {
                if let Ok(mut list) = res.json::<Vec<GitHubPackage>>() {
                    packages.append(&mut list);
                }
            }
        }
    }

    Ok(packages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_repo_deserialization() {
        let json = r#"{
            "full_name": "user/repo",
            "name": "repo",
            "description": "A test repo",
            "private": false,
            "html_url": "https://github.com/user/repo",
            "default_branch": "main",
            "stargazers_count": 42,
            "language": "Rust",
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;
        let repo: GitHubRepo = serde_json::from_str(json).expect("deserialization should succeed");
        assert_eq!(repo.full_name, "user/repo");
        assert_eq!(repo.stargazers_count, 42);
        assert_eq!(repo.language, Some("Rust".into()));
        assert!(!repo.private);
    }

    #[test]
    fn github_repo_with_null_optional_fields() {
        let json = r#"{
            "full_name": "user/repo",
            "name": "repo",
            "description": null,
            "private": true,
            "html_url": "https://github.com/user/repo",
            "default_branch": "main",
            "stargazers_count": 0,
            "language": null,
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;
        let repo: GitHubRepo = serde_json::from_str(json).expect("deserialization should succeed");
        assert!(repo.description.is_none());
        assert!(repo.language.is_none());
        assert!(repo.private);
    }

    #[test]
    fn pull_request_api_response_deserialization() {
        let json = r#"{
            "number": 1,
            "title": "Fix bug",
            "state": "open",
            "user": {"login": "contributor"},
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "html_url": "https://github.com/user/repo/pull/1",
            "head": {"ref": "fix-branch"},
            "base": {"ref": "main"},
            "draft": false
        }"#;
        let api_response: PullRequestApiResponse =
            serde_json::from_str(json).expect("deserialization should succeed");
        assert_eq!(api_response.number, 1);
        assert_eq!(api_response.user.login, "contributor");
        assert_eq!(api_response.head.ref_name, "fix-branch");
        assert_eq!(api_response.base.ref_name, "main");
    }

    #[test]
    fn action_run_deserialization() {
        let json = r#"{
            "id": 12345,
            "name": "CI",
            "status": "completed",
            "conclusion": "success",
            "html_url": "https://github.com/user/repo/actions/runs/12345",
            "created_at": "2024-01-01T00:00:00Z",
            "head_branch": "main",
            "event": "push"
        }"#;
        let action_run: ActionRun =
            serde_json::from_str(json).expect("deserialization should succeed");
        assert_eq!(action_run.id, 12345);
        assert_eq!(action_run.conclusion, Some("success".into()));
    }

    #[test]
    fn workflow_runs_response_deserialization() {
        let json = r#"{
            "workflow_runs": [
                {
                    "id": 1,
                    "name": "Build",
                    "status": "in_progress",
                    "conclusion": null,
                    "html_url": "https://github.com/user/repo/actions/runs/1",
                    "created_at": "2024-01-01T00:00:00Z",
                    "head_branch": "dev",
                    "event": "pull_request"
                }
            ]
        }"#;
        let response: WorkflowRunsResponse =
            serde_json::from_str(json).expect("deserialization should succeed");
        assert_eq!(response.workflow_runs.len(), 1);
        assert_eq!(response.workflow_runs[0].name, "Build");
        assert!(response.workflow_runs[0].conclusion.is_none());
    }

    #[test]
    fn pull_request_serialization_roundtrip() {
        let pull_request = PullRequest {
            number: 42,
            title: "Add feature".into(),
            state: "open".into(),
            user_login: "author".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-02T00:00:00Z".into(),
            html_url: "https://github.com/user/repo/pull/42".into(),
            head_ref: "feature-branch".into(),
            base_ref: "main".into(),
            draft: true,
        };
        let serialized =
            serde_json::to_string(&pull_request).expect("serialization should succeed");
        let deserialized: PullRequest =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(pull_request, deserialized);
    }
}
