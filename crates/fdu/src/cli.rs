//! The `fdu` command line.
//!
//! The CLI serves two audiences from one binary, and neither is an afterthought:
//!
//! - **Humans** get colored, fixed-column tree output with percentage bars, sensible
//!   defaults, and `NO_COLOR` plus pipe detection so redirection degrades cleanly.
//! - **Agents** get `--help` as the complete source of truth, JSON whose schema is
//!   versioned with the tool, and meaningful exit codes — no pager, no prompts, no
//!   interactive surprises.

use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Style as AnsiStyle, Styles};
use clap::{ArgAction, ColorChoice, CommandFactory, FromArgMatches, Parser, ValueEnum};

use crate::index::{EntryId, Index};
use crate::{EntryKind, Freshness, OpenConfig, OpenPath, ScanConfig, default_cache_path, open};

/// The JSON schema identifier. Bump the version on any breaking shape change so an
/// agent can tell what it is parsing without guessing from the payload.
const JSON_SCHEMA: &str = "fdu.tree/2";

const SKILL_TEMPLATE: &str = include_str!("skills/SKILL.md");

const STYLE_HEADING: AnsiStyle = AnsiColor::Cyan.on_default().bold();
const STYLE_DIRECTORY: AnsiStyle = AnsiColor::Cyan.on_default();
const STYLE_BAR: AnsiStyle = AnsiColor::Green.on_default();
const STYLE_WARNING: AnsiStyle = AnsiColor::Yellow.on_default().bold();
const STYLE_ERROR: AnsiStyle = AnsiColor::Red.on_default().bold();
const STYLE_CAUSE: AnsiStyle = AnsiStyle::new().dimmed();
const CLI_STYLES: Styles = Styles::styled()
    .header(STYLE_HEADING)
    .usage(STYLE_HEADING)
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .error(STYLE_ERROR)
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Yellow.on_default());

const AFTER_HELP: &str = "Examples:\n  fdu\n  fdu --depth 3 --number 20 ~/src\n  fdu --by-type ~/Downloads\n  fdu --json --depth 1 --number 50 .\n\nOutput and automation:\n  Human output reports allocated disk space unless --apparent-size is set.\n  Results go to stdout; warnings and errors go to stderr.\n  JSON is schema-versioned, never colorized, and includes completeness and truncation.\n  For automation, check the exit status, complete, errors, tree_truncated, and scan_max_depth.\n  The command never prompts, pages, or animates progress.\n\nResult scope:\n  --depth and --number limit only the rendered view.\n  --max-depth limits the scan scope and retained index.\n\nCache:\n  Unless --no-cache is set, fdu reads and writes a snapshot in the user cache directory.\n\nColor:\n  --color overrides NO_COLOR and FORCE_COLOR. In auto mode, NO_COLOR disables color,\n  FORCE_COLOR enables it, and otherwise the destination must be a terminal.\n\nExit status:\n  0  Complete result, or a partial result accepted with --allow-partial\n  1  Fatal filesystem or cache error\n  2  Partial result, or command-line usage error";

/// When terminal styling should be enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ColorWhen {
    /// Style output only when its destination is a terminal.
    #[default]
    Auto,
    /// Style output even when its destination is redirected.
    Always,
    /// Never style output.
    Never,
}

/// Successful command outcome. Partial results are rendered before the caller returns
/// exit status 2, so scripts can opt into them without confusing them with complete data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunOutcome {
    /// Every path in scope was read successfully.
    Complete,
    /// Output was produced, but one or more filesystem paths could not be read.
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
    styles = CLI_STYLES,
    after_help = AFTER_HELP
)]
// A command line is a flat bag of independent switches. Folding these into enums to
// satisfy the lint would obscure the one thing this struct exists to mirror: the flags a
// user actually types.
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Directory to summarize.
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Directory levels to show; does not limit scanning.
    #[arg(short, long, default_value_t = 2, value_name = "N")]
    pub depth: usize,

    /// Entries to show per directory, largest first.
    #[arg(short = 'n', long, default_value_t = 10, value_name = "N")]
    pub number: usize,

    /// Use apparent bytes instead of allocated disk space.
    #[arg(short = 'a', long, action = ArgAction::SetTrue)]
    pub apparent_size: bool,

    /// Group totals by file extension instead of directory.
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "json")]
    pub by_type: bool,

    /// Write schema-versioned JSON to stdout.
    #[arg(long, action = ArgAction::SetTrue)]
    pub json: bool,

    /// Do not read or write the snapshot cache.
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_cache: bool,

    /// Limit scanning and retention to N entry levels.
    #[arg(long, value_name = "N")]
    pub max_depth: Option<usize>,

    /// Colorize human output: auto, always, or never.
    #[arg(long, value_name = "WHEN", default_value = "auto", hide_possible_values = true)]
    pub color: ColorWhen,

    /// Accept incomplete totals when paths cannot be read.
    #[arg(long, action = ArgAction::SetTrue)]
    pub allow_partial: bool,

    /// Print a portable agent skill to stdout.
    #[arg(
        long,
        action = ArgAction::SetTrue,
        conflicts_with_all = [
            "path",
            "depth",
            "number",
            "apparent_size",
            "by_type",
            "json",
            "no_cache",
            "max_depth",
            "allow_partial"
        ]
    )]
    pub skill: bool,
}

impl Cli {
    /// Run the command, writing results to `out` and warnings to `diagnostic`.
    pub fn run(
        &self,
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        stdout_is_terminal: bool,
        stderr_is_terminal: bool,
    ) -> anyhow::Result<RunOutcome> {
        if self.skill {
            write!(out, "{}", compose_skill())?;
            return Ok(RunOutcome::Complete);
        }

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
            self.write_human(
                out,
                diagnostic,
                &index,
                &report,
                stdout_is_terminal,
                stderr_is_terminal,
            )?;
        }
        Ok(if report.is_complete() { RunOutcome::Complete } else { RunOutcome::Partial })
    }

    fn size_of(&self, roll: &crate::RollUp) -> u64 {
        if self.apparent_size { roll.bytes } else { roll.allocated }
    }

    fn write_human(
        &self,
        out: &mut dyn Write,
        diagnostic: &mut dyn Write,
        index: &Index,
        report: &crate::OpenReport,
        stdout_is_terminal: bool,
        stderr_is_terminal: bool,
    ) -> anyhow::Result<()> {
        let color = self.use_color(stdout_is_terminal);
        let diagnostic_color = self.use_color(stderr_is_terminal);
        let total = index.total();
        // Extension tallies retain apparent bytes only. Treat `--by-type` as an
        // apparent-size view end to end so its summary, rows, bars, and percentages
        // cannot disagree about the selected measure.
        let selected_total = if self.by_type { total.bytes } else { self.size_of(total) };
        let grand = selected_total.max(1);

        writeln!(
            out,
            "{}  {} {}, {} {}, {}",
            paint(&index.root_path().display().to_string(), STYLE_HEADING, color),
            total.files,
            plural(total.files, "file", "files"),
            total.dirs,
            plural(total.dirs, "dir", "dirs"),
            human_bytes(selected_total),
        )?;

        if self.by_type {
            // Per-extension tallies only carry apparent bytes, so the denominator has to
            // be apparent bytes too — sharing the allocated-bytes total used elsewhere
            // would print shares that never reach 100%.
            let by_type_total = total.bytes.max(1);
            let named = index.by_ext_named(total);
            let mut kinds: Vec<_> = named.iter().collect();
            kinds.sort_by(|a, b| b.1.bytes.cmp(&a.1.bytes).then_with(|| a.0.cmp(b.0)));
            for (ext, tally) in kinds.into_iter().take(self.number) {
                let share = ratio(tally.bytes, by_type_total);
                writeln!(
                    out,
                    "{:>10}  {}  {:>4.0}%  {}  {} {}",
                    human_bytes(tally.bytes),
                    bar(share, color),
                    share * 100.0,
                    paint(ext, STYLE_DIRECTORY, color),
                    tally.files,
                    plural(tally.files, "file", "files"),
                )?;
            }
        } else {
            self.write_dir(out, index, EntryId::ROOT, 0, grand, color)?;
        }

        if !report.scan.is_complete() {
            let shown = report.scan.errors.len().min(3);
            writeln!(
                diagnostic,
                "{} {} {} could not be read; totals are incomplete",
                paint("warning:", STYLE_WARNING, diagnostic_color),
                report.scan.errors.len(),
                if report.scan.errors.len() == 1 { "path" } else { "paths" }
            )?;
            for err in report.scan.errors.iter().take(shown) {
                writeln!(diagnostic, "  {err}")?;
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

        let mut stack: Vec<_> = self
            .human_rows(index, id)
            .into_iter()
            .rev()
            .map(|(size, name, child, is_dir)| (size, name, child, is_dir, depth))
            .collect();

        while let Some((size, name, child, is_dir, row_depth)) = stack.pop() {
            let share = ratio(size, grand);
            let display_name = name.to_string_lossy();
            let label = if is_dir {
                paint(&format!("{display_name}/"), STYLE_DIRECTORY, color)
            } else {
                display_name.into_owned()
            };
            writeln!(
                out,
                "{:>10}  {}  {:>4.0}%  {}{}",
                human_bytes(size),
                bar(share, color),
                share * 100.0,
                "  ".repeat(row_depth),
                label,
            )?;
            if is_dir && row_depth + 1 < self.depth {
                stack.extend(self.human_rows(index, child).into_iter().rev().map(
                    |(child_size, child_name, child_id, child_is_dir)| {
                        (child_size, child_name, child_id, child_is_dir, row_depth + 1)
                    },
                ));
            }
        }
        Ok(())
    }

    fn human_rows<'a>(
        &self,
        index: &'a Index,
        id: EntryId,
    ) -> Vec<(u64, &'a OsStr, EntryId, bool)> {
        let mut rows: Vec<_> = index
            .children_of(id)
            .into_iter()
            .flatten()
            .map(|(name, child)| {
                let is_dir = index.kind_of(child).expect("child handle is live").is_dir();
                let size = self.entry_size(index, child);
                (size, name, child, is_dir)
            })
            .collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
        rows.truncate(self.number);
        rows
    }

    fn entry_size(&self, index: &Index, id: EntryId) -> u64 {
        index.rollup_of(id).map_or_else(
            || {
                let attrs = index.attrs_of(id).expect("tree handle is live");
                if self.apparent_size { attrs.size } else { attrs.allocated }
            },
            |roll| self.size_of(roll),
        )
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
        write_raw_os_field(out, 2, "root_raw", index.root_path().as_os_str())?;
        writeln!(
            out,
            "  \"source\": {},",
            quote(match report.path_taken {
                OpenPath::ColdScan => "cold_scan",
                OpenPath::WarmRevalidate => "warm_revalidate",
            })
        )?;
        writeln!(out, "  \"complete\": {},", report.is_complete())?;
        writeln!(out, "  \"display_depth\": {},", self.depth)?;
        writeln!(out, "  \"entries_per_directory\": {},", self.number)?;
        match self.max_depth {
            Some(max_depth) => writeln!(out, "  \"scan_max_depth\": {max_depth},")?,
            None => writeln!(out, "  \"scan_max_depth\": null,")?,
        }
        writeln!(out, "  \"tree_truncated\": {},", self.tree_is_truncated(index))?;
        writeln!(out, "  \"freshness\": {},", quote(freshness_label(index.freshness())))?;
        writeln!(out, "  \"errors\": [")?;
        for (position, error) in report.errors().iter().enumerate() {
            let comma = if position + 1 == report.errors().len() { "" } else { "," };
            writeln!(out, "    {}{comma}", quote(&error.to_string()))?;
        }
        writeln!(out, "  ],")?;

        write!(out, "  \"by_extension\": {{")?;
        let named = index.by_ext_named(total);
        let mut kinds: Vec<_> = named.iter().collect();
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
        self.write_json_tree(out, index)?;
        writeln!(out)?;
        writeln!(out, "}}")?;
        Ok(())
    }

    /// Return whether the rendered tree omits any entry retained in `index`.
    fn tree_is_truncated(&self, index: &Index) -> bool {
        let mut stack = vec![(EntryId::ROOT, 0_usize)];
        while let Some((id, depth)) = stack.pop() {
            let Some(children) = index.children_of(id) else {
                continue;
            };
            let child_count = children.len();
            if child_count > 0 && (depth >= self.depth || child_count > self.number) {
                return true;
            }
            stack.extend(children.filter_map(|(_, child)| {
                index.kind_of(child).is_some_and(EntryKind::is_dir).then_some((child, depth + 1))
            }));
        }
        false
    }

    fn write_json_tree(&self, out: &mut dyn Write, index: &Index) -> anyhow::Result<()> {
        let mut stack = vec![JsonAction::Node {
            id: EntryId::ROOT,
            name: OsStr::new("."),
            depth: 0,
            indent: 2,
            prefix_indent: None,
        }];

        while let Some(action) = stack.pop() {
            match action {
                JsonAction::Node { id, name, depth, indent, prefix_indent } => {
                    if let Some(prefix_indent) = prefix_indent {
                        write!(out, "{}", " ".repeat(prefix_indent))?;
                    }
                    let pad = " ".repeat(indent);
                    let attrs = index.attrs_of(id).expect("tree handle is live");
                    let kind = index.kind_of(id).expect("tree handle is live");

                    writeln!(out, "{{")?;
                    writeln!(out, "{pad}  \"name\": {},", quote(&name.to_string_lossy()))?;
                    write_raw_os_field(out, indent + 2, "name_raw", name)?;
                    writeln!(out, "{pad}  \"kind\": {},", quote(entry_kind_label(kind)))?;

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

                    let rows = if kind.is_dir() && depth < self.depth {
                        self.json_rows(index, id)
                    } else {
                        Vec::new()
                    };
                    if rows.is_empty() {
                        writeln!(out)?;
                        write!(out, "{pad}}}")?;
                    } else {
                        writeln!(out, ",")?;
                        writeln!(out, "{pad}  \"children\": [")?;
                        stack.push(JsonAction::CloseNode { indent });
                        stack.push(JsonAction::CloseChildren { indent });
                        let row_count = rows.len();
                        for (position, (_, child_name, child)) in rows.into_iter().enumerate().rev()
                        {
                            stack.push(JsonAction::FinishChild { comma: position + 1 < row_count });
                            stack.push(JsonAction::Node {
                                id: child,
                                name: child_name,
                                depth: depth + 1,
                                indent: indent + 4,
                                prefix_indent: Some(indent + 4),
                            });
                        }
                    }
                }
                JsonAction::FinishChild { comma } => {
                    if comma {
                        write!(out, ",")?;
                    }
                    writeln!(out)?;
                }
                JsonAction::CloseChildren { indent } => {
                    write!(out, "{}  ]", " ".repeat(indent))?;
                }
                JsonAction::CloseNode { indent } => {
                    writeln!(out)?;
                    write!(out, "{}}}", " ".repeat(indent))?;
                }
            }
        }
        Ok(())
    }

    fn json_rows<'a>(&self, index: &'a Index, id: EntryId) -> Vec<(u64, &'a OsStr, EntryId)> {
        let mut rows: Vec<_> = index
            .children_of(id)
            .into_iter()
            .flatten()
            .map(|(name, child)| (self.entry_size(index, child), name, child))
            .collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
        rows.truncate(self.number);
        rows
    }

    fn use_color(&self, stdout_is_terminal: bool) -> bool {
        ColorContext::from_environment(self.color, self.json, self.skill, stdout_is_terminal)
            .enabled()
    }
}

enum JsonAction<'a> {
    Node { id: EntryId, name: &'a OsStr, depth: usize, indent: usize, prefix_indent: Option<usize> },
    FinishChild { comma: bool },
    CloseChildren { indent: usize },
    CloseNode { indent: usize },
}

/// Run `fdu` through its real process boundary and return its stable numeric exit code.
///
/// This is shared by the native binary and the Python wheel's console entry point so
/// parsing, streams, color, diagnostics, broken pipes, and exit semantics cannot drift.
pub fn run_process<I, T>(args: I) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let stdout_is_terminal = stdout.is_terminal();
    let stderr_is_terminal = stderr.is_terminal();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut diagnostic = stderr.lock();

    run_with_io(&args, &mut out, &mut diagnostic, stdout_is_terminal, stderr_is_terminal)
}

fn run_with_io(
    args: &[OsString],
    out: &mut dyn Write,
    diagnostic: &mut dyn Write,
    stdout_is_terminal: bool,
    stderr_is_terminal: bool,
) -> u8 {
    let requested_color = requested_color(args);
    let json_requested = flag_is_present(args, "--json");
    let skill_requested = flag_is_present(args, "--skill");
    let command = Cli::command().color(ColorChoice::Always);
    let matches = match command.try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) => {
            let use_stderr = error.use_stderr();
            let destination_is_terminal =
                if use_stderr { stderr_is_terminal } else { stdout_is_terminal };
            let color = ColorContext::from_environment(
                requested_color,
                json_requested,
                skill_requested,
                destination_is_terminal,
            )
            .enabled();
            let rendered = error.render();
            let write_result = if use_stderr {
                write_styled(diagnostic, &rendered, color)
            } else {
                write_styled(out, &rendered, color)
            };
            if let Err(write_error) = write_result {
                return match write_error.kind() {
                    io::ErrorKind::BrokenPipe => 0,
                    _ => 1,
                };
            }
            return u8::try_from(error.exit_code()).unwrap_or(2);
        }
    };
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = write!(diagnostic, "{error}");
            return 2;
        }
    };

    let result =
        cli.run(out, diagnostic, stdout_is_terminal, stderr_is_terminal).and_then(|outcome| {
            out.flush()?;
            Ok(outcome)
        });
    let diagnostic_color =
        ColorContext::from_environment(cli.color, cli.json, cli.skill, stderr_is_terminal)
            .enabled();
    finish(result, cli.allow_partial, diagnostic, diagnostic_color)
}

fn write_styled(
    out: &mut dyn Write,
    rendered: &clap::builder::StyledStr,
    color: bool,
) -> io::Result<()> {
    if color { write!(out, "{}", rendered.ansi()) } else { write!(out, "{rendered}") }
}

fn finish(
    result: anyhow::Result<RunOutcome>,
    allow_partial: bool,
    diagnostic: &mut dyn Write,
    color: bool,
) -> u8 {
    match result {
        Ok(RunOutcome::Complete) => 0,
        Ok(RunOutcome::Partial) if allow_partial => 0,
        Ok(RunOutcome::Partial) => 2,
        Err(error) if is_broken_pipe(&error) => 0,
        Err(error) => {
            let _ = writeln!(diagnostic, "{} {error}", paint("fdu:", STYLE_ERROR, color));
            for cause in error.chain().skip(1) {
                let cause = format!("  caused by: {cause}");
                let _ = writeln!(diagnostic, "{}", paint(&cause, STYLE_CAUSE, color));
            }
            1
        }
    }
}

fn is_broken_pipe(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|io_error| io_error.kind() == io::ErrorKind::BrokenPipe)
    })
}

fn requested_color(args: &[OsString]) -> ColorWhen {
    let mut arguments = args.iter().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--" {
            break;
        }
        if argument == "--color" {
            return arguments.next().and_then(|value| color_value(value)).unwrap_or_default();
        }
        if let Some(value) = argument.to_str().and_then(|value| value.strip_prefix("--color=")) {
            return color_value(OsStr::new(value)).unwrap_or_default();
        }
    }
    ColorWhen::Auto
}

fn color_value(value: &OsStr) -> Option<ColorWhen> {
    match value.to_str()? {
        "auto" => Some(ColorWhen::Auto),
        "always" => Some(ColorWhen::Always),
        "never" => Some(ColorWhen::Never),
        _ => None,
    }
}

fn flag_is_present(args: &[OsString], flag: &str) -> bool {
    args.iter().skip(1).take_while(|argument| *argument != "--").any(|argument| argument == flag)
}

#[derive(Clone, Copy)]
// Each boolean is an independent external input to the color contract. Naming them here
// is clearer than encoding unrelated facts into bit flags or positional arguments.
#[allow(clippy::struct_excessive_bools)]
struct ColorContext {
    when: ColorWhen,
    json: bool,
    skill: bool,
    no_color_env: bool,
    force_color_env: bool,
    destination_is_terminal: bool,
}

impl ColorContext {
    fn from_environment(
        when: ColorWhen,
        json: bool,
        skill: bool,
        destination_is_terminal: bool,
    ) -> Self {
        let no_color_env = std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
        let force_color_env =
            std::env::var_os("FORCE_COLOR").is_some_and(|value| !value.is_empty() && value != "0");
        Self { when, json, skill, no_color_env, force_color_env, destination_is_terminal }
    }

    fn enabled(self) -> bool {
        if self.json || self.skill {
            return false;
        }
        match self.when {
            ColorWhen::Always => true,
            ColorWhen::Never => false,
            ColorWhen::Auto if self.no_color_env => false,
            ColorWhen::Auto if self.force_color_env => true,
            ColorWhen::Auto => self.destination_is_terminal,
        }
    }
}

fn paint(text: &str, style: AnsiStyle, color: bool) -> String {
    if color { format!("{style}{text}{style:#}") } else { text.to_string() }
}

fn plural<'a>(count: u64, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn compose_skill() -> String {
    compose_skill_from(SKILL_TEMPLATE)
}

fn compose_skill_from(template: &str) -> String {
    // Git checkouts may translate the Markdown resource to CRLF on Windows. Keep the
    // public skill byte-stable across installation platforms before substituting the
    // reviewed package version.
    template.replace("\r\n", "\n").replace("__FDU_VERSION__", env!("CARGO_PKG_VERSION"))
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
    paint(&rendered, STYLE_BAR, color)
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

/// Write lossless identity metadata when an operating-system string cannot be represented
/// by the display-oriented JSON string beside it.
fn write_raw_os_field(
    out: &mut dyn Write,
    indent: usize,
    field: &str,
    value: &OsStr,
) -> io::Result<()> {
    let Some((encoding, hex)) = raw_os_identity(value) else {
        return Ok(());
    };
    let pad = " ".repeat(indent);
    writeln!(
        out,
        "{pad}{}: {{\"encoding\": {}, \"hex\": {}}},",
        quote(field),
        quote(encoding),
        quote(&hex)
    )
}

fn raw_os_identity(value: &OsStr) -> Option<(&'static str, String)> {
    if value.to_str().is_some() {
        return None;
    }

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;

        Some(("unix-bytes", hex_bytes(value.as_bytes().iter().copied())))
    }

    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;

        Some(("windows-wtf16le", hex_bytes(value.encode_wide().flat_map(u16::to_le_bytes))))
    }

    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

#[cfg(any(unix, windows))]
fn hex_bytes(bytes: impl IntoIterator<Item = u8>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::new();
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    const DEEP_RENDER_CHILD_ENV: &str = "FDU_DEEP_RENDER_CHILD";
    const DEEP_RENDER_DEPTH: usize = 1_024;
    const DEEP_RENDER_STACK_BYTES: usize = 64 * 1_024;

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("output failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

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
            color: ColorWhen::Never,
            allow_partial: false,
            skill: false,
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
        index.apply_ok(&crate::Observation::new(vec![
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
                attribution: crate::scan::WalkAttribution::default(),
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
        assert_eq!(paint("text", STYLE_HEADING, false), "text");
        assert!(paint("text", STYLE_HEADING, true).contains("\u{1b}["));
    }

    #[test]
    fn color_decision_has_stable_precedence_and_machine_output_is_plain() {
        let auto_terminal = ColorContext {
            when: ColorWhen::Auto,
            json: false,
            skill: false,
            no_color_env: false,
            force_color_env: false,
            destination_is_terminal: true,
        };

        assert!(auto_terminal.enabled());
        assert!(!ColorContext { destination_is_terminal: false, ..auto_terminal }.enabled());
        assert!(
            ColorContext { force_color_env: true, destination_is_terminal: false, ..auto_terminal }
                .enabled()
        );
        assert!(
            !ColorContext { no_color_env: true, force_color_env: true, ..auto_terminal }.enabled()
        );
        assert!(
            ColorContext {
                when: ColorWhen::Always,
                no_color_env: true,
                destination_is_terminal: false,
                ..auto_terminal
            }
            .enabled()
        );
        assert!(
            !ColorContext { when: ColorWhen::Never, force_color_env: true, ..auto_terminal }
                .enabled()
        );
        assert!(!ColorContext { json: true, ..auto_terminal }.enabled());
        assert!(!ColorContext { skill: true, ..auto_terminal }.enabled());
    }

    #[test]
    fn portable_skill_is_self_contained_and_exactly_versioned() {
        let skill = compose_skill();

        assert!(skill.starts_with("---\nname: fdu\n"));
        assert!(!skill.contains('\r'), "the public skill must use portable LF endings");
        assert!(skill.contains(&format!("uvx --from fdu=={} fdu", env!("CARGO_PKG_VERSION"))));
        assert!(!skill.contains("__FDU_VERSION__"));
        assert!(!skill.contains("uvx --from fdu fdu"));
        assert!(!skill.contains("fdu==latest"), "the runnable command must not float releases");
        assert_eq!(
            compose_skill_from("---\r\nversion: __FDU_VERSION__\r\n"),
            format!("---\nversion: {}\n", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn run_outcomes_and_broken_pipes_have_stable_exit_codes() {
        let mut diagnostic = Vec::new();
        assert_eq!(finish(Ok(RunOutcome::Complete), false, &mut diagnostic, false), 0);
        assert_eq!(finish(Ok(RunOutcome::Partial), false, &mut diagnostic, false), 2);
        assert_eq!(finish(Ok(RunOutcome::Partial), true, &mut diagnostic, false), 0);

        let broken_pipe =
            anyhow::Error::new(io::Error::new(io::ErrorKind::BrokenPipe, "reader closed"))
                .context("render output");
        assert_eq!(finish(Err(broken_pipe), false, &mut diagnostic, false), 0);
        assert!(diagnostic.is_empty());

        let args = [OsString::from("fdu"), OsString::from("--help")];
        assert_eq!(
            run_with_io(&args, &mut FailingWriter, &mut diagnostic, false, false),
            1,
            "a non-pipe help-output failure is fatal"
        );
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
        index.apply_ok(&crate::Observation::new(vec![
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

    #[test]
    fn tree_truncation_distinguishes_exact_fit_from_hidden_entries() {
        let (mut cli, index, _) = schema_fixture();

        cli.depth = 2;
        cli.number = 10;
        assert!(!cli.tree_is_truncated(&index));

        cli.depth = 1;
        assert!(cli.tree_is_truncated(&index));

        cli.depth = 2;
        cli.number = 4;
        assert!(!cli.tree_is_truncated(&index));

        cli.number = 3;
        assert!(cli.tree_is_truncated(&index));

        cli.depth = 0;
        cli.number = 10;
        assert!(cli.tree_is_truncated(&index));

        cli.depth = 2;
        cli.number = 0;
        assert!(cli.tree_is_truncated(&index));
    }

    #[test]
    fn deep_rendering_is_stack_safe() {
        if std::env::var_os(DEEP_RENDER_CHILD_ENV).is_some() {
            run_deep_render_child();
            return;
        }

        let output = Command::new(std::env::current_exe().expect("current test executable"))
            .args(["--exact", "cli::tests::deep_rendering_is_stack_safe", "--nocapture"])
            .env(DEEP_RENDER_CHILD_ENV, "1")
            .output()
            .expect("run deep-render child");

        assert!(
            output.status.success(),
            "deep renderer failed in child process\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn run_deep_render_child() {
        let (mut cli, _, report) = schema_fixture();
        cli.depth = DEEP_RENDER_DEPTH + 1;
        cli.number = 1;

        let mut index = Index::new("/fixture");
        let mut path = PathBuf::new();
        for depth in 0..DEEP_RENDER_DEPTH {
            path.push("d");
            index.apply_ok(&crate::Observation::new(vec![crate::Op::Upsert {
                path: path.clone(),
                kind: EntryKind::Dir,
                attrs: crate::Attrs {
                    mtime_ns: i64::try_from(depth).expect("fixture depth fits i64"),
                    ..Default::default()
                },
            }]));
        }
        index.set_initial_freshness(false);

        std::thread::Builder::new()
            .name("deep-render".to_string())
            .stack_size(DEEP_RENDER_STACK_BYTES)
            .spawn(move || {
                let mut human = Vec::new();
                let mut diagnostic = Vec::new();
                cli.write_human(&mut human, &mut diagnostic, &index, &report, false, false)
                    .expect("render human tree");
                assert!(!cli.tree_is_truncated(&index));

                let mut json = Vec::new();
                cli.write_json(&mut json, &index, &report).expect("render JSON tree");
            })
            .expect("spawn deep-render thread")
            .join()
            .expect("deep-render thread");
    }

    fn assert_json_preserves_raw_identity(
        root: PathBuf,
        first: &OsStr,
        second: &OsStr,
        encoding: &str,
        root_hex: &str,
        first_hex: &str,
        second_hex: &str,
    ) {
        assert_eq!(first.to_string_lossy(), second.to_string_lossy());

        let (mut cli, _, _) = schema_fixture();
        cli.depth = 1;
        let mut index = Index::new(root);
        index.apply_ok(&crate::Observation::new(vec![
            crate::Op::Upsert {
                path: PathBuf::from(first),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 1, allocated: 1, ..Default::default() },
            },
            crate::Op::Upsert {
                path: PathBuf::from(second),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 1, allocated: 1, ..Default::default() },
            },
        ]));
        index.set_initial_freshness(false);
        let report = crate::OpenReport {
            path_taken: OpenPath::ColdScan,
            scan: crate::ScanReport {
                dirs_read: 1,
                entries: 2,
                errors: Vec::new(),
                attribution: crate::scan::WalkAttribution::default(),
            },
        };

        let mut output = Vec::new();
        cli.write_json(&mut output, &index, &report).expect("render JSON");
        let rendered = String::from_utf8(output).expect("JSON is UTF-8");
        let lossy_name = quote(&first.to_string_lossy());

        assert_eq!(rendered.matches(&format!("\"name\": {lossy_name}")).count(), 2);
        assert!(
            rendered.contains(&format!(
                "\"root_raw\": {{\"encoding\": \"{encoding}\", \"hex\": \"{root_hex}\"}}"
            )),
            "{rendered}"
        );
        for hex in [first_hex, second_hex] {
            assert!(
                rendered.contains(&format!(
                    "\"name_raw\": {{\"encoding\": \"{encoding}\", \"hex\": \"{hex}\"}}"
                )),
                "{rendered}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn json_preserves_distinct_non_utf8_unix_names() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        assert_json_preserves_raw_identity(
            PathBuf::from(OsString::from_vec(vec![b'/', b'r', 0x80])),
            &OsString::from_vec(vec![b'n', 0x80]),
            &OsString::from_vec(vec![b'n', 0x81]),
            "unix-bytes",
            "2f7280",
            "6e80",
            "6e81",
        );
    }

    #[cfg(windows)]
    #[test]
    fn json_preserves_distinct_non_unicode_windows_names() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        assert_json_preserves_raw_identity(
            PathBuf::from(OsString::from_wide(&[u16::from(b'R'), u16::from(b':'), 0xd800])),
            &OsString::from_wide(&[u16::from(b'n'), 0xd800]),
            &OsString::from_wide(&[u16::from(b'n'), 0xd801]),
            "windows-wtf16le",
            "52003a0000d8",
            "6e0000d8",
            "6e0001d8",
        );
    }
}
