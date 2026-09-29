---
type: is
id: is-01m3nbsak6hftpnr1rx5x90d1w
title: "H171: bucketed .gitignore matching (literal-name and *.suffix maps, highest matching index wins)"
kind: task
status: closed
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies:
  - type: blocks
    target: is-01m3n1xqd4kg6q11yd8jcytg4c
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
hold: null
hold_until: null
created_at: 2026-09-29T01:17:08.070Z
updated_at: 2026-09-29T10:10:22.733Z
started_at: 2026-09-29T08:15:58.351Z
closed_at: 2026-09-29T10:10:22.733Z
close_reason: "Accepted and merged (2379233a): exp-178 (H171, default-tree -29.62%, summary -25.45%) and exp-179 (H175, default-tree -3.31%) on linux-v6.12, quiet, 20 pairs; placebos at zero; answers identical to the base and to git."
resolution: null
duplicate_of: null
---
At Gitignore::parse (gitignore.rs:104), bucket patterns: Basename with no metacharacter goes to literal_names HashMap<name, Vec<index>>; Basename *.suffix goes to suffixes, keyed by the text after the last '.' and confirmed by ends_with; everything else goes to residual. matches_components (gitignore.rs:125) probes both maps with the entry's name and extension, scans the residual, and answers patterns[max matching index].ignored, so last-match-wins, negation and directory_only are exact. Add ControlChain::is_ignored_within(dir_components, name, is_dir) and split a listing's directory once, in push_directory (index.rs:1684-1695) and per cached parent in SummaryControls::classify (execution.rs:797-801). Snapshot format unchanged. Optional: bucket anchored Fixed patterns by segment count (~80M more). Prototype (diff in notes): linux-v6.12 default-tree consumer instructions 2,067M -> 761M (-63%), summary 1,918M -> 660M, answers byte-identical over 1,629,566 JSON leaves, 93 control tests pass. Screen under load: tree -27-31%. ACCEPT (pre-register): default-tree (deciding) and aggregate-summary, controls on, linux-v6.12: wall -3% with 95% interval below zero; consumer Ir -50%; placebos (both arms --no-controls on linux-v6.12; default-tree on balanced-1M) include zero; peak RSS non-inferior; route differential identical. TESTS: keep the linear matcher under cfg(test) and property-test random rule sets (literal, suffix, anchored, negated, dir-only, escaped, [..], non-UTF-8, names ending in '.', '*.') x random names against it, plus the git check-ignore verdict table. ~150 lines.

## Notes

diff --git a/crates/fdu-core/src/control.rs b/crates/fdu-core/src/control.rs
index 01600c41..c884927e 100644
--- a/crates/fdu-core/src/control.rs
+++ b/crates/fdu-core/src/control.rs
@@ -766,15 +766,37 @@ impl ControlChain {
         if self.governing.is_empty() {
             return false;
         }
-        gitignore::with_components(directory, Some(name), |components| {
-            self.governing
-                .iter()
-                .find_map(|(leading, source)| {
-                    let relative = components.get(*leading..).unwrap_or_default();
-                    source.matcher.matches_components(relative, is_dir)
-                })
-                .unwrap_or(false)
-        })
+        gitignore::with_components(directory, Some(name), |components| self.decide(components, is_dir))
+    }
+
+    /// PROTOTYPE (research, not for merge): [`Self::is_ignored`] with the directory's
+    /// components already split once for the whole listing.
+    pub(crate) fn is_ignored_within(&self, directory: &[&[u8]], name: &[u8], is_dir: bool) -> bool {
+        if self.governing.is_empty() {
+            return false;
+        }
+        let total = directory.len() + 1;
+        if total <= 32 {
+            let mut inline: [&[u8]; 32] = [&[]; 32];
+            inline[..directory.len()].copy_from_slice(directory);
+            inline[directory.len()] = name;
+            self.decide(&inline[..total], is_dir)
+        } else {
+            let mut spilled: Vec<&[u8]> = Vec::with_capacity(total);
+            spilled.extend_from_slice(directory);
+            spilled.push(name);
+            self.decide(&spilled, is_dir)
+        }
+    }
+
+    fn decide(&self, components: &[&[u8]], is_dir: bool) -> bool {
+        self.governing
+            .iter()
+            .find_map(|(leading, source)| {
+                let relative = components.get(*leading..).unwrap_or_default();
+                source.matcher.matches_components(relative, is_dir)
+            })
+            .unwrap_or(false)
     }
 }
 
diff --git a/crates/fdu-core/src/control/gitignore.rs b/crates/fdu-core/src/control/gitignore.rs
index 63bdce12..463bea21 100644
--- a/crates/fdu-core/src/control/gitignore.rs
+++ b/crates/fdu-core/src/control/gitignore.rs
@@ -70,6 +70,14 @@ pub(super) fn with_components<R>(
 #[derive(Clone, Debug, Default)]
 pub(super) struct Gitignore {
     patterns: Vec<Pattern>,
+    /// PROTOTYPE (research, not for merge): basename patterns that are one literal name,
+    /// by name, so an entry meets them in one lookup instead of one glob each.
+    literal_names: std::collections::HashMap<Vec<u8>, Vec<u32>>,
+    /// PROTOTYPE: `*.suffix` basename patterns keyed by the text after the pattern's last
+    /// `.`; the entry's own extension selects the candidates and `ends_with` confirms.
+    suffixes: std::collections::HashMap<Vec<u8>, Vec<(Vec<u8>, u32)>>,
+    /// PROTOTYPE: every other pattern, matched in order as before.
+    residual: Vec<u32>,
 }
 
 #[derive(Clone, Debug)]
@@ -102,8 +110,26 @@ enum Segment {
 
 impl Gitignore {
     pub(super) fn parse(source: &[u8]) -> Self {
-        let patterns = source.split(|byte| *byte == b'\n').filter_map(Pattern::parse).collect();
-        Self { patterns }
+        let patterns: Vec<Pattern> =
+            source.split(|byte| *byte == b'\n').filter_map(Pattern::parse).collect();
+        let mut this = Self { patterns, ..Self::default() };
+        for (index, pattern) in this.patterns.iter().enumerate() {
+            let index = u32::try_from(index).expect("a control file holds fewer than 2^32 rules");
+            match pattern.bucket() {
+                Bucket::Literal(name) => {
+                    this.literal_names.entry(name.to_vec()).or_default().push(index);
+                }
+                Bucket::Suffix(suffix) => {
+                    let dot = suffix.iter().rposition(|byte| *byte == b'.').unwrap_or(0);
+                    this.suffixes
+                        .entry(suffix[dot + 1..].to_vec())
+                        .or_default()
+                        .push((suffix.to_vec(), index));
+                }
+                Bucket::Residual => this.residual.push(index),
+            }
+        }
+        this
     }
 
     /// Accepted executable rules; comments, blank lines, and rejected patterns do not count.
@@ -123,14 +149,47 @@ impl Gitignore {
 
     /// [`Self::matches`] over components already split, relative to this file's directory.
     pub(super) fn matches_components(&self, components: &[&[u8]], is_dir: bool) -> Option<bool> {
-        self.patterns
-            .iter()
-            .filter(|pattern| pattern.matches(components, is_dir))
-            .map(|pattern| pattern.ignored)
-            .next_back()
+        // PROTOTYPE: the last matching line wins, so the answer is the highest index among
+        // the literal-name bucket, the suffix bucket, and the residual list.
+        let name = *components.last()?;
+        let mut last: Option<u32> = None;
+        if let Some(indices) = self.literal_names.get(name) {
+            for &index in indices {
+                if self.patterns[index as usize].admits_kind(is_dir) {
+                    last = last.max(Some(index));
+                }
+            }
+        }
+        if let Some(dot) = name.iter().rposition(|byte| *byte == b'.') {
+            if let Some(candidates) = self.suffixes.get(&name[dot + 1..]) {
+                for (suffix, index) in candidates {
+                    if name.ends_with(suffix) && self.patterns[*index as usize].admits_kind(is_dir)
+                    {
+                        last = last.max(Some(*index));
+                    }
+                }
+            }
+        }
+        for &index in &self.residual {
+            if self.patterns[index as usize].matches(components, is_dir) {
+                last = last.max(Some(index));
+            }
+        }
+        last.map(|index| self.patterns[index as usize].ignored)
     }
 }
 
+/// PROTOTYPE: how a pattern is indexed for matching.
+enum Bucket<'a> {
+    Literal(&'a [u8]),
+    Suffix(&'a [u8]),
+    Residual,
+}
+
+fn has_glob_metachar(bytes: &[u8]) -> bool {
+    bytes.iter().any(|byte| matches!(byte, b'*' | b'?' | b'[' | b'\\'))
+}
+
 impl Pattern {
     fn parse(raw: &[u8]) -> Option<Self> {
         let mut line = raw.strip_suffix(b"\r").unwrap_or(raw);
@@ -207,6 +266,29 @@ impl Pattern {
         Some(Self { ignored, directory_only, shape, segments })
     }
 
+    /// PROTOTYPE: a directory-only pattern matches only a directory.
+    fn admits_kind(&self, is_dir: bool) -> bool {
+        !self.directory_only || is_dir
+    }
+
+    /// PROTOTYPE: the bucket this pattern is matched from.
+    fn bucket(&self) -> Bucket<'_> {
+        if self.shape != Shape::Basename {
+            return Bucket::Residual;
+        }
+        let Some(Segment::Glob(glob)) = self.segments.first() else {
+            return Bucket::Residual;
+        };
+        if !has_glob_metachar(glob) {
+            Bucket::Literal(glob)
+        } else if glob.len() >= 2 && glob[0] == b'*' && glob[1] == b'.' && !has_glob_metachar(&glob[1..])
+        {
+            Bucket::Suffix(&glob[1..])
+        } else {
+            Bucket::Residual
+        }
+    }
+
     fn matches(&self, path: &[&[u8]], is_dir: bool) -> bool {
         if path.is_empty() {
             return false;
diff --git a/crates/fdu-core/src/index.rs b/crates/fdu-core/src/index.rs
index cc4e067f..4a305e67 100644
--- a/crates/fdu-core/src/index.rs
+++ b/crates/fdu-core/src/index.rs
@@ -1573,6 +1573,9 @@ impl IndexHandle {
     }
 }
 
+/// PROTOTYPE (research, not for merge): whether the builder folds files without entries.
+static PROTO_TREE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
+
 /// Private parent-first builder for a cold index that is not yet externally visible.
 ///
 /// Directory groups arrive while filesystem workers are still running. Applying each
@@ -1683,14 +1686,85 @@ impl DetachedIndexBuilder {
         // once here rather than looked up per child (H163).
         let chain = (!parent_ignored && !self.index.controls.is_empty())
             .then(|| self.index.controls.chain_for(path));
+        // PROTOTYPE (research, not for merge): split the listing's directory once.
+        let dir_components: Vec<&[u8]> = if chain.is_some() {
+            path.components()
+                .filter_map(|component| match component {
+                    std::path::Component::Normal(value) => Some(value.as_encoded_bytes()),
+                    _ => None,
+                })
+                .collect()
+        } else {
+            Vec::new()
+        };
         self.index.reserve_detached_children(parent, children.len());
+        // PROTOTYPE (research cost proxy, not exact, not for merge): with FDU_PROTO_TREE
+        // set, fold files into the parent's roll-up scalars without allocating entries and
+        // leave their names in the listing so the worker frees them (H159).
+        if *PROTO_TREE.get_or_init(|| std::env::var_os("FDU_PROTO_TREE").is_some()) {
+            for child in children.iter_mut() {
+                let kind = child.kind;
+                crate::counters::bump(|counts| counts.upserts += 1);
+                let ignored = match &chain {
+                    Some(chain) => chain.is_ignored_within(
+                        &dir_components,
+                        child.name.as_encoded_bytes(),
+                        kind.is_dir(),
+                    ),
+                    None => parent_ignored,
+                };
+                if kind == EntryKind::File {
+                    let attrs = child.attrs;
+                    let rollup = self.index.entry_mut(parent).rollup_mut();
+                    rollup.all.files += 1;
+                    rollup.all.bytes += attrs.size;
+                    rollup.all.allocated += attrs.allocated;
+                    rollup.all.newest_mtime_ns = rollup.all.newest_mtime_ns.max(attrs.mtime_ns);
+                    if !ignored {
+                        rollup.unignored.files += 1;
+                        rollup.unignored.bytes += attrs.size;
+                        rollup.unignored.allocated += attrs.allocated;
+                        rollup.unignored.newest_mtime_ns =
+                            rollup.unignored.newest_mtime_ns.max(attrs.mtime_ns);
+                    }
+                    self.inserted = self.inserted.saturating_add(1);
+                    continue;
+                }
+                if !kind.is_dir() {
+                    continue;
+                }
+                let name = std::mem::take(&mut child.name);
+                let child_path = path.join(&name);
+                let child_id = self.index.alloc(Entry::new_detached(
+                    NewEntry {
+                        parent: Some(parent),
+                        name,
+                        ext_id: None,
+                        ignored,
+                        source: Source::Scanned,
+                        kind,
+                        attrs: child.attrs,
+                    },
+                    false,
+                ));
+                self.index.push_detached_child(parent, child_id);
+                let direct = self.index.contribution(child_id);
+                crate::counters::bump(|counts| counts.rollup_merges += 1);
+                self.index.entry_mut(parent).rollup_mut().merge(&direct);
+                self.directory_ids.insert(child_path, child_id);
+                self.inserted = self.inserted.saturating_add(1);
+            }
+            return Ok(());
+        }
         for child in children.drain(..) {
             let crate::scan::DetachedChild { name, kind, attrs, .. } = child;
             crate::counters::bump(|counts| counts.upserts += 1);
             let ext_id = (kind == EntryKind::File)
                 .then(|| self.index.intern_ext(&crate::classify::ext_bucket(&name)));
             let ignored = match &chain {
-                Some(chain) => chain.is_ignored(path, name.as_encoded_bytes(), kind.is_dir()),
+                Some(chain) => {
+                    chain.is_ignored_within(&dir_components, name.as_encoded_bytes(), kind.is_dir())
+                }
                 None => parent_ignored,
             };
             let child_path = kind.is_dir().then(|| path.join(&name));

2026-09-29: .gitignore matching algorithm survey launched (read-only agent, in the 0.2.1 session). Scope: ripgrep globset+ignore, git dir.c+wildmatch.c (reverse scan with early exit, nowildcardlen, PATTERN_FLAG_*), gitoxide gix-ignore/gix-glob, Sapling pathmatcher TreeMatcher (matches_directory Yes/No/Maybe), Mercurial rust hg-core matchers, jj, libgit2, regex-automata multi-pattern lazy DFA; microbenchmark instructions/entry on linux-v6.12's 358 .gitignore files for current matcher, H171 prototype, H171+anchored grouping, H173 live set, ignore::Gitignore, gix-ignore; every engine checked against git check-ignore; git t3070-wildmatch / t0008-ignores as a conformance table. If its results are not appended here, rerun it: clone those repos into attic/ at release tags, build the bench under a scratch dir (never in the repo; crates >=14 days old, pinned), measure with callgrind. Prototype diff for H171 is in fdu-sdul notes above.

2026-09-29: the matching-algorithm survey launched in the 0.2.1 session was stopped before reporting; nothing from it is recorded. It is now its own bead, fdu-p6vc, which this bead depends on.
