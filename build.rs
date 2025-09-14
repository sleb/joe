use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Tell cargo to rerun this build script if these files change
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/main");
    println!("cargo:rerun-if-changed=.git/refs/tags");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Get version information (git-based or fallback)
    let version_info = get_version_info();

    // Set environment variables that our code can access
    println!("cargo:rustc-env=BUILD_VERSION={}", version_info.version);
    println!(
        "cargo:rustc-env=BUILD_VERSION_FULL={}",
        version_info.full_version
    );
    println!("cargo:rustc-env=BUILD_GIT_HASH={}", version_info.git_hash);
    println!(
        "cargo:rustc-env=BUILD_GIT_BRANCH={}",
        version_info.git_branch
    );
    println!("cargo:rustc-env=BUILD_GIT_DIRTY={}", version_info.git_dirty);
    println!(
        "cargo:rustc-env=BUILD_IS_RELEASE={}",
        version_info.is_release
    );
    println!(
        "cargo:rustc-env=BUILD_COMMITS_SINCE_TAG={}",
        version_info.commits_since_tag
    );

    if let Some(tag) = &version_info.base_tag {
        println!("cargo:rustc-env=BUILD_BASE_TAG={}", tag);
    } else {
        println!("cargo:rustc-env=BUILD_BASE_TAG=");
    }

    // Add build timestamp
    let build_time = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_time);

    // Add target and profile information
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={}", target);
    println!("cargo:rustc-env=BUILD_PROFILE={}", profile);
}

#[derive(Debug)]
struct VersionInfo {
    /// Clean version string (e.g., "1.0.0" or "1.0.0-dev.5")
    version: String,
    /// Full version with git info (e.g., "1.0.0-5-g1a2b3c4-dirty")
    full_version: String,
    /// Short git hash
    git_hash: String,
    /// Current branch
    git_branch: String,
    /// Whether working directory is dirty
    git_dirty: bool,
    /// Whether this is a release build (on exact tag)
    is_release: bool,
    /// Number of commits since the base tag
    commits_since_tag: u32,
    /// The base tag this version is derived from
    base_tag: Option<String>,
}

fn get_version_info() -> VersionInfo {
    // Try git-based versioning first
    match get_git_version_info() {
        Ok(git_info) => git_info,
        Err(git_error) => {
            // Log the git error as a warning but continue
            println!("cargo:warning=Git versioning not available: {}", git_error);
            println!("cargo:warning=Falling back to Cargo.toml version");

            // Fall back to Cargo.toml version
            get_fallback_version_info()
        }
    }
}

fn get_git_version_info() -> Result<VersionInfo, String> {
    // Check if we're in a git repository
    if !Path::new(".git").exists() {
        return Err("Not in a git repository".to_string());
    }

    // Check if any tags exist
    let has_tags = run_git_command(&["tag", "-l"])
        .map(|output| !output.trim().is_empty())
        .unwrap_or(false);

    if !has_tags {
        return Err("No git tags found".to_string());
    }

    // Use git describe to get version information
    let describe_output = run_git_command(&["describe", "--tags", "--long", "--dirty", "--always"])
        .ok_or_else(|| "Failed to run 'git describe'".to_string())?;

    // Parse git describe output
    let (version, full_version, is_release, commits_since_tag, base_tag) =
        parse_git_describe(&describe_output)?;

    // Get additional git information
    let git_hash = run_git_command(&["rev-parse", "--short", "HEAD"])
        .ok_or_else(|| "Failed to get git hash".to_string())?;

    let git_branch = run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|| "unknown".to_string());

    let git_dirty = describe_output.contains("-dirty");

    Ok(VersionInfo {
        version,
        full_version,
        git_hash,
        git_branch,
        git_dirty,
        is_release,
        commits_since_tag,
        base_tag,
    })
}

fn get_fallback_version_info() -> VersionInfo {
    // Get version from Cargo.toml
    let cargo_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());

    // Create a fallback version info with minimal information
    VersionInfo {
        version: cargo_version.clone(),
        full_version: format!("{}-fallback", cargo_version),
        git_hash: "unknown".to_string(),
        git_branch: "unknown".to_string(),
        git_dirty: false,
        is_release: true, // Assume fallback builds are releases
        commits_since_tag: 0,
        base_tag: Some(format!("v{}", cargo_version)),
    }
}

fn parse_git_describe(
    describe: &str,
) -> Result<(String, String, bool, u32, Option<String>), String> {
    // git describe formats:
    // v1.0.0                     (exact tag)
    // v1.0.0-5-g1a2b3c4          (5 commits after v1.0.0)
    // v1.0.0-5-g1a2b3c4-dirty    (5 commits after v1.0.0, working directory dirty)

    let full_version = describe.to_string();
    let clean_describe = describe.replace("-dirty", "");

    // Check if this is an exact tag (no commits after)
    let parts: Vec<&str> = clean_describe.split('-').collect();

    if parts.len() == 1 {
        // Exact tag match (e.g., "v1.0.0")
        let tag = parts[0];
        if !tag.starts_with('v') {
            return Err(format!(
                "Git tag '{}' doesn't follow semantic versioning format (should start with 'v').\n\
                 Example: git tag v1.0.0",
                tag
            ));
        }

        let version = tag.strip_prefix('v').unwrap().to_string();
        Ok((version, full_version, true, 0, Some(tag.to_string())))
    } else if parts.len() >= 3 {
        // Check if this is actually an exact tag (0 commits since tag)
        let commits_str = parts[1];
        if commits_str == "0" {
            // This is an exact tag match, just with --long format
            let tag = parts[0];
            if !tag.starts_with('v') {
                return Err(format!(
                    "Git tag '{}' doesn't follow semantic versioning format (should start with 'v').\n\
                     Example: git tag v1.0.0",
                    tag
                ));
            }
            let version = tag.strip_prefix('v').unwrap().to_string();
            Ok((version, full_version, true, 0, Some(tag.to_string())))
        } else {
            // Development version (e.g., "v1.0.0-5-g1a2b3c4")
            let tag = parts[0];

            if !tag.starts_with('v') {
                return Err(format!(
                    "Git tag '{}' doesn't follow semantic versioning format (should start with 'v').\n\
                     Example: git tag v1.0.0",
                    tag
                ));
            }

            let commits_since_tag: u32 = commits_str.parse().map_err(|_| {
                format!(
                    "Invalid commit count '{}' in git describe output",
                    commits_str
                )
            })?;

            let base_version = tag.strip_prefix('v').unwrap();
            let dev_version = if describe.contains("-dirty") {
                format!("{}-dev.{}.dirty", base_version, commits_since_tag)
            } else {
                format!("{}-dev.{}", base_version, commits_since_tag)
            };

            Ok((
                dev_version,
                full_version,
                false,
                commits_since_tag,
                Some(tag.to_string()),
            ))
        }
    } else {
        Err(format!(
            "Unexpected git describe format: '{}'\n\
             Expected format like 'v1.0.0' or 'v1.0.0-5-g1a2b3c4'",
            describe
        ))
    }
}

fn run_git_command(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
