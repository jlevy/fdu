//! Research-only token probe. It deliberately does not model comments or literals.
//! Run with: rustc -O explorations/branch_points_probe.rs -o /external/scratch/probe
//!             && /external/scratch/probe path/to/source.rs ...

use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

fn branch_word(word: &[u8], rust: bool) -> bool {
    if rust {
        matches!(word, b"if" | b"for" | b"while" | b"match")
    } else {
        matches!(word, b"if" | b"for" | b"while" | b"switch" | b"catch")
    }
}

fn scan(bytes: &[u8], rust: bool, branches: bool) -> (u64, u64) {
    let mut lines = 0_u64;
    let mut points = 0_u64;
    let mut word_start = None;
    for (offset, &byte) in bytes.iter().enumerate() {
        if byte == b'\n' {
            lines += 1;
        }
        if branches {
            if byte.is_ascii_alphabetic() || byte == b'_' {
                word_start.get_or_insert(offset);
            } else if let Some(start) = word_start.take() {
                points += u64::from(branch_word(&bytes[start..offset], rust));
            }
        }
    }
    if branches {
        if let Some(start) = word_start {
            points += u64::from(branch_word(&bytes[start..], rust));
        }
    }
    (lines, points)
}

fn timed(inputs: &[(Vec<u8>, bool)], branches: bool) -> (u128, u64) {
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..200 {
        for (bytes, rust) in inputs {
            let (lines, points) = scan(black_box(bytes), *rust, branches);
            checksum = checksum.wrapping_add(lines).wrapping_add(points);
        }
    }
    (started.elapsed().as_nanos(), checksum)
}

fn main() {
    let inputs: Vec<_> = std::env::args()
        .skip(1)
        .filter_map(|argument| {
            let path = Path::new(&argument);
            let rust = match path.extension()?.to_str()? {
                "rs" => true,
                "js" => false,
                _ => return None,
            };
            Some((std::fs::read(path).expect("source file is readable"), rust))
        })
        .collect();
    let bytes: usize = inputs.iter().map(|(source, _)| source.len()).sum();
    black_box(timed(&inputs, false));
    black_box(timed(&inputs, true));
    let (baseline_ns, baseline_checksum) = timed(&inputs, false);
    let (probe_ns, probe_checksum) = timed(&inputs, true);
    println!(
        "files={},bytes={},passes=200,baseline_ns={baseline_ns},probe_ns={probe_ns},baseline_checksum={baseline_checksum},probe_checksum={probe_checksum}",
        inputs.len(),
        bytes
    );
}
