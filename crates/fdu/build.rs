//! Stamp the checkout revision into the binary's `--version`.
//!
//! Development builds carry the Git revision and a dirty marker, so a binary built from a
//! checkout never impersonates the published release.
//!
//! This duplicates `emit_version` in `fdu-core/build.rs`, and the duplication is
//! load-bearing rather than an oversight (fdu-zuyq). `env!` expands in the crate that
//! reads it, so a consumer cannot see the value a dependency's build script emitted, and
//! the three ways out are all worse:
//!
//!   * A `pub const` in `fdu-core` that the command line displays. Tried; it fails
//!     `cargo package --locked -p fdu-core -p fdu`, because verifying `fdu` resolves
//!     `fdu-core` through the registry rather than the sibling being packaged. Trading a
//!     working release path for forty deduplicated lines is the wrong trade.
//!   * A shared file both scripts `include!`. The file would have to live outside both
//!     packages, and `cargo package` ships only what is inside one.
//!   * A shared build-dependency crate. That is a third crate, and this split is
//!     deliberately two.
//!
//! `fdu-core` keeps its own stamp because the perf probe is its example and the
//! provenance gate (`explorations/benchmarks/realtree/provenance.py`) asserts the revision appears in
//! the probe's `--version`. Neither copy is removable; keep them in step by hand.
//!
//! The file-type rules are the other half of `fdu-core`'s build script and stay there:
//! they are engine data, and a command line has no business owning them.

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
                if let Some(head_ref) = git(&["symbolic-ref", "-q", "HEAD"])
                    && let Some(common) = git(&["rev-parse", "--git-common-dir"])
                {
                    println!("cargo:rerun-if-changed={common}/{head_ref}");
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
