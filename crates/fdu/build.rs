//! Embeds the git revision into the version string, so `--version` distinguishes a
//! dev build from the published release it would otherwise impersonate.
//!
//! A build from a checkout reports `0.0.1-dev+g<rev>` (with a `.dirty` suffix when the
//! tree has uncommitted changes); a build without git metadata — a published crate, a
//! source tarball — reports the plain semver, which for those artifacts is the truth.
//! The `.dirty` marker describes the tree the binary was built from. It can only go
//! stale in one direction: editing a file that is not compiled into the crate (a doc,
//! a test fixture) will not trigger a rebuild, so the marker lags until the next one.

use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    Some(text.trim().to_string())
}

fn main() {
    let semver = std::env::var("CARGO_PKG_VERSION").expect("cargo sets CARGO_PKG_VERSION");

    let version = match git(&["rev-parse", "--short=9", "HEAD"]) {
        Some(rev) => {
            // Rebuild when the checked-out commit moves. HEAD is resolved through the
            // git dir rather than assumed at `../../.git`, because worktrees keep it
            // elsewhere; when HEAD is a symbolic ref, track the ref file too.
            if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
                println!("cargo:rerun-if-changed={git_dir}/HEAD");
                if let Some(head_ref) = git(&["symbolic-ref", "-q", "HEAD"]) {
                    if let Some(common) = git(&["rev-parse", "--git-common-dir"]) {
                        println!("cargo:rerun-if-changed={common}/{head_ref}");
                    }
                }
            }
            let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
                .is_some_and(|status| !status.is_empty());
            format!("{semver}-dev+g{rev}{}", if dirty { ".dirty" } else { "" })
        }
        None => semver,
    };

    println!("cargo:rustc-env=FDU_BUILD_VERSION={version}");
}
