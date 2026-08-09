//! The `fdu` command line.
//!
//! The CLI serves two audiences from one binary, and neither is an afterthought:
//!
//! - **Humans** get colored, width-aware tree output with percentage bars, sensible
//!   defaults, and `NO_COLOR` plus pipe detection so redirection degrades cleanly.
//! - **Agents** get `--help` as the complete source of truth, JSON whose schema is
//!   versioned with the tool, and meaningful exit codes — no pager, no prompts, no
//!   interactive surprises.

use std::ffi::OsStr;
use std::fmt::Write as _;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::index::{EntryId, Index};
use crate::{EntryKind, Freshness, OpenConfig, OpenPath, ScanConfig, default_cache_path, open};

/// The JSON schema identifier. Bump the version on any breaking shape change so an
/// agent can tell what it is parsing without guessing from the payload.
const JSON_SCHEMA: &str = "fdu.tree/2";

/// Successful command outcome. Partial results are rendered before the caller returns
/// exit status 2, so scripts can opt into them without confusing them with complete data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunOutcome {
    Complete,
    Partial,
}

/// Summarize directory trees: sizes, counts, recency, and file types, rolled up for
/// every directory at once.
#[derive(Parser, Debug)]
#[command(
    name = "fdu",
    version,
    about,
    long_about = None,
    after_help = "Result scope:\n  --depth and --number limit only the rendered view.\n  --max-depth limits the scan scope and retained index.\n\nExit status:\n  0  Complete result, or a partial result accepted with --allow-partial\n  1  Fatal filesystem or cache error\n  2  Partial result, or command-line usage error"
)]
// A command line is a flat bag of independent switches. Folding these into enums to
// satisfy the lint would obscure the one thing this struct exists to mirror: the flags a
// user actually types.
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Directory to summarize.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Directory levels below the root to render; does not limit scanning.
    #[arg(short, long, default_value_t = 2, value_name = "N")]
    pub depth: usize,

    /// Entries to render per directory, largest first; does not limit scanning.
    #[arg(short = 'n', long, default_value_t = 10, value_name = "N")]
    pub number: usize,

    /// Report apparent size rather than the space actually allocated on disk.
    #[arg(short = 'a', long, action = ArgAction::SetTrue)]
    pub apparent_size: bool,

    /// Break the tree down by file extension instead of by directory.
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "json")]
    pub by_type: bool,

    /// Emit machine-readable JSON on stdout.
    #[arg(long, action = ArgAction::SetTrue)]
    pub json: bool,

    /// Ignore any cached snapshot and do not write one.
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_cache: bool,

    /// Maximum entry depth to scan and retain; zero keeps only the root.
    #[arg(long, value_name = "N")]
    pub max_depth: Option<usize>,

    /// Never colorize output.
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_color: bool,

    /// Exit successfully even when unreadable paths make the result partial.
    #[arg(long, action = ArgAction::SetTrue)]
    pub allow_partial: bool,
}

impl Cli {
    /// Run the command, writing to `out`.
    pub fn run(&self, out: &mut dyn Write) -> anyhow::Result<RunOutcome> {
        let cache_path = if self.no_cache { None } else { default_cache_path(&self.path) };
        let config = OpenConfig {
            scan: ScanConfig { max_depth: self.max_depth, ..ScanConfig::default() },
            cache_path,
            save_on_open: !self.no_cache,
        };

        let (index, report) = open(&self.path, &config)?;

        if self.json {
            self.write_json(out, &index, &report)?;
        } else {
            self.write_human(out, &index, &report)?;
        }
        Ok(if report.is_complete() { RunOutcome::Complete } else { RunOutcome::Partial })
    }

    fn size_of(&self, roll: &crate::RollUp) -> u64 {
        if self.apparent_size { roll.bytes } else { roll.allocated }
    }

    fn write_human(
        &self,
        out: &mut dyn Write,
        index: &Index,
        report: &crate::OpenReport,
    ) -> anyhow::Result<()> {
        let color = self.use_color();
        let total = index.total();
        // Extension tallies retain apparent bytes only. Treat `--by-type` as an
        // apparent-size view end to end so its summary, rows, bars, and percentages
        // cannot disagree about the selected measure.
        let selected_total = if self.by_type { total.bytes } else { self.size_of(total) };
        let grand = selected_total.max(1);

        writeln!(
            out,
            "{}  {} files, {} dirs, {}",
            paint(&index.root_path().display().to_string(), Style::Bold, color),
            total.files,
            total.dirs,
            human_bytes(selected_total),
        )?;

        if self.by_type {
            // Per-extension tallies only carry apparent bytes, so the denominator has to
            // be apparent bytes too — sharing the allocated-bytes total used elsewhere
            // would print shares that never reach 100%.
            let by_type_total = total.bytes.max(1);
            let mut kinds: Vec<_> = total.by_ext.iter().collect();
            kinds.sort_by(|a, b| b.1.bytes.cmp(&a.1.bytes).then_with(|| a.0.cmp(b.0)));
            for (ext, tally) in kinds.into_iter().take(self.number) {
                let share = ratio(tally.bytes, by_type_total);
                writeln!(
                    out,
                    "{:>10}  {}  {:>4.0}%  {}  {} files",
                    human_bytes(tally.bytes),
                    bar(share, color),
                    share * 100.0,
                    paint(ext, Style::Cyan, color),
                    tally.files,
                )?;
            }
        } else {
            self.write_dir(out, index, EntryId::ROOT, 0, grand, color)?;
        }

        if !report.scan.is_complete() {
            let shown = report.scan.errors.len().min(3);
            writeln!(
                out,
                "\n{} {} path(s) could not be read; totals are incomplete",
                paint("warning:", Style::Yellow, color),
                report.scan.errors.len()
            )?;
            for err in report.scan.errors.iter().take(shown) {
                writeln!(out, "  {err}")?;
            }
        }
        Ok(())
    }

    fn write_dir(
        &self,
        out: &mut dyn Write,
        index: &Index,
        id: EntryId,
        depth: usize,
        grand: u64,
        color: bool,
    ) -> anyhow::Result<()> {
        if depth >= self.depth {
            return Ok(());
        }

        let mut rows: Vec<(u64, &OsStr, EntryId, bool)> = index
            .children_of(id)
            .unwrap_or_default()
            .into_iter()
            .map(|(name, child)| {
                let is_dir = index.kind_of(child).expect("child handle is live").is_dir();
                let size = index.rollup_of(child).map_or_else(
                    || {
                        let attrs = index.attrs_of(child).expect("child handle is live");
                        if self.apparent_size { attrs.size } else { attrs.allocated }
                    },
                    |roll| self.size_of(roll),
                );
                (size, name, child, is_dir)
            })
            .collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));

        for (size, name, child, is_dir) in rows.into_iter().take(self.number) {
            let share = ratio(size, grand);
            let display_name = name.to_string_lossy();
            let label = if is_dir {
                paint(&format!("{display_name}/"), Style::Blue, color)
            } else {
                display_name.into_owned()
            };
            writeln!(
                out,
                "{:>10}  {}  {:>4.0}%  {}{}",
                human_bytes(size),
                bar(share, color),
                share * 100.0,
                "  ".repeat(depth),
                label,
            )?;
            if is_dir {
                self.write_dir(out, index, child, depth + 1, grand, color)?;
            }
        }
        Ok(())
    }

    fn write_json(
        &self,
        out: &mut dyn Write,
        index: &Index,
        report: &crate::OpenReport,
    ) -> anyhow::Result<()> {
        let total = index.total();
        writeln!(out, "{{")?;
        writeln!(out, "  \"schema\": {},", quote(JSON_SCHEMA))?;
        writeln!(
            out,
            "  \"generator\": {},",
            quote(&format!("fdu {}", env!("CARGO_PKG_VERSION")))
        )?;
        writeln!(out, "  \"root\": {},", quote(&index.root_path().display().to_string()))?;
        writeln!(
            out,
            "  \"source\": {},",
            quote(match report.path_taken {
                OpenPath::ColdScan => "cold_scan",
                OpenPath::WarmRevalidate => "warm_revalidate",
            })
        )?;
        writeln!(out, "  \"complete\": {},", report.is_complete())?;
        writeln!(out, "  \"freshness\": {},", quote(freshness_label(index.freshness())))?;
        writeln!(out, "  \"errors\": [")?;
        for (position, error) in report.errors().iter().enumerate() {
            let comma = if position + 1 == report.errors().len() { "" } else { "," };
            writeln!(out, "    {}{comma}", quote(&error.to_string()))?;
        }
        writeln!(out, "  ],")?;

        write!(out, "  \"by_extension\": {{")?;
        let mut kinds: Vec<_> = total.by_ext.iter().collect();
        kinds.sort_by(|a, b| b.1.bytes.cmp(&a.1.bytes).then_with(|| a.0.cmp(b.0)));
        for (i, (ext, tally)) in kinds.iter().enumerate() {
            if i > 0 {
                write!(out, ",")?;
            }
            write!(
                out,
                "\n    {}: {{\"files\": {}, \"bytes\": {}}}",
                quote(ext),
                tally.files,
                tally.bytes
            )?;
        }
        if kinds.is_empty() {
            writeln!(out, "}},")?;
        } else {
            writeln!(out, "\n  }},")?;
        }

        write!(out, "  \"tree\": ")?;
        self.write_json_node(out, index, EntryId::ROOT, OsStr::new("."), 0, 2)?;
        writeln!(out)?;
        writeln!(out, "}}")?;
        Ok(())
    }

    fn write_json_node(
        &self,
        out: &mut dyn Write,
        index: &Index,
        id: EntryId,
        name: &OsStr,
        depth: usize,
        indent: usize,
    ) -> anyhow::Result<()> {
        let pad = " ".repeat(indent);
        let attrs = index.attrs_of(id).expect("tree handle is live");
        let is_dir = index.kind_of(id).expect("tree handle is live").is_dir();

        writeln!(out, "{{")?;
        writeln!(out, "{pad}  \"name\": {},", quote(&name.to_string_lossy()))?;
        writeln!(
            out,
            "{pad}  \"kind\": {},",
            quote(entry_kind_label(index.kind_of(id).expect("tree handle is live")))
        )?;

        if let Some(roll) = index.rollup_of(id) {
            writeln!(out, "{pad}  \"bytes\": {},", roll.bytes)?;
            writeln!(out, "{pad}  \"allocated\": {},", roll.allocated)?;
            writeln!(out, "{pad}  \"files\": {},", roll.files)?;
            writeln!(out, "{pad}  \"dirs\": {},", roll.dirs)?;
            write!(out, "{pad}  \"newest_mtime_ns\": {}", roll.newest_mtime_ns)?;
        } else {
            writeln!(out, "{pad}  \"bytes\": {},", attrs.size)?;
            writeln!(out, "{pad}  \"allocated\": {},", attrs.allocated)?;
            write!(out, "{pad}  \"newest_mtime_ns\": {}", attrs.mtime_ns)?;
        }

        if is_dir && depth < self.depth {
            let mut rows: Vec<(u64, &OsStr, EntryId)> = index
                .children_of(id)
                .unwrap_or_default()
                .into_iter()
                .map(|(child_name, child)| {
                    let size = index.rollup_of(child).map_or_else(
                        || {
                            let attrs = index.attrs_of(child).expect("child handle is live");
                            if self.apparent_size { attrs.size } else { attrs.allocated }
                        },
                        |roll| self.size_of(roll),
                    );
                    (size, child_name, child)
                })
                .collect();
            rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
            let rows: Vec<_> = rows.into_iter().take(self.number).collect();

            if !rows.is_empty() {
                writeln!(out, ",")?;
                writeln!(out, "{pad}  \"children\": [")?;
                for (i, (_, child_name, child)) in rows.iter().enumerate() {
                    write!(out, "{pad}    ")?;
                    self.write_json_node(out, index, *child, child_name, depth + 1, indent + 4)?;
                    if i + 1 < rows.len() {
                        write!(out, ",")?;
                    }
                    writeln!(out)?;
                }
                write!(out, "{pad}  ]")?;
            }
        }
        writeln!(out)?;
        write!(out, "{pad}}}")?;
        Ok(())
    }

    fn use_color(&self) -> bool {
        if self.no_color || self.json {
            return false;
        }
        // NO_COLOR is honored whenever it is set to anything non-empty, per the
        // no-color.org convention.
        if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
            return false;
        }
        io::stdout().is_terminal()
    }
}

#[derive(Clone, Copy)]
enum Style {
    Bold,
    Blue,
    Cyan,
    Yellow,
}

impl Style {
    fn code(self) -> &'static str {
        match self {
            Self::Bold => "1",
            Self::Blue => "34",
            Self::Cyan => "36",
            Self::Yellow => "33",
        }
    }
}

fn paint(text: &str, style: Style, color: bool) -> String {
    if color { format!("\u{1b}[{}m{text}\u{1b}[0m", style.code()) } else { text.to_string() }
}

fn entry_kind_label(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::File => "file",
        EntryKind::Dir => "dir",
        EntryKind::Symlink => "symlink",
        EntryKind::Other => "other",
    }
}

fn freshness_label(freshness: Freshness) -> &'static str {
    match freshness {
        Freshness::Fresh => "fresh",
        Freshness::Reconciling => "reconciling",
        Freshness::Stale => "stale",
        Freshness::Partial => "partial",
    }
}

fn ratio(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let r = part as f64 / whole as f64;
    r.clamp(0.0, 1.0)
}

// Rounding a fraction to one of eleven bar widths is exactly the case where float-cast
// lints have nothing to protect: the value is clamped to [0, 1] before the cast and the
// result is clamped to WIDTH after it.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
fn bar(share: f64, color: bool) -> String {
    const WIDTH: usize = 10;
    let filled = ((share.clamp(0.0, 1.0) * WIDTH as f64).round() as usize).min(WIDTH);
    let rendered = format!("{}{}", "█".repeat(filled), "░".repeat(WIDTH - filled));
    paint(&rendered, Style::Blue, color)
}

/// Format a byte count the way a person reads it.
fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    #[allow(clippy::cast_precision_loss)]
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Encode a string as a JSON string literal.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema_fixture() -> (Cli, Index, crate::OpenReport) {
        let cli = Cli {
            path: PathBuf::from("/fixture"),
            depth: 2,
            number: 10,
            apparent_size: true,
            by_type: false,
            json: true,
            no_cache: true,
            max_depth: None,
            no_color: true,
            allow_partial: false,
        };
        let attrs = |size, mtime_ns| crate::Attrs {
            size,
            allocated: size,
            mtime_ns,
            ctime_ns: mtime_ns,
            inode: u64::try_from(mtime_ns).expect("positive fixture time"),
            dev: 1,
        };
        let mut index = Index::new("/fixture");
        index.apply(&crate::Observation::new(vec![
            crate::Op::Upsert {
                path: PathBuf::from("directory"),
                kind: EntryKind::Dir,
                attrs: attrs(0, 1),
            },
            crate::Op::Upsert {
                path: PathBuf::from("directory/nested.bin"),
                kind: EntryKind::File,
                attrs: attrs(5, 5),
            },
            crate::Op::Upsert {
                path: PathBuf::from("file.txt"),
                kind: EntryKind::File,
                attrs: attrs(4, 4),
            },
            crate::Op::Upsert {
                path: PathBuf::from("link"),
                kind: EntryKind::Symlink,
                attrs: attrs(3, 3),
            },
            crate::Op::Upsert {
                path: PathBuf::from("special"),
                kind: EntryKind::Other,
                attrs: attrs(2, 2),
            },
        ]));
        index.set_initial_freshness(false);
        let report = crate::OpenReport {
            path_taken: OpenPath::ColdScan,
            scan: crate::ScanReport {
                dirs_read: 1,
                entries: 5,
                errors: vec![crate::Error::io(
                    "/fixture/denied",
                    std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
                )],
            },
        };
        (cli, index, report)
    }

    #[test]
    fn bytes_render_at_human_scale() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(999), "999 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(1536), "1.5 KiB");
        assert_eq!(human_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(human_bytes(150 * 1024 * 1024), "150 MiB");
    }

    #[test]
    fn bars_saturate_rather_than_overflow() {
        assert_eq!(bar(0.0, false).chars().filter(|c| *c == '█').count(), 0);
        assert_eq!(bar(1.0, false).chars().filter(|c| *c == '█').count(), 10);
        assert_eq!(bar(2.0, false).chars().filter(|c| *c == '█').count(), 10);
        assert_eq!(bar(0.5, false).chars().filter(|c| *c == '█').count(), 5);
    }

    #[test]
    fn ratio_is_safe_against_an_empty_tree() {
        assert!((ratio(5, 0) - 0.0).abs() < f64::EPSILON);
        assert!((ratio(1, 2) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn json_strings_escape_control_and_quote_characters() {
        assert_eq!(quote("plain"), "\"plain\"");
        assert_eq!(quote("say \"hi\""), "\"say \\\"hi\\\"\"");
        assert_eq!(quote("back\\slash"), "\"back\\\\slash\"");
        assert_eq!(quote("line\nbreak"), "\"line\\nbreak\"");
        assert_eq!(quote("bell\u{7}"), "\"bell\\u0007\"");
    }

    #[test]
    fn paint_is_a_no_op_when_color_is_off() {
        assert_eq!(paint("text", Style::Bold, false), "text");
        assert!(paint("text", Style::Bold, true).contains("\u{1b}["));
    }

    #[test]
    fn schema_v2_golden_covers_kinds_and_partial_errors() {
        let (cli, index, report) = schema_fixture();
        let mut output = Vec::new();
        cli.write_json(&mut output, &index, &report).expect("render JSON");
        let rendered = String::from_utf8(output).expect("UTF-8 fixture");
        assert_eq!(rendered, include_str!("testdata/tree-schema-v2.json"));
    }

    #[test]
    fn json_child_order_uses_the_selected_size_measure() {
        let (mut cli, mut index, report) = schema_fixture();
        cli.apparent_size = false;
        index.apply(&crate::Observation::new(vec![
            crate::Op::Upsert {
                path: PathBuf::from("apparent-heavy"),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 1_000, allocated: 1, ..Default::default() },
            },
            crate::Op::Upsert {
                path: PathBuf::from("allocated-heavy"),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 1, allocated: 2_000, ..Default::default() },
            },
        ]));

        let mut output = Vec::new();
        cli.write_json(&mut output, &index, &report).expect("render JSON");
        let rendered = String::from_utf8(output).expect("UTF-8 fixture");
        let allocated = rendered.find("\"name\": \"allocated-heavy\"").expect("allocated file");
        let apparent = rendered.find("\"name\": \"apparent-heavy\"").expect("apparent file");

        assert!(allocated < apparent);
    }
}
