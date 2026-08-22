//! Stamp the checkout revision into the binary's `--version`.
//!
//! Half of the library's build script. The other half compiles file-type rules, which
//! belong to `fdu` and stay there; a command line has no business owning them.
//!
//! Development builds carry the Git revision and a dirty marker, so a binary built from a
//! checkout never impersonates the published release.

use std::process::Command;

fn main() {
    emit_version();
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_string())
}

fn emit_version() {
    let semver = std::env::var("CARGO_PKG_VERSION").expect("cargo sets CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed=FDU_RELEASE_TAG");
    if let Ok(tag) = std::env::var("FDU_RELEASE_TAG") {
        let expected = format!("v{semver}");
        assert_eq!(tag, expected, "FDU_RELEASE_TAG must exactly match the Cargo package version");
        println!("cargo:rustc-env=FDU_BUILD_VERSION={semver}");
        return;
    }
    let version = match git(&["rev-parse", "--short=9", "HEAD"]) {
        Some(revision) => {
            // This preserves Cargo's default whole-package dirty tracking after adding
            // the narrower manifest and Git rerun directives below.
            println!("cargo:rerun-if-changed=.");
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
            let expected_tag = format!("v{semver}");
            let exact_release = !dirty
                && git(&["tag", "--points-at", "HEAD"])
                    .is_some_and(|tags| tags.lines().any(|tag| tag == expected_tag));
            if exact_release {
                semver
            } else {
                format!("{semver}-dev+g{revision}{}", if dirty { ".dirty" } else { "" })
            }
        }
        None => semver,
    };
    println!("cargo:rustc-env=FDU_BUILD_VERSION={version}");
}
