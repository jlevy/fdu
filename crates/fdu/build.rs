//! Stamp the checkout revision into the binary's `--version`.
//!
//! Half of what was one build script. The other half compiles the file-type rules,
//! which belong to `fdu-core` and stay there; a command line has no business owning them.
//!
//! Development builds carry the Git revision and a dirty marker, so a binary built from a
//! checkout never impersonates the published release.

use std::process::Command;

fn main() {
    emit_version();
}

// BEGIN shared version stamp -- keep byte-identical with the other crate's build.rs.
//
// Duplicated rather than shared on purpose. A cross-crate `include!` would name a file
// outside the including crate's package, so it would not be in the published `.crate`
// and the packaged source would not compile -- the same failure that shipped `fdu`
// without its build script once. A third crate to hold twenty lines is worse than two
// copies. `tests/release/test_metadata.py` fails if the copies drift, which is the risk
// that actually matters: `fdu --version` and the performance probe's `--version`
// describe the same checkout, and the perf harness matches the probe's git revision
// against the source it claims to measure.
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
// END shared version stamp.
