use std::env;

use std::path::Path;
use std::process::Command;

fn main() {
    // Tell cargo to rerun this build script if these files change
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/main");

    // Get version from Cargo.toml
    let cargo_version = env::var("CARGO_PKG_VERSION").unwrap();

    // Try to get git information
    let git_info = get_git_info();

    // Set environment variables that our code can access
    println!("cargo:rustc-env=BUILD_CARGO_VERSION={}", cargo_version);

    match git_info {
        Some(info) => {
            println!("cargo:rustc-env=BUILD_GIT_HASH={}", info.hash);
            println!("cargo:rustc-env=BUILD_GIT_BRANCH={}", info.branch);
            println!("cargo:rustc-env=BUILD_GIT_DIRTY={}", info.dirty);

            if let Some(tag) = &info.latest_tag {
                println!("cargo:rustc-env=BUILD_GIT_TAG={}", tag);

                // Extract version from tag (remove 'v' prefix if present)
                let tag_version = tag.strip_prefix('v').unwrap_or(tag);
                println!("cargo:rustc-env=BUILD_TAG_VERSION={}", tag_version);

                // Check version consistency
                if cargo_version != tag_version {
                    println!("cargo:warning=Version mismatch detected:");
                    println!("cargo:warning=  Cargo.toml: {}", cargo_version);
                    println!("cargo:warning=  Latest tag: {}", tag_version);
                    println!("cargo:warning=Consider updating version or creating new tag");
                }
            } else {
                println!("cargo:rustc-env=BUILD_GIT_TAG=");
                println!("cargo:rustc-env=BUILD_TAG_VERSION=");
                println!(
                    "cargo:warning=No git tags found. Consider creating tag: git tag v{}",
                    cargo_version
                );
            }
        }
        None => {
            // Not in a git repository or git not available
            println!("cargo:rustc-env=BUILD_GIT_HASH=unknown");
            println!("cargo:rustc-env=BUILD_GIT_BRANCH=unknown");
            println!("cargo:rustc-env=BUILD_GIT_DIRTY=false");
            println!("cargo:rustc-env=BUILD_GIT_TAG=");
            println!("cargo:rustc-env=BUILD_TAG_VERSION=");
        }
    }

    // Add build timestamp
    let build_time = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_time);

    // Add target information
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={}", target);

    // Add profile information
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={}", profile);
}

#[derive(Debug)]
struct GitInfo {
    hash: String,
    branch: String,
    dirty: bool,
    latest_tag: Option<String>,
}

fn get_git_info() -> Option<GitInfo> {
    // Check if we're in a git repository
    if !Path::new(".git").exists() {
        return None;
    }

    // Get git hash
    let hash = run_git_command(&["rev-parse", "--short", "HEAD"])?;

    // Get current branch
    let branch = run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|| "unknown".to_string());

    // Check if working directory is dirty
    let status_output = run_git_command(&["status", "--porcelain"])?;
    let dirty = !status_output.trim().is_empty();

    // Get latest tag
    let latest_tag = run_git_command(&["describe", "--tags", "--abbrev=0"]);

    Some(GitInfo {
        hash,
        branch,
        dirty,
        latest_tag,
    })
}

fn run_git_command(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}
