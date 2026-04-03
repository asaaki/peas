use crate::global_config::GlobalPeasConfig;
use chrono::{DateTime, Duration, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::thread::{self, JoinHandle};

/// Result of an update check.
#[derive(Debug, Clone)]
pub enum UpdateCheckOutcome {
    /// A newer version is available.
    UpdateAvailable(String),
    /// peas is already up to date.
    UpToDate,
    /// The network check failed.
    CheckFailed,
    /// Update checks are disabled, or the cache is still valid with no new version.
    Skipped,
}

/// On-disk cache format.
#[derive(Debug, Serialize, Deserialize)]
struct UpdateCache {
    last_checked: DateTime<Utc>,
    check_succeeded: bool,
    latest_version: String,
    retry_interval_hours: i64,
}

/// Spawn a background thread that performs the update check.
///
/// Returns immediately; the caller can `.join()` the handle when it wants the result.
pub fn spawn_update_check(global_config: &GlobalPeasConfig) -> JoinHandle<UpdateCheckOutcome> {
    let enabled = global_config.updates.enabled;
    thread::spawn(move || {
        if !enabled {
            return UpdateCheckOutcome::Skipped;
        }
        run_update_check()
    })
}

fn cache_path() -> Option<std::path::PathBuf> {
    ProjectDirs::from("", "", "peas").map(|d| d.cache_dir().join("update-check.json"))
}

fn read_cache() -> Option<UpdateCache> {
    let path = cache_path()?;
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_cache(cache: &UpdateCache) {
    let Some(path) = cache_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = std::fs::write(path, json);
    }
}

fn next_retry_interval(current: i64) -> i64 {
    // Step down: 24 → 12 → 6 → 3 → 1, then hold at 1
    match current {
        h if h > 12 => 12,
        h if h > 6 => 6,
        h if h > 3 => 3,
        h if h > 1 => 1,
        _ => 1,
    }
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> Vec<u32> {
        s.trim_start_matches('v')
            .split('.')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };
    let va = parse(a);
    let vb = parse(b);
    let len = va.len().max(vb.len());
    for i in 0..len {
        let a_part = va.get(i).copied().unwrap_or(0);
        let b_part = vb.get(i).copied().unwrap_or(0);
        match a_part.cmp(&b_part) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

fn fetch_latest_version() -> Option<String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let user_agent = format!("peas/{}", current_version);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;

    let resp = client
        .get("https://api.github.com/repos/asaaki/peas/releases/latest")
        .header("User-Agent", user_agent)
        .send()
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
    }

    let release: Release = resp.json().ok()?;
    Some(release.tag_name)
}

fn run_update_check() -> UpdateCheckOutcome {
    let now = Utc::now();
    let current_version = env!("CARGO_PKG_VERSION");

    // Check cache
    let cached_retry_interval = if let Some(cache) = read_cache() {
        let age = now - cache.last_checked;
        let interval = Duration::hours(cache.retry_interval_hours);
        if age < interval {
            // Cache still valid
            if cache.check_succeeded {
                let latest = cache.latest_version.trim_start_matches('v');
                if compare_versions(latest, current_version) == std::cmp::Ordering::Greater {
                    return UpdateCheckOutcome::UpdateAvailable(latest.to_string());
                }
            }
            return UpdateCheckOutcome::Skipped;
        }
        cache.retry_interval_hours
    } else {
        24
    };

    // Cache stale or missing — fetch
    match fetch_latest_version() {
        Some(tag) => {
            let latest = tag.trim_start_matches('v').to_string();
            let cache = UpdateCache {
                last_checked: now,
                check_succeeded: true,
                latest_version: latest.clone(),
                retry_interval_hours: 24,
            };
            write_cache(&cache);
            if compare_versions(&latest, current_version) == std::cmp::Ordering::Greater {
                UpdateCheckOutcome::UpdateAvailable(latest)
            } else {
                UpdateCheckOutcome::UpToDate
            }
        }
        None => {
            // Failure: step down retry interval
            let cache = UpdateCache {
                last_checked: now,
                check_succeeded: false,
                latest_version: String::new(),
                retry_interval_hours: next_retry_interval(cached_retry_interval),
            };
            write_cache(&cache);
            UpdateCheckOutcome::CheckFailed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert_eq!(
            compare_versions("0.2.2", "0.2.1"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("0.2.1", "0.2.1"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(compare_versions("0.2.0", "0.2.1"), std::cmp::Ordering::Less);
        assert_eq!(
            compare_versions("v0.2.2", "0.2.1"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.0.0", "0.9.9"),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_retry_interval_stepdown() {
        assert_eq!(next_retry_interval(24), 12);
        assert_eq!(next_retry_interval(12), 6);
        assert_eq!(next_retry_interval(6), 3);
        assert_eq!(next_retry_interval(3), 1);
        assert_eq!(next_retry_interval(1), 1);
    }
}
