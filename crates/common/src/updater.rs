//! Self-update logic: check GitHub releases and update binaries in-place.

use anyhow::{Context, Result};
use self_update::backends::github;
use self_update::cargo_crate_version;

pub const REPO_OWNER: &str = "pedrokarim";
pub const REPO_NAME: &str = "my-power-toys";

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
}

/// Check GitHub for a newer release.
/// Returns `Some(UpdateInfo)` if a newer version exists, `None` if up-to-date.
pub fn check_for_update() -> Result<Option<UpdateInfo>> {
    let current = cargo_crate_version!();

    let releases = github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()
        .context("failed to build release list")?
        .fetch()
        .context("failed to fetch releases from GitHub")?;

    let latest = match releases.first() {
        Some(release) => release,
        None => return Ok(None),
    };

    let latest_version = latest.version.trim_start_matches('v');

    if version_is_newer(current, latest_version)? {
        Ok(Some(UpdateInfo {
            current_version: current.to_string(),
            latest_version: latest_version.to_string(),
            release_notes: latest.body.clone().unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

/// Download and install the latest release, replacing the current binary.
/// `bin_name` should be the name of the binary to replace (e.g. "mpt-ctl").
/// Returns the new version string on success.
pub fn perform_update(bin_name: &str) -> Result<String> {
    let status = github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(bin_name)
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()
        .context("failed to build updater")?
        .update()
        .context("failed to perform update")?;

    Ok(status.version().to_string())
}

/// Compare two semver strings. Returns true if `latest` is newer than `current`.
fn version_is_newer(current: &str, latest: &str) -> Result<bool> {
    let current = semver_parse(current)?;
    let latest = semver_parse(latest)?;
    Ok(latest > current)
}

/// Parse a version string into (major, minor, patch) for comparison.
fn semver_parse(v: &str) -> Result<(u64, u64, u64)> {
    let v = v.trim_start_matches('v');
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() < 3 {
        anyhow::bail!("invalid semver: {v}");
    }
    let major: u64 = parts[0].parse().context("invalid major version")?;
    let minor: u64 = parts[1].parse().context("invalid minor version")?;
    // Handle pre-release suffixes like "1.0.0-beta"
    let patch_str = parts[2].split('-').next().unwrap_or(parts[2]);
    let patch: u64 = patch_str.parse().context("invalid patch version")?;
    Ok((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_newer() {
        assert!(version_is_newer("0.1.0", "0.2.0").unwrap());
        assert!(version_is_newer("0.1.0", "1.0.0").unwrap());
        assert!(version_is_newer("0.1.0", "0.1.1").unwrap());
        assert!(!version_is_newer("0.2.0", "0.1.0").unwrap());
        assert!(!version_is_newer("0.1.0", "0.1.0").unwrap());
    }

    #[test]
    fn test_version_with_v_prefix() {
        assert!(version_is_newer("v0.1.0", "v0.2.0").unwrap());
        assert!(version_is_newer("0.1.0", "v0.2.0").unwrap());
    }

    #[test]
    fn test_semver_parse() {
        assert_eq!(semver_parse("1.2.3").unwrap(), (1, 2, 3));
        assert_eq!(semver_parse("v1.2.3").unwrap(), (1, 2, 3));
        assert_eq!(semver_parse("0.1.0-beta").unwrap(), (0, 1, 0));
    }
}
