use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;

pub const GITHUB_USER: &str = "Tushar27-git";
pub const GITHUB_REPO: &str = "Grace";
pub const GITHUB_PROFILE_URL: &str = "https://github.com/Tushar27-git";
pub const GITHUB_REPO_URL: &str = "https://github.com/Tushar27-git/Grace";
pub const GITHUB_RELEASES_URL: &str = "https://github.com/Tushar27-git/Grace/releases";

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    #[allow(dead_code)]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub html_url: String,
    pub body: Option<String>,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate {
        current_version: String,
        checked_time: String,
    },
    UpdateAvailable {
        current_version: String,
        latest_version: String,
        release_url: String,
        release_name: String,
        release_notes: String,
        download_url: Option<String>,
        asset_name: Option<String>,
    },
    Downloading {
        progress_msg: String,
    },
    UpdatedRestartRequired {
        version: String,
        message: String,
    },
    Error(String),
}

#[derive(Clone)]
pub struct UpdaterState {
    pub status: Arc<Mutex<UpdateStatus>>,
}

impl Default for UpdaterState {
    fn default() -> Self {
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
        }
    }
}

impl UpdaterState {
    pub fn check_for_updates(&self) {
        let status_arc = Arc::clone(&self.status);
        *status_arc.lock().unwrap() = UpdateStatus::Checking;

        thread::spawn(move || {
            let current_ver = env!("CARGO_PKG_VERSION");
            let releases_api_url = format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                GITHUB_USER, GITHUB_REPO
            );

            let output = std::process::Command::new("curl.exe")
                .args([
                    "-s",
                    "-H",
                    "User-Agent: LogicLab-Grace-App",
                    "-H",
                    "Accept: application/vnd.github.v3+json",
                    &releases_api_url,
                ])
                .output();

            let res = match output {
                Ok(out) if out.status.success() => {
                    let text = String::from_utf8_lossy(&out.stdout);
                    if text.contains("\"message\":\"Not Found\"") {
                        // Fallback: query all releases list if latest not marked
                        let all_api_url = format!(
                            "https://api.github.com/repos/{}/{}/releases",
                            GITHUB_USER, GITHUB_REPO
                        );
                        let all_out = std::process::Command::new("curl.exe")
                            .args([
                                "-s",
                                "-H",
                                "User-Agent: LogicLab-Grace-App",
                                "-H",
                                "Accept: application/vnd.github.v3+json",
                                &all_api_url,
                            ])
                            .output();

                        if let Ok(all_res) = all_out {
                            let all_text = String::from_utf8_lossy(&all_res.stdout);
                            if let Ok(releases) =
                                serde_json::from_str::<Vec<GitHubRelease>>(&all_text)
                            {
                                if let Some(first) = releases.into_iter().next() {
                                    Ok(first)
                                } else {
                                    Err("No releases published yet on GitHub repository.".to_string())
                                }
                            } else {
                                Err("No releases published yet on GitHub repository.".to_string())
                            }
                        } else {
                            Err("No releases published yet on GitHub repository.".to_string())
                        }
                    } else {
                        serde_json::from_str::<GitHubRelease>(&text)
                            .map_err(|e| format!("Failed to parse release information: {}", e))
                    }
                }
                Ok(out) => Err(format!(
                    "GitHub API request failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                )),
                Err(e) => Err(format!("Could not connect to GitHub: {}", e)),
            };

            let mut lock = status_arc.lock().unwrap();
            match res {
                Ok(release) => {
                    let is_newer = is_version_newer(&release.tag_name, current_ver);
                    if is_newer {
                        // Look for .exe or .zip release asset
                        let asset = release.assets.iter().find(|a| {
                            let lower = a.name.to_lowercase();
                            lower.ends_with(".exe") || lower.ends_with(".zip")
                        });

                        *lock = UpdateStatus::UpdateAvailable {
                            current_version: current_ver.to_string(),
                            latest_version: release.tag_name.clone(),
                            release_url: release.html_url.clone(),
                            release_name: release.name.unwrap_or_else(|| release.tag_name.clone()),
                            release_notes: release.body.unwrap_or_default(),
                            download_url: asset.map(|a| a.browser_download_url.clone()),
                            asset_name: asset.map(|a| a.name.clone()),
                        };
                    } else {
                        *lock = UpdateStatus::UpToDate {
                            current_version: format!("v{}", current_ver),
                            checked_time: "Just now".to_string(),
                        };
                    }
                }
                Err(err_msg) => {
                    if err_msg.contains("No releases published") {
                        *lock = UpdateStatus::UpToDate {
                            current_version: format!("v{} (Latest Build)", current_ver),
                            checked_time: "GitHub checked just now".to_string(),
                        };
                    } else {
                        *lock = UpdateStatus::Error(err_msg);
                    }
                }
            }
        });
    }

    pub fn start_auto_update(&self, download_url: String, version: String) {
        let status_arc = Arc::clone(&self.status);
        *status_arc.lock().unwrap() = UpdateStatus::Downloading {
            progress_msg: format!("Downloading {} update from GitHub...", version),
        };

        thread::spawn(move || {
            let current_exe = match std::env::current_exe() {
                Ok(p) => p,
                Err(e) => {
                    *status_arc.lock().unwrap() = UpdateStatus::Error(format!(
                        "Cannot identify running executable location: {}",
                        e
                    ));
                    return;
                }
            };

            let exe_dir = match current_exe.parent() {
                Some(p) => p,
                None => {
                    *status_arc.lock().unwrap() =
                        UpdateStatus::Error("Cannot determine executable directory".into());
                    return;
                }
            };

            let temp_download = exe_dir.join("logic-app-update-temp.exe");
            let backup_old = exe_dir.join("logic-app.old");

            let download_res = std::process::Command::new("curl.exe")
                .args([
                    "-L",
                    "-s",
                    "-H",
                    "User-Agent: LogicLab-Grace-App",
                    "-o",
                    temp_download.to_string_lossy().as_ref(),
                    &download_url,
                ])
                .output();

            match download_res {
                Ok(out) if out.status.success() => {
                    let size = std::fs::metadata(&temp_download)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    if size < 50_000 {
                        let _ = std::fs::remove_file(&temp_download);
                        *status_arc.lock().unwrap() = UpdateStatus::Error(
                            "Downloaded asset is too small or invalid. Please check GitHub release assets."
                                .into(),
                        );
                        return;
                    }

                    // Windows in-place replace: rename running exe to .old, move temp to exe
                    let _ = std::fs::remove_file(&backup_old);

                    if let Err(e) = std::fs::rename(&current_exe, &backup_old) {
                        let _ = std::fs::remove_file(&temp_download);
                        *status_arc.lock().unwrap() = UpdateStatus::Error(format!(
                            "Failed to rename running executable: {}. Try running with Administrator privileges.",
                            e
                        ));
                        return;
                    }

                    if let Err(e) = std::fs::rename(&temp_download, &current_exe) {
                        let _ = std::fs::rename(&backup_old, &current_exe);
                        let _ = std::fs::remove_file(&temp_download);
                        *status_arc.lock().unwrap() = UpdateStatus::Error(format!(
                            "Failed to write new version executable: {}",
                            e
                        ));
                        return;
                    }

                    *status_arc.lock().unwrap() = UpdateStatus::UpdatedRestartRequired {
                        version,
                        message: "Update successfully installed!".into(),
                    };
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    *status_arc.lock().unwrap() = UpdateStatus::Error(format!(
                        "Download failed: {}. You can download manually from GitHub Releases.",
                        err
                    ));
                }
                Err(e) => {
                    *status_arc.lock().unwrap() = UpdateStatus::Error(format!(
                        "Failed to execute download command: {}",
                        e
                    ));
                }
            }
        });
    }
}

pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .spawn();
    }
}

pub fn restart_application() {
    if let Ok(current_exe) = std::env::current_exe() {
        let _ = std::process::Command::new(current_exe).spawn();
        std::process::exit(0);
    }
}

pub fn clean_old_updates() {
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let old_file = dir.join("logic-app.old");
            if old_file.exists() {
                let _ = std::fs::remove_file(old_file);
            }
            let temp_file = dir.join("logic-app-update-temp.exe");
            if temp_file.exists() {
                let _ = std::fs::remove_file(temp_file);
            }
        }
    }
}

pub fn is_version_newer(remote_tag: &str, current_ver: &str) -> bool {
    let clean_remote = remote_tag.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    let clean_current = current_ver.trim().trim_start_matches(|c| c == 'v' || c == 'V');

    fn parse_parts(s: &str) -> Vec<u64> {
        s.split('.')
            .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
            .filter_map(|p| p.parse::<u64>().ok())
            .collect()
    }

    let r_parts = parse_parts(clean_remote);
    let c_parts = parse_parts(clean_current);

    if !r_parts.is_empty() && !c_parts.is_empty() {
        for (r, c) in r_parts.iter().zip(c_parts.iter()) {
            if r > c {
                return true;
            }
            if r < c {
                return false;
            }
        }
        r_parts.len() > c_parts.len()
    } else {
        clean_remote != clean_current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_version_newer("v0.2.0", "0.1.0"));
        assert!(is_version_newer("v0.1.1", "0.1.0"));
        assert!(is_version_newer("1.0.0", "0.1.0"));
        assert!(!is_version_newer("v0.1.0", "0.1.0"));
        assert!(!is_version_newer("0.0.9", "0.1.0"));
    }
}
