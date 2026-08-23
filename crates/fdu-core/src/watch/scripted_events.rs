//! A deterministic backend for the conditions a real filesystem cannot be asked for.
//!
//! Causal sequencing covers changes a test can cause: create a file, wait for the record
//! that proves it arrived. It cannot cover the conditions that matter most and occur
//! least. Every [`crate::InvalidateReason`] except `Requested` exists for a situation the
//! kernel produces under pressure -- a dropped-event queue, a rename whose halves never
//! meet, a directory whose watch was installed a moment too late -- and none of them can
//! be provoked on demand.
//!
//! So the *backend* becomes the seam, not the observation. A script replaces the events
//! `notify` would deliver and nothing else: the same coalescing, the same stat
//! verification, the same delta path. A scripted event is still verified against the real
//! filesystem before it becomes an `Op`, so "a watch sample is valid at its stat point"
//! survives intact and this stays a test seam rather than a back door -- a script cannot
//! state a fact about the tree, only claim that something there may have changed.
//!
//! The format is line-oriented rather than JSON because the engine has no JSON parser and
//! a test seam is a poor reason to acquire one. Fields are tab-separated so a path may
//! contain spaces; blank lines and `#` comments are ignored. Paths are relative to the
//! watch root, which is what keeps a script portable between machines:
//!
//! ```text
//! # a create, then the rename that has no partner
//! create      src/new.txt
//! rename-from src/old.txt
//! # the kernel dropped events under this path
//! rescan      src
//! # and the backend itself failed
//! error       simulated backend failure
//! ```

use std::path::Path;

use notify::EventKind;
use notify::event::{CreateKind, Flag, ModifyKind, RemoveKind, RenameMode};

/// One line of a script, already resolved against the watch root.
pub(super) type ScriptedEvent = notify::Result<notify::Event>;

/// Read a script, resolving every path against `root`.
///
/// Errors name a line number for the same reason the rule manifest's do: this is a file a
/// person writes, and a rejection without a location is a worse message than none.
pub(super) fn read_script(path: &Path, root: &Path) -> Result<Vec<ScriptedEvent>, String> {
    let source =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    parse_script(&source, root).map_err(|error| format!("{}: {error}", path.display()))
}

/// Parse a script's text. Separated from reading so the format is testable without a file.
pub(super) fn parse_script(source: &str, root: &Path) -> Result<Vec<ScriptedEvent>, String> {
    let mut events = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t').map(str::trim).filter(|field| !field.is_empty());
        let verb = fields.next().ok_or_else(|| format!("line {line_number}: empty directive"))?;
        let arguments: Vec<&str> = fields.collect();
        events.push(event_for(verb, &arguments, root, line_number)?);
    }
    Ok(events)
}

fn event_for(
    verb: &str,
    arguments: &[&str],
    root: &Path,
    line: usize,
) -> Result<ScriptedEvent, String> {
    let kind = match verb {
        "create" => EventKind::Create(CreateKind::File),
        "create-dir" => EventKind::Create(CreateKind::Folder),
        "modify" => EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
        "remove" => EventKind::Remove(RemoveKind::Any),
        "rename-from" => EventKind::Modify(ModifyKind::Name(RenameMode::From)),
        "rename-to" => EventKind::Modify(ModifyKind::Name(RenameMode::To)),
        "rename-both" => EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
        "rescan" => EventKind::Any,
        "error" => {
            let message = arguments.join(" ");
            if message.is_empty() {
                return Err(format!("line {line}: error takes a message"));
            }
            return Ok(Err(notify::Error::generic(&message)));
        }
        other => {
            return Err(format!(
                "line {line}: unknown directive {other:?}; expected create, create-dir, modify, \
                 remove, rename-from, rename-to, rename-both, rescan, or error"
            ));
        }
    };

    let wanted = if verb == "rename-both" { 2 } else { 1 };
    if arguments.len() != wanted {
        return Err(format!("line {line}: {verb} takes {wanted} path(s), got {}", arguments.len()));
    }
    let mut paths = Vec::with_capacity(wanted);
    for argument in arguments {
        let relative = Path::new(argument);
        if relative.is_absolute() {
            return Err(format!(
                "line {line}: {argument:?} is absolute; script paths are relative to the watch \
                 root so a script stays portable"
            ));
        }
        paths.push(root.join(relative));
    }

    let mut event = notify::Event::new(kind);
    event.paths = paths;
    if verb == "rescan" {
        // The one flag that is not a path fact: every backend signals a dropped-event
        // queue through it, and swallowing it silently corrupts any index built on events.
        event = event.set_flag(Flag::Rescan);
    }
    Ok(Ok(event))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn parse(source: &str) -> Vec<ScriptedEvent> {
        parse_script(source, Path::new("/root")).expect("script parses")
    }

    #[test]
    fn paths_resolve_against_the_watch_root() {
        let events = parse("create\tsrc/new.txt\n");
        let event = events[0].as_ref().expect("an ok event");
        assert_eq!(event.paths, vec![PathBuf::from("/root/src/new.txt")]);
        assert_eq!(event.kind, EventKind::Create(CreateKind::File));
    }

    #[test]
    fn a_rescan_carries_the_flag_every_backend_signals_drops_with() {
        let events = parse("rescan\tsrc\n");
        let event = events[0].as_ref().expect("an ok event");
        assert_eq!(event.flag(), Some(Flag::Rescan));
    }

    #[test]
    fn a_rename_pair_carries_both_sides() {
        let events = parse("rename-both\told.txt\tnew.txt\n");
        let event = events[0].as_ref().expect("an ok event");
        assert_eq!(event.paths.len(), 2);
        assert_eq!(event.kind, EventKind::Modify(ModifyKind::Name(RenameMode::Both)));
    }

    #[test]
    fn an_error_line_becomes_a_backend_error() {
        let events = parse("error\tsimulated backend failure\n");
        assert!(events[0].is_err());
    }

    #[test]
    fn comments_and_blank_lines_are_skipped() {
        assert!(parse("# nothing here\n\n   \n").is_empty());
    }

    /// The format is tested for what it rejects: an accepted script proves less.
    #[test]
    fn a_script_rejects_what_it_cannot_mean() {
        for (source, expected) in [
            ("teleport\ta.txt\n", "unknown directive"),
            ("create\n", "takes 1 path"),
            ("rename-both\tonly.txt\n", "takes 2 path"),
            ("create\t/absolute.txt\n", "is absolute"),
            ("error\n", "error takes a message"),
        ] {
            let Err(error) = parse_script(source, Path::new("/root")) else {
                panic!("{source:?} must be rejected");
            };
            assert!(error.contains(expected), "{error:?} should mention {expected:?}");
        }
    }
}
