use std::path::{Path, PathBuf};

use anyhow::Context;
use chrono::{FixedOffset, Local, TimeZone};

fn main() -> anyhow::Result<()> {
    let manifiest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").context("Failed to get CARGO_MANIFEST_DIR")?,
    );
    // Go from {project}/src/backend/backend to {project}
    // Due to issues with docker this is not available yet...
    if std::env::var_os("SKIP_GIT_BUILD_INFO").is_none() {
        // Go from {project}/src/backend/backend to {project}
        let dir_with_git = go_many_parents(&manifiest_dir, 3);
        //println!("cargo:warning=Dir With Git : {:?}", dir_with_git);
        let repository = gix::discover(dir_with_git)?;
        set_commit_short(&repository)?;
        set_commit_time(&repository)?;
        set_branch(&repository)?;
    }

    set_build_time();
    Ok(())
}

fn go_many_parents(path: &Path, n: usize) -> PathBuf {
    let mut current = path.to_path_buf();
    for _ in 0..n {
        current.pop();
    }
    current
}
/// Access via env!("PROJECT_COMMIT_SHORT")
fn set_commit_short(repository: &gix::Repository) -> anyhow::Result<()> {
    let mut head = repository.head()?;

    let commit = head.peel_to_commit_in_place()?;
    let commit_short = commit.short_id()?;
    println!(
        "cargo:rustc-env=PROJECT_COMMIT_SHORT={}",
        commit_short
    );
    Ok(())
}
/// Access via env!("PROJECT_COMMIT_TIME")
fn set_commit_time(repository: &gix::Repository) -> anyhow::Result<()> {
    let mut head = repository.head()?;

    let commit = head.peel_to_commit_in_place()?;
    let time = commit.time()?;
    let offset = match time.sign {
        gix::date::time::Sign::Plus => FixedOffset::east_opt(time.offset.abs()),
        gix::date::time::Sign::Minus => FixedOffset::west_opt(time.offset.abs()),
    };
    let offset = offset.unwrap_or_else(|| *Local::now().offset());

    let datetime = offset
        .timestamp_millis_opt(time.seconds * 1000)
        .single()
        .context("Failed to convert commit time to datetime")?;

    println!(
        "cargo:rustc-env=PROJECT_COMMIT_TIME={}",
        datetime.to_rfc3339()
    );
    Ok(())
}
/// Access via env!("PROJECT_BRANCH")
fn set_branch(repository: &gix::Repository) -> anyhow::Result<()> {
    let head_name = repository.head_name()?;

    if let Some(head_name) = head_name {
        println!(
            "cargo:rustc-env=PROJECT_BRANCH={}",
            head_name.shorten()
        );
    }
    Ok(())
}

/// Access via env!("PROJECT_BUILD_TIME")
fn set_build_time() {
    let now = chrono::Local::now();
    println!("cargo:rustc-env=PROJECT_BUILD_TIME={}", now.to_rfc3339());
}
