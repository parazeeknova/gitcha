use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GitIdentity {
    pub name: Option<String>,
    pub email: Option<String>,
    pub signing_key: Option<String>,
    pub gpg_sign_commits: bool,
    pub ssh_keys: Vec<SshKeyInfo>,
    pub gpg_keys: Vec<GpgKeyInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SshKeyInfo {
    pub path: String,
    pub key_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GpgKeyInfo {
    pub key_id: String,
    pub uid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GhCliStatus {
    pub logged_in: bool,
    pub username: Option<String>,
}

fn run_git_config(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--global", key])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

pub fn detect_git_config() -> GitIdentity {
    let name = run_git_config("user.name");
    let email = run_git_config("user.email");
    let signing_key = run_git_config("user.signingkey");
    let gpg_sign_commits = run_git_config("commit.gpgsign")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    GitIdentity {
        name,
        email,
        signing_key,
        gpg_sign_commits,
        ssh_keys: detect_ssh_keys(),
        gpg_keys: detect_gpg_keys(),
    }
}

pub fn detect_gh_cli_auth() -> Option<GhCliStatus> {
    let output = Command::new("gh").args(["auth", "status"]).output().ok()?;

    // gh auth status outputs to stderr
    let combined_output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if combined_output.contains("not logged") || !output.status.success() {
        return Some(GhCliStatus {
            logged_in: false,
            username: None,
        });
    }

    // gh auth status output includes "Logged in to github.com account <username>"
    let username = combined_output
        .lines()
        .find(|line| line.contains("Logged in") || line.contains("account"))
        .and_then(|line| {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            let mut raw_token = None;
            if let Some(pos) = tokens.iter().position(|&t| t == "account") {
                raw_token = tokens.get(pos + 1).copied();
            }
            if raw_token.is_none() {
                raw_token = tokens.iter().rev().find(|&&t| !t.starts_with('(')).copied();
            }
            raw_token.map(|token| {
                let mut clean = token;
                if let Some(idx) = clean.find('(') {
                    clean = &clean[..idx];
                }
                clean
                    .trim_matches(|character: char| {
                        !character.is_alphanumeric() && character != '-' && character != '_'
                    })
                    .to_string()
            })
        })
        .filter(|name| !name.is_empty());

    Some(GhCliStatus {
        logged_in: true,
        username,
    })
}

pub fn detect_ssh_keys() -> Vec<SshKeyInfo> {
    let Some(home_directory) = dirs_home() else {
        return Vec::new();
    };

    let ssh_directory = home_directory.join(".ssh");
    if !ssh_directory.is_dir() {
        return Vec::new();
    }

    let key_file_prefixes = [
        "id_rsa",
        "id_ed25519",
        "id_ecdsa",
        "id_ecdsa_sk",
        "id_ed25519_sk",
        "id_dsa",
    ];

    let mut discovered_keys = Vec::new();

    for prefix in &key_file_prefixes {
        let public_key_path = ssh_directory.join(format!("{prefix}.pub"));
        let private_key_path = ssh_directory.join(prefix);

        if public_key_path.exists() || private_key_path.exists() {
            let key_type = match *prefix {
                name if name.contains("ed25519_sk") => "ed25519-sk",
                name if name.contains("ecdsa_sk") => "ecdsa-sk",
                name if name.contains("ed25519") => "ed25519",
                name if name.contains("ecdsa") => "ecdsa",
                name if name.contains("rsa") => "rsa",
                name if name.contains("dsa") => "dsa",
                _ => "unknown",
            };

            let display_path = if public_key_path.exists() {
                public_key_path
            } else {
                private_key_path
            };

            discovered_keys.push(SshKeyInfo {
                path: display_path.to_string_lossy().to_string(),
                key_type: key_type.to_string(),
            });
        }
    }

    discovered_keys
}

pub fn detect_gpg_keys() -> Vec<GpgKeyInfo> {
    let output = match Command::new("gpg")
        .args(["--list-secret-keys", "--keyid-format", "long"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let raw_output = String::from_utf8_lossy(&output.stdout);
    parse_gpg_output(&raw_output)
}

fn parse_gpg_output(output: &str) -> Vec<GpgKeyInfo> {
    let mut discovered_keys = Vec::new();
    let mut current_key_id: Option<String> = None;

    for line in output.lines() {
        // Lines like "sec   rsa4096/ABCDEF1234567890 2024-01-01 [SC]"
        if line.starts_with("sec") || line.starts_with("ssb") {
            current_key_id = line
                .split('/')
                .nth(1)
                .and_then(|segment| segment.split_whitespace().next())
                .map(|key_id| key_id.to_string());
        }

        // UID lines like "uid           [ultimate] User Name <email@example.com>"
        if line.contains("uid") && !line.starts_with("sec") && !line.starts_with("ssb") {
            if let Some(ref key_id) = current_key_id {
                let raw_uid = line.trim().strip_prefix("uid").unwrap_or(line).trim();
                let uid_text = if raw_uid.starts_with('[') {
                    if let Some(pos) = raw_uid.find(']') {
                        &raw_uid[pos + 1..]
                    } else {
                        raw_uid
                    }
                } else {
                    raw_uid
                };

                // Fall back to the full trimmed uid portion if bracket-stripping consumed everything
                let uid_display = if uid_text.trim().is_empty() {
                    line.trim()
                        .strip_prefix("uid")
                        .unwrap_or(line)
                        .trim()
                        .to_string()
                } else {
                    uid_text.trim().to_string()
                };

                if !uid_display.is_empty() {
                    discovered_keys.push(GpgKeyInfo {
                        key_id: key_id.clone(),
                        uid: uid_display,
                    });
                    current_key_id = None;
                }
            }
        }
    }

    discovered_keys
}

fn dirs_home() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home));
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            if !user_profile.is_empty() {
                return Some(PathBuf::from(user_profile));
            }
        }
        if let (Some(drive), Some(path)) =
            (std::env::var_os("HOMEDRIVE"), std::env::var_os("HOMEPATH"))
        {
            if !drive.is_empty() && !path.is_empty() {
                let mut base = PathBuf::from(drive);
                base.push(path);
                return Some(base);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_identity_default_has_no_values() {
        let identity = GitIdentity {
            name: None,
            email: None,
            signing_key: None,
            gpg_sign_commits: false,
            ssh_keys: Vec::new(),
            gpg_keys: Vec::new(),
        };
        assert!(identity.name.is_none());
        assert!(identity.email.is_none());
        assert!(!identity.gpg_sign_commits);
        assert!(identity.ssh_keys.is_empty());
        assert!(identity.gpg_keys.is_empty());
    }

    #[test]
    fn git_identity_serialization_roundtrip() {
        let identity = GitIdentity {
            name: Some("Test User".into()),
            email: Some("test@example.com".into()),
            signing_key: Some("ABCDEF".into()),
            gpg_sign_commits: true,
            ssh_keys: vec![SshKeyInfo {
                path: "/home/test/.ssh/id_ed25519.pub".into(),
                key_type: "ed25519".into(),
            }],
            gpg_keys: vec![GpgKeyInfo {
                key_id: "ABCDEF1234567890".into(),
                uid: "Test User <test@example.com>".into(),
            }],
        };
        let serialized = serde_json::to_string(&identity).expect("serialization should succeed");
        let deserialized: GitIdentity =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(identity, deserialized);
    }

    #[test]
    fn gpg_output_parsing_extracts_keys() {
        let gpg_output = "\
sec   rsa4096/ABCDEF1234567890 2024-01-01 [SC]
      FINGERPRINT1234567890FINGERPRINT12345678
uid           [ultimate] Test User <test@example.com>
ssb   rsa4096/1234567890ABCDEF 2024-01-01 [E]
";
        let keys = parse_gpg_output(gpg_output);
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].key_id, "ABCDEF1234567890");
    }

    #[test]
    fn gpg_output_parsing_handles_empty_input() {
        let keys = parse_gpg_output("");
        assert!(keys.is_empty());
    }

    #[test]
    fn gh_cli_status_serialization_roundtrip() {
        let status = GhCliStatus {
            logged_in: true,
            username: Some("testuser".into()),
        };
        let serialized = serde_json::to_string(&status).expect("serialization should succeed");
        let deserialized: GhCliStatus =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(status, deserialized);
    }
}
