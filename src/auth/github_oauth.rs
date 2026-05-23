use serde::{Deserialize, Serialize};
use std::fmt;

pub const GITHUB_CLIENT_ID: &str = "Ov23liNZV0Yhd9IWb3bq";
pub const GITHUB_SCOPES: &str = "repo read:user user:email workflow";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
    pub bio: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AuthError {
    Network(String),
    Api(String),
    Pending,
    SlowDown,
    Expired,
    AccessDenied,
}

impl fmt::Display for AuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Network(message) => write!(formatter, "Network error: {message}"),
            AuthError::Api(message) => write!(formatter, "API error: {message}"),
            AuthError::Pending => write!(formatter, "Authorization pending"),
            AuthError::SlowDown => write!(formatter, "Polling too fast, slowing down"),
            AuthError::Expired => write!(formatter, "Device code expired"),
            AuthError::AccessDenied => write!(formatter, "Access denied by user"),
        }
    }
}

impl std::error::Error for AuthError {}

/// GitHub uses an `error` field in JSON responses to signal device flow states
/// rather than HTTP status codes.
#[derive(Deserialize)]
struct OAuthErrorResponse {
    error: String,
    #[serde(default)]
    error_description: String,
}

fn build_http_client() -> Result<reqwest::blocking::Client, AuthError> {
    reqwest::blocking::Client::builder()
        .user_agent("Palimpsest")
        .build()
        .map_err(|error| AuthError::Network(error.to_string()))
}

pub fn request_device_code(client_id: &str) -> Result<DeviceCodeResponse, AuthError> {
    let http_client = build_http_client()?;

    let response = http_client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", client_id), ("scope", GITHUB_SCOPES)])
        .send()
        .map_err(|error| AuthError::Network(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AuthError::Api(format!(
            "GitHub returned status {}",
            response.status()
        )));
    }

    response
        .json::<DeviceCodeResponse>()
        .map_err(|error| AuthError::Api(format!("Failed to parse device code response: {error}")))
}

pub fn poll_for_token(client_id: &str, device_code: &str) -> Result<TokenResponse, AuthError> {
    let http_client = build_http_client()?;

    let response = http_client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("device_code", device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .map_err(|error| AuthError::Network(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AuthError::Api(format!(
            "GitHub returned status {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .map_err(|error| AuthError::Network(error.to_string()))?;

    if let Ok(token_response) = serde_json::from_str::<TokenResponse>(&body) {
        return Ok(token_response);
    }

    if let Ok(error_response) = serde_json::from_str::<OAuthErrorResponse>(&body) {
        return Err(match error_response.error.as_str() {
            "authorization_pending" => AuthError::Pending,
            "slow_down" => AuthError::SlowDown,
            "expired_token" => AuthError::Expired,
            "access_denied" => AuthError::AccessDenied,
            _ => AuthError::Api(format!(
                "{}: {}",
                error_response.error, error_response.error_description
            )),
        });
    }

    Err(AuthError::Api(format!(
        "Unexpected response from GitHub: {body}"
    )))
}

pub fn fetch_user_profile(token: &str) -> Result<GitHubUser, AuthError> {
    let http_client = build_http_client()?;

    let response = http_client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .map_err(|error| AuthError::Network(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AuthError::Api(format!(
            "GitHub API returned status {}",
            response.status()
        )));
    }

    response
        .json::<GitHubUser>()
        .map_err(|error| AuthError::Api(format!("Failed to parse user profile: {error}")))
}

pub fn validate_token(token: &str) -> bool {
    fetch_user_profile(token).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_error_display_formats_correctly() {
        assert_eq!(
            format!("{}", AuthError::Network("timeout".into())),
            "Network error: timeout"
        );
        assert_eq!(
            format!("{}", AuthError::Api("bad request".into())),
            "API error: bad request"
        );
        assert_eq!(format!("{}", AuthError::Pending), "Authorization pending");
        assert_eq!(
            format!("{}", AuthError::SlowDown),
            "Polling too fast, slowing down"
        );
        assert_eq!(format!("{}", AuthError::Expired), "Device code expired");
        assert_eq!(
            format!("{}", AuthError::AccessDenied),
            "Access denied by user"
        );
    }

    #[test]
    fn auth_error_implements_std_error() {
        let error: &dyn std::error::Error = &AuthError::Pending;
        assert_eq!(error.to_string(), "Authorization pending");
    }

    #[test]
    fn device_code_response_serialization_roundtrip() {
        let response = DeviceCodeResponse {
            device_code: "abc123".into(),
            user_code: "ABCD-1234".into(),
            verification_uri: "https://github.com/login/device".into(),
            expires_in: 900,
            interval: 5,
        };
        let serialized = serde_json::to_string(&response).expect("serialization should succeed");
        let deserialized: DeviceCodeResponse =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(response, deserialized);
    }

    #[test]
    fn token_response_deserialization() {
        let json = r#"{"access_token":"gho_abc","token_type":"bearer","scope":"repo"}"#;
        let response: TokenResponse =
            serde_json::from_str(json).expect("deserialization should succeed");
        assert_eq!(response.access_token, "gho_abc");
        assert_eq!(response.token_type, "bearer");
        assert_eq!(response.scope, "repo");
    }

    #[test]
    fn github_user_deserialization_with_optional_fields() {
        let json = r#"{
            "login": "testuser",
            "name": null,
            "email": null,
            "avatar_url": "https://avatars.githubusercontent.com/u/1",
            "html_url": "https://github.com/testuser",
            "bio": null
        }"#;
        let user: GitHubUser = serde_json::from_str(json).expect("deserialization should succeed");
        assert_eq!(user.login, "testuser");
        assert!(user.name.is_none());
        assert!(user.email.is_none());
        assert!(user.bio.is_none());
    }

    #[test]
    fn oauth_error_response_parsing() {
        let json = r#"{"error":"authorization_pending","error_description":"waiting for user"}"#;
        let error: OAuthErrorResponse =
            serde_json::from_str(json).expect("deserialization should succeed");
        assert_eq!(error.error, "authorization_pending");
        assert_eq!(error.error_description, "waiting for user");
    }
}
