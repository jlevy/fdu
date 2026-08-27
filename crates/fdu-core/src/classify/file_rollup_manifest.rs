//! Dependency-free parser for the File Rollup v3 registry profile.
//!
//! The engine needs classifier and browsing semantics, not TOML as a general-purpose
//! configuration language. Keeping this parser beside the existing compact fdu manifest
//! parser admits the shared reviewed document without adding a TOML dependency to every
//! standalone fdu binary.

use std::collections::{BTreeMap, BTreeSet};

pub(super) const SCHEMA_VERSION: u32 = 3;
pub(super) const MAX_EXTENSION_COMPONENTS: u8 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Group {
    pub id: String,
    pub label: String,
    pub order: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Family {
    pub id: String,
    pub label: String,
    pub group: String,
    pub order: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Kind {
    pub id: String,
    pub family: String,
    pub group: String,
    pub content_family: String,
    pub extensions: Vec<String>,
    pub filenames: Vec<String>,
    pub shebangs: Vec<String>,
    pub priority: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Registry {
    pub schema_version: u32,
    pub revision: u32,
    pub max_extension_components: u8,
    pub groups: Vec<Group>,
    pub families: Vec<Family>,
    pub kinds: Vec<Kind>,
}

enum Block {
    Group(Group, BTreeSet<String>),
    Family(Family, BTreeSet<String>),
    Kind(Kind, BTreeSet<String>),
}

pub(super) fn looks_like_registry(source: &str) -> bool {
    source.lines().any(|line| line.trim().starts_with("schema_version"))
        || source.lines().any(|line| matches!(line.trim(), "[[group]]" | "[[family]]"))
}

pub(super) fn parse(source: &str) -> Result<Registry, String> {
    let mut registry = Registry::default();
    let mut top_seen = BTreeSet::new();
    let mut block = None;
    let mut presentation_multiline = false;
    for (line_index, raw) in source.lines().enumerate() {
        let line_number = line_index + 1;
        if presentation_multiline {
            if raw.contains("\"\"\"") {
                presentation_multiline = false;
            }
            continue;
        }
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match line {
            "[[group]]" => {
                close(&mut registry, block.take(), line_number)?;
                block = Some(Block::Group(Group::default(), BTreeSet::new()));
                continue;
            }
            "[[family]]" => {
                close(&mut registry, block.take(), line_number)?;
                block = Some(Block::Family(Family::default(), BTreeSet::new()));
                continue;
            }
            "[[kind]]" => {
                close(&mut registry, block.take(), line_number)?;
                block =
                    Some(Block::Kind(Kind { priority: 100, ..Kind::default() }, BTreeSet::new()));
                continue;
            }
            _ => {}
        }
        let (raw_key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {line_number}: expected key = value"))?;
        let key = raw_key.trim();
        let value = raw_value.trim();
        match block.as_mut() {
            None => {
                unique(&mut top_seen, key, line_number)?;
                match key {
                    "schema_version" => registry.schema_version = integer(value, line_number)?,
                    "registry_revision" => registry.revision = integer(value, line_number)?,
                    "max_extension_components" => {
                        registry.max_extension_components = integer(value, line_number)?;
                    }
                    _ => return Err(format!("line {line_number}: unknown registry field {key:?}")),
                }
            }
            Some(Block::Group(group, seen)) => {
                unique(seen, key, line_number)?;
                match key {
                    "id" => group.id = string(value, line_number)?,
                    "label" => group.label = string(value, line_number)?,
                    "order" => group.order = integer(value, line_number)?,
                    _ => return Err(format!("line {line_number}: unknown group field {key:?}")),
                }
            }
            Some(Block::Family(family, seen)) => {
                unique(seen, key, line_number)?;
                match key {
                    "id" => family.id = string(value, line_number)?,
                    "label" => family.label = string(value, line_number)?,
                    "group" => family.group = string(value, line_number)?,
                    "order" => family.order = integer(value, line_number)?,
                    // Presentation metadata is validated by shape, but is intentionally
                    // not retained by the filesystem engine.
                    "hue" => {
                        let hue = finite_number(value, key, line_number)?;
                        if !(0.0..360.0).contains(&hue) {
                            return Err(format!(
                                "line {line_number}: hue must be in [0, 360) degrees"
                            ));
                        }
                    }
                    "lightness_rank" => {
                        let _ = finite_number(value, key, line_number)?;
                    }
                    "deviation" if value.starts_with("\"\"\"") => {
                        presentation_multiline = !value[3..].contains("\"\"\"");
                    }
                    "linguist" | "linguist_color" | "deviation" => {
                        let _ = string(value, line_number)?;
                    }
                    _ => return Err(format!("line {line_number}: unknown family field {key:?}")),
                }
            }
            Some(Block::Kind(kind, seen)) => {
                unique(seen, key, line_number)?;
                match key {
                    "id" => kind.id = string(value, line_number)?,
                    "family" => kind.family = string(value, line_number)?,
                    "group" => kind.group = string(value, line_number)?,
                    "content_family" => kind.content_family = string(value, line_number)?,
                    "extensions" => kind.extensions = strings(value, line_number)?,
                    "filenames" => kind.filenames = strings(value, line_number)?,
                    "shebangs" => kind.shebangs = strings(value, line_number)?,
                    "priority" => kind.priority = integer(value, line_number)?,
                    _ => return Err(format!("line {line_number}: unknown kind field {key:?}")),
                }
            }
        }
    }
    if presentation_multiline {
        return Err("unterminated multiline presentation string".to_string());
    }
    close(&mut registry, block.take(), source.lines().count().saturating_add(1))?;
    require_fields(
        &top_seen,
        &["schema_version", "registry_revision", "max_extension_components"],
        "registry",
    )?;
    validate(&registry)?;
    Ok(registry)
}

fn close(registry: &mut Registry, block: Option<Block>, line: usize) -> Result<(), String> {
    match block {
        Some(Block::Group(value, seen)) => {
            require_fields(&seen, &["id", "label", "order"], "group")?;
            registry.groups.push(value);
        }
        Some(Block::Family(value, seen)) => {
            require_fields(&seen, &["id", "label", "group", "order", "hue"], "family")?;
            if seen.contains("linguist") != seen.contains("linguist_color") {
                return Err(format!(
                    "line {line}: family linguist and linguist_color must be present together"
                ));
            }
            if seen.contains("lightness_rank") && !seen.contains("deviation") {
                return Err(format!("line {line}: family lightness_rank requires a deviation"));
            }
            registry.families.push(value);
        }
        Some(Block::Kind(value, seen)) => {
            require_fields(
                &seen,
                &["id", "content_family", "extensions", "filenames", "shebangs", "priority"],
                "kind",
            )?;
            registry.kinds.push(value);
        }
        None => {}
    }
    Ok(())
}

fn require_fields(seen: &BTreeSet<String>, required: &[&str], table: &str) -> Result<(), String> {
    if let Some(missing) = required.iter().find(|field| !seen.contains(**field)) {
        return Err(format!("{table} is missing required field {missing:?}"));
    }
    Ok(())
}

fn unique(seen: &mut BTreeSet<String>, key: &str, line: usize) -> Result<(), String> {
    if !seen.insert(key.to_string()) {
        return Err(format!("line {line}: duplicate field {key:?}"));
    }
    Ok(())
}

fn string(value: &str, line: usize) -> Result<String, String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| format!("line {line}: expected a quoted string"))
}

fn strings(value: &str, line: usize) -> Result<Vec<String>, String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| format!("line {line}: expected a string array"))?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner.split(',').map(|item| string(item.trim(), line)).collect()
}

fn integer<T: std::str::FromStr>(value: &str, line: usize) -> Result<T, String> {
    value.parse().map_err(|_| format!("line {line}: expected a nonnegative integer"))
}

fn finite_number(value: &str, key: &str, line: usize) -> Result<f64, String> {
    let number =
        value.parse::<f64>().map_err(|_| format!("line {line}: {key} must be a finite number"))?;
    if !number.is_finite() {
        return Err(format!("line {line}: {key} must be a finite number"));
    }
    Ok(number)
}

fn validate(registry: &Registry) -> Result<(), String> {
    if registry.schema_version != SCHEMA_VERSION {
        return Err(format!("unsupported schema_version {}", registry.schema_version));
    }
    if registry.revision == 0 {
        return Err("registry_revision must be positive".to_string());
    }
    if registry.max_extension_components != MAX_EXTENSION_COMPONENTS {
        return Err(format!("max_extension_components must be {MAX_EXTENSION_COMPONENTS}"));
    }
    if registry.groups.is_empty() || registry.kinds.is_empty() {
        return Err("a registry requires at least one group and one kind".to_string());
    }

    let mut group_ids = BTreeSet::new();
    let mut group_orders = BTreeSet::new();
    for group in &registry.groups {
        valid_identity(&group.id, "group")?;
        if group.label.is_empty() {
            return Err(format!("group {} has no label", group.id));
        }
        if !group_ids.insert(group.id.as_str()) {
            return Err(format!("duplicate group id {:?}", group.id));
        }
        if !group_orders.insert(group.order) {
            return Err(format!("duplicate group order {}", group.order));
        }
    }
    if !group_ids.contains("other") {
        return Err("registry must declare the other group".to_string());
    }

    let mut family_groups = BTreeMap::new();
    let mut family_orders = BTreeSet::new();
    for family in &registry.families {
        valid_identity(&family.id, "family")?;
        if family.label.is_empty() {
            return Err(format!("family {} has no label", family.id));
        }
        if !group_ids.contains(family.group.as_str()) {
            return Err(format!("family {} names unknown group {:?}", family.id, family.group));
        }
        if family_groups.insert(family.id.as_str(), family.group.as_str()).is_some() {
            return Err(format!("duplicate family id {:?}", family.id));
        }
        if !family_orders.insert((family.group.as_str(), family.order)) {
            return Err(format!(
                "duplicate family order {} in group {:?}",
                family.order, family.group
            ));
        }
    }

    let mut kind_ids = BTreeSet::new();
    let mut extensions = BTreeMap::new();
    let mut filenames = BTreeMap::new();
    for kind in &registry.kinds {
        valid_identity(&kind.id, "kind")?;
        if !kind_ids.insert(kind.id.as_str()) {
            return Err(format!("duplicate kind id {:?}", kind.id));
        }
        let group = if kind.family.is_empty() {
            if kind.group.is_empty() {
                return Err(format!("kind {} names neither family nor group", kind.id));
            }
            kind.group.as_str()
        } else {
            let family_group = family_groups.get(kind.family.as_str()).ok_or_else(|| {
                format!("kind {} names unknown family {:?}", kind.id, kind.family)
            })?;
            if !kind.group.is_empty() && kind.group != *family_group {
                return Err(format!("kind {} group conflicts with family", kind.id));
            }
            family_group
        };
        if !group_ids.contains(group) {
            return Err(format!("kind {} names unknown group {group:?}", kind.id));
        }
        if !super::type_rule_manifest::MANIFEST_FAMILIES.contains(&kind.content_family.as_str()) {
            return Err(format!(
                "kind {} has invalid content_family {:?}",
                kind.id, kind.content_family
            ));
        }
        if kind.extensions.is_empty() && kind.filenames.is_empty() && kind.shebangs.is_empty() {
            return Err(format!("kind {} declares no evidence", kind.id));
        }
        for extension in &kind.extensions {
            let components: Vec<_> = extension.split('.').collect();
            if extension.starts_with('.')
                || extension != &extension.to_ascii_lowercase()
                || components.is_empty()
                || components.len() > usize::from(MAX_EXTENSION_COMPONENTS)
                || components.iter().any(|component| {
                    component.is_empty()
                        || component.len() > 12
                        || !component
                            .bytes()
                            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                })
            {
                return Err(format!("kind {} has invalid extension {:?}", kind.id, extension));
            }
            if let Some(previous) = extensions.insert(extension.as_str(), kind.id.as_str()) {
                return Err(format!(
                    "extension {extension:?} belongs to {previous} and {}",
                    kind.id
                ));
            }
        }
        for filename in &kind.filenames {
            if filename.is_empty()
                || filename != &filename.to_ascii_lowercase()
                || filename.contains('/')
                || filename.contains('\\')
            {
                return Err(format!("kind {} has invalid filename {:?}", kind.id, filename));
            }
            if let Some(previous) = filenames.insert(filename.as_str(), kind.id.as_str()) {
                return Err(format!("filename {filename:?} belongs to {previous} and {}", kind.id));
            }
        }
    }
    for family in &registry.families {
        if !registry
            .kinds
            .iter()
            .any(|kind| kind.family == family.id && !kind.extensions.is_empty())
        {
            return Err(format!("family {} has no declared extension evidence", family.id));
        }
    }
    Ok(())
}

fn valid_identity(value: &str, kind: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || !value.as_bytes()[0].is_ascii_lowercase()
    {
        return Err(format!("invalid {kind} id {value:?}"));
    }
    Ok(())
}

pub(super) fn fingerprint(registry: &Registry) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    fn add(hash: &mut u64, value: &[u8]) {
        let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
        for byte in length.to_le_bytes().iter().chain(value) {
            *hash = (*hash ^ u64::from(*byte)).wrapping_mul(PRIME);
        }
    }
    fn values(hash: &mut u64, items: &[String]) {
        for item in items {
            add(hash, item.as_bytes());
        }
    }
    let mut hash = OFFSET;
    add(&mut hash, b"file-rollup-registry-v3");
    add(&mut hash, &registry.revision.to_le_bytes());
    for group in &registry.groups {
        add(&mut hash, group.id.as_bytes());
        add(&mut hash, group.label.as_bytes());
        add(&mut hash, &group.order.to_le_bytes());
    }
    for family in &registry.families {
        add(&mut hash, family.id.as_bytes());
        add(&mut hash, family.label.as_bytes());
        add(&mut hash, family.group.as_bytes());
        add(&mut hash, &family.order.to_le_bytes());
    }
    for kind in &registry.kinds {
        add(&mut hash, kind.id.as_bytes());
        add(&mut hash, kind.family.as_bytes());
        add(&mut hash, kind.group.as_bytes());
        add(&mut hash, kind.content_family.as_bytes());
        values(&mut hash, &kind.extensions);
        values(&mut hash, &kind.filenames);
        values(&mut hash, &kind.shebangs);
        add(&mut hash, &kind.priority.to_le_bytes());
    }
    hash
}
