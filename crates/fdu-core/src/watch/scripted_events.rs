//! Deterministic filesystem-event hints for observation orchestration tests.
//!
//! A script replaces only the backend event source. Every named path still passes
//! through the production coalescer, admission policy, filesystem verification, exact
//! commit boundary, and recovery path, so it cannot inject an index fact.

use std::path::Path;

use notify::EventKind;
use notify::event::{CreateKind, Flag, ModifyKind, RemoveKind, RenameMode};

pub(super) type ScriptedEvent = notify::Result<notify::Event>;

pub(super) fn read_script(path: &Path, root: &Path) -> Result<Vec<ScriptedEvent>, String> {
    let source =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    parse_script(&source, root).map_err(|error| format!("{}: {error}", path.display()))
}

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
    fn a_rescan_carries_the_backend_loss_flag() {
        let events = parse("rescan\tsrc\n");
        assert_eq!(events[0].as_ref().expect("an ok event").flag(), Some(Flag::Rescan));
    }

    #[test]
    fn a_script_rejects_what_it_cannot_mean() {
        for (source, expected) in [
            ("teleport\ta.txt\n", "unknown directive"),
            ("create\n", "takes 1 path"),
            ("rename-both\tonly.txt\n", "takes 2 path"),
            ("error\n", "error takes a message"),
        ] {
            let error = parse_script(source, Path::new("/root")).expect_err("script must fail");
            assert!(error.contains(expected), "{error:?} should mention {expected:?}");
        }

        let absolute = std::env::current_dir().expect("current directory").join("absolute.txt");
        let source = format!("create\t{}\n", absolute.display());
        let error = parse_script(&source, Path::new("/root"))
            .expect_err("an absolute script path must fail on this platform");
        assert!(error.contains("is absolute"));
    }
}
