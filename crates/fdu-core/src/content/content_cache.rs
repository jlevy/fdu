//! Versioned content sidecar, independent of the metadata snapshot format.

use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use crate::classify::{
    Classification, ClassificationFlags, ContentFamily, DetectionConfidence, DetectionSource,
    FileTypeId,
};
use crate::stored_state::{
    AnalyzerProvenance, ContentTierIdentity, ENTRY_TIER_BYTES, EntryTierIdentity,
};
use crate::{Error, Fingerprint, Index, Result};

use super::{
    AnalysisApplyOutcome, AnalysisRequest, AnalysisSet, AnalyzerId, AnalyzerVersion,
    ContentProvenance, CoverageReason, FileAnalysis, LogicalWordStats, MetricValues,
};

const MAGIC: &[u8; 8] = b"FDUCTNT\0";
const TRAILER: &[u8; 8] = b"FDUCTEND";
/// On-disk format version. Bump on any layout change or any change to what a record means;
/// a sidecar of another version is a clean miss.
///
/// 5: the header records the engine fingerprint beside the version, at the offset a
/// snapshot's prologue gives it, and the content tier identity after the path encoding:
/// the entry tier the records were analyzed over, which holds their type rules, then the
/// analyzer set, the options fingerprint, and the analyzers.
const FORMAT_VERSION: u32 = 5;
const CHECKSUM_BYTES: usize = 4;
const MAX_CACHE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_RECORDS: u64 = 5_000_000;
const MAX_PATH_BYTES: usize = 1024 * 1024;
const MAX_TYPE_BYTES: usize = 256;
const MAX_ANALYZERS: usize = 16;
const MAX_ANALYZER_ID_BYTES: usize = 128;
const MAX_ERROR_BYTES: usize = 512;

/// Result of conditionally restoring one content sidecar.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ContentCacheLoad {
    /// Whether the sidecar header and integrity checks matched this request.
    pub usable: bool,
    /// Records accepted by the current metadata index.
    pub hits: u64,
    /// Apparent bytes represented by accepted records.
    pub bytes: u64,
    /// Accepted records with an expected non-analyzed coverage outcome.
    pub coverage_exclusions: u64,
    /// Records that no longer matched a live candidate.
    pub stale: u64,
    /// Candidates this restore walked. Cache-only completeness compares `hits` to this
    /// instead of walking `analysis_candidates` again.
    pub(crate) candidates: u64,
}

#[derive(Default)]
struct RestoreTimings {
    parse: Duration,
    apply: Duration,
}

impl RestoreTimings {
    fn add_parse(&mut self, started: Instant) {
        add_duration(&mut self.parse, started.elapsed());
    }

    fn add_apply(&mut self, started: Instant) {
        add_duration(&mut self.apply, started.elapsed());
    }

    fn publish(self) {
        crate::counters::bump(|counts| {
            counts.content_sidecar_parse_us =
                counts.content_sidecar_parse_us.saturating_add(duration_micros(self.parse));
            counts.content_sidecar_apply_us =
                counts.content_sidecar_apply_us.saturating_add(duration_micros(self.apply));
        });
    }
}

fn add_duration(total: &mut Duration, elapsed: Duration) {
    *total = total.saturating_add(elapsed);
}

fn duration_micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}

/// Derive the content-sidecar path without changing the metadata snapshot name.
pub fn content_cache_path(snapshot_path: &Path) -> PathBuf {
    let mut name = snapshot_path.as_os_str().to_os_string();
    name.push(".content");
    PathBuf::from(name)
}

/// Persist the content tier's sparse records as a separately invalidated sidecar, under the
/// identity the tier holds.
///
/// The tier's own identity decides everything a request could have said: an index with no
/// prepared tier writes nothing, and a prepared tier names an enabled analyzer set. A
/// request argument could only disagree with it, and a sidecar labelled with the tier's set
/// after a caller asked for another is worse than no argument at all.
pub fn save_content_cache(index: &Index, path: &Path) -> Result<()> {
    let Some(content) = index.content() else {
        return Ok(());
    };
    let Some(identity) = content.identity() else {
        return Ok(());
    };
    // The tier states its records' type rules once, in its entry tier, and a save writes it
    // only as the identity this index gives records of that set, so records produced under
    // any other, such as other type rules, never reach a sidecar under its label.
    if *identity != index.content_identity(identity.analysis) {
        return Err(Error::Snapshot(
            "content records were produced under another identity than their index".into(),
        ));
    }
    // Every record the tier holds carries its identity, because the tier refuses any other,
    // so which records are written is decided per record: those this pass verified.
    let records = content
        .records()
        .filter(|(path, record)| crate::stored_state::content_record_writable(index, path, record))
        .collect::<Vec<_>>();
    let record_count = u64::try_from(records.len())
        .map_err(|_| Error::Snapshot("content sidecar record count overflow".into()))?;
    if record_count > MAX_RECORDS {
        return Err(Error::Snapshot("content sidecar exceeds record limit".into()));
    }

    let mut buffer = Vec::new();
    buffer.extend_from_slice(MAGIC);
    buffer.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    buffer.extend_from_slice(&identity.entries.engine.to_le_bytes());
    buffer.push(crate::snapshot::path_encoding());
    put_identity(&mut buffer, identity)?;
    crate::snapshot::put_os_str(&mut buffer, index.root_path().as_os_str())?;
    buffer.extend_from_slice(&record_count.to_le_bytes());
    for (relative_path, record) in records {
        put_record(&mut buffer, relative_path, record)?;
    }
    let checksum = crate::snapshot::crc32c(&buffer);
    buffer.extend_from_slice(&checksum.to_le_bytes());
    buffer.extend_from_slice(TRAILER);
    crate::snapshot::write_atomically(path, &buffer)
}

/// Restore the records of a sidecar whose identity equals `wanted`, returning a miss for an
/// absent, corrupt, or foreign sidecar and for one of any other identity.
///
/// Equality, not containment: a sidecar of a wider analyzer set holds metrics a narrower
/// request did not ask for, and one produced under other analyzer versions, options, type
/// rules, entries, or engine counts differently. An identity this index cannot hold,
/// because it names another entry tier or type rules, restores nothing into it.
pub fn load_content_cache(
    index: &mut Index,
    wanted: &ContentTierIdentity,
    path: &Path,
) -> Result<ContentCacheLoad> {
    if !wanted.analysis.is_enabled() || *wanted != index.content_identity(wanted.analysis) {
        return Ok(ContentCacheLoad::default());
    }
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ContentCacheLoad::default());
        }
        Err(error) => return Err(Error::io(path, error)),
    };
    if metadata.len() > MAX_CACHE_BYTES {
        return Ok(ContentCacheLoad::default());
    }
    let read_started = crate::counters::enabled().then(std::time::Instant::now);
    let image = fs::read(path).map_err(|error| Error::io(path, error))?;
    crate::counters::add_elapsed(read_started, |counts, elapsed| {
        counts.content_sidecar_read_us = counts.content_sidecar_read_us.saturating_add(elapsed);
    });
    let mut timings = crate::counters::enabled().then(RestoreTimings::default);
    let parse_started = timings.as_ref().map(|_| Instant::now());
    let Some(mut stream) = parse_header(&image, index.root_path(), wanted) else {
        if let (Some(timings), Some(started)) = (&mut timings, parse_started) {
            timings.add_parse(started);
        }
        if let Some(timings) = timings {
            timings.publish();
        }
        return Ok(ContentCacheLoad::default());
    };
    if let (Some(timings), Some(started)) = (&mut timings, parse_started) {
        timings.add_parse(started);
    }
    index.prepare_content_analysis(AnalysisRequest {
        profile: wanted.analysis,
        ..AnalysisRequest::default()
    });
    // Keyed by hash rather than by order. This map is only ever drained by lookup —
    // nothing iterates it — so its ordering was never observable, and ordering a
    // `PathBuf` costs more than it looks: `Ord` walks components, so building a tree
    // pays about log2(n) comparisons per insert and each one walks the paths again.
    //
    // Worth about 3%, not more. A flat callgrind profile of a warm 14,542-file open
    // shows `compare_components` at 34% of instructions and it is tempting to read that
    // as this map; the caller tree says this map's sort is about 0.9%, and the measured
    // change was −3.03% [−4.62%, −1.62%]. The 34% was mostly
    // `classify::classify_path_with_prefix`. Cache-only restore now walks file
    // identities without classifying (`restore_analysis_candidates`); the sidecar
    // already stores the classification that would have replaced the live result.
    let candidates_started = crate::counters::enabled().then(std::time::Instant::now);
    let (mut candidates, visited) = index.restore_analysis_candidates(wanted.analysis);
    crate::counters::add_elapsed(candidates_started, |counts, elapsed| {
        counts.content_sidecar_candidates_us =
            counts.content_sidecar_candidates_us.saturating_add(elapsed);
    });
    // Completeness is files visited, not unique `PathBuf` keys. Trailing-separator
    // aliases collapse in the map and would otherwise shrink the denominator.
    let mut loaded =
        ContentCacheLoad { usable: true, candidates: visited, ..ContentCacheLoad::default() };
    for _ in 0..stream.remaining {
        let decode_started = timings.as_ref().map(|_| Instant::now());
        let Some((relative_path, analysis)) = read_record(&mut stream) else {
            if let (Some(timings), Some(started)) = (&mut timings, decode_started) {
                timings.add_parse(started);
            }
            index.clear_content();
            if let Some(timings) = timings {
                timings.publish();
            }
            return Ok(ContentCacheLoad::default());
        };
        if let (Some(timings), Some(started)) = (&mut timings, decode_started) {
            timings.add_parse(started);
        }
        let Some(candidate) = candidates.remove(&relative_path) else {
            loaded.stale = loaded.stale.saturating_add(1);
            continue;
        };
        if candidate.attrs.fingerprint() != analysis.fingerprint {
            loaded.stale = loaded.stale.saturating_add(1);
            continue;
        }
        let coverage_exclusion =
            !matches!(analysis.coverage, CoverageReason::Analyzed | CoverageReason::Binary);
        let bytes = analysis.bytes;
        let apply_started = timings.as_ref().map(|_| Instant::now());
        let outcome = index.apply_restored_analysis(candidate, analysis);
        if let (Some(timings), Some(started)) = (&mut timings, apply_started) {
            timings.add_apply(started);
        }
        match outcome {
            AnalysisApplyOutcome::Applied => {
                loaded.hits = loaded.hits.saturating_add(1);
                loaded.bytes = loaded.bytes.saturating_add(bytes);
                loaded.coverage_exclusions =
                    loaded.coverage_exclusions.saturating_add(u64::from(coverage_exclusion));
            }
            AnalysisApplyOutcome::Stale => loaded.stale = loaded.stale.saturating_add(1),
        }
    }
    if !stream.reader.is_empty() {
        index.clear_content();
        if let Some(timings) = timings {
            timings.publish();
        }
        return Ok(ContentCacheLoad::default());
    }
    let rebuild_started = timings.as_ref().map(|_| Instant::now());
    index.rebuild_content_rollups();
    if let (Some(timings), Some(started)) = (&mut timings, rebuild_started) {
        timings.add_apply(started);
    }
    if let Some(timings) = timings {
        timings.publish();
    }
    Ok(loaded)
}

/// The size of the content sidecar at `path`, when fdu wrote the file there.
///
/// Decided by the magic alone, deliberately not by the integrity check: a truncated or
/// older-format sidecar is still fdu's, and it goes when its snapshot goes. Only a regular
/// file is opened, so a symbolic link is never followed out of the cache directory.
pub(crate) fn content_sidecar_bytes(path: &Path) -> Result<Option<u64>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::io(path, error)),
    };
    if !metadata.file_type().is_file() {
        return Ok(None);
    }
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::io(path, error)),
    };
    let mut magic = [0u8; MAGIC.len()];
    match file.read_exact(&mut magic) {
        Ok(()) => Ok((&magic == MAGIC).then_some(metadata.len())),
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
        Err(error) => Err(Error::io(path, error)),
    }
}

/// Identify the content sidecar at `path` by its contents, or return `None` when no sidecar
/// fdu wrote is there.
///
/// Mirrors [`crate::snapshot::identify`]. The magic alone decides whether the file is fdu's
/// sidecar, as [`content_sidecar_bytes`] decides it for pairing and clearing; the rest of
/// the header decides whether this build can serve it. Every sidecar format puts its
/// version after the magic, and format 5 put the engine fingerprint after the version, so
/// a sidecar from another release is recognized as stale rather than mistaken for a
/// foreign file. Only the bounded header and the trailer are read, never the records, and
/// only a regular file is opened, so a symbolic link is never followed out of the cache
/// directory.
pub(crate) fn identify_sidecar(path: &Path) -> Result<Option<crate::cache::ContentStatus>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::io(path, error)),
    };
    if !metadata.file_type().is_file() {
        return Ok(None);
    }
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::io(path, error)),
    };
    let state = identify_sidecar_contents(&mut file).map_err(|error| Error::io(path, error))?;
    Ok(state.map(|state| crate::cache::ContentStatus { bytes: metadata.len(), state }))
}

/// Classify a sidecar from its header and trailer, or `None` when it lacks the sidecar
/// magic. Only an I/O failure is an error; every malformed byte is an answer.
fn identify_sidecar_contents(
    file: &mut fs::File,
) -> io::Result<Option<crate::cache::ContentState>> {
    use crate::cache::{ContentInfo, ContentState, StaleReason};

    let stale = |reason| Ok(Some(ContentState::Stale(reason)));
    let trailer_intact = sidecar_trailer_intact(file)?;
    file.seek(SeekFrom::Start(0))?;
    let mut header = Vec::new();
    if !read_more(file, &mut header, MAGIC.len())? || header != MAGIC {
        return Ok(None);
    }
    if !read_more(file, &mut header, 4)? {
        return stale(StaleReason::Unreadable);
    }
    let version = u32::from_le_bytes(header[MAGIC.len()..].try_into().expect("four bytes"));
    match version.cmp(&FORMAT_VERSION) {
        std::cmp::Ordering::Less => return stale(StaleReason::OlderFormat { version }),
        std::cmp::Ordering::Greater => return stale(StaleReason::NewerFormat { version }),
        std::cmp::Ordering::Equal => {}
    }
    if !read_more(file, &mut header, 8)? {
        return stale(StaleReason::Unreadable);
    }
    let engine = u64::from_le_bytes(header[MAGIC.len() + 4..].try_into().expect("eight bytes"));
    if engine != crate::snapshot::engine_fingerprint() {
        return stale(StaleReason::OtherEngine);
    }
    // Truncation removes the tail and leaves the header readable, so a header-only check
    // would call a half-written sidecar current.
    if !trailer_intact || !read_sidecar_header(file, &mut header)? {
        return stale(StaleReason::Unreadable);
    }
    let parsed = (|| {
        let mut reader = Reader::new(header.get(MAGIC.len() + 4 + 8..)?);
        if reader.u8()? != crate::snapshot::path_encoding() {
            return None;
        }
        let identity = read_identity(&mut reader, engine)?;
        reader.os_string()?;
        let records = reader.u64()?;
        (records <= MAX_RECORDS && reader.is_empty()).then_some(ContentInfo { identity, records })
    })();
    Ok(Some(parsed.map_or(ContentState::Stale(StaleReason::Unreadable), ContentState::Current)))
}

/// Read the header fields after a format-5 prologue into `header`: each field's length is
/// bounded before it is read. `false` when the file ends first or a bound is exceeded.
fn read_sidecar_header(file: &mut fs::File, header: &mut Vec<u8>) -> io::Result<bool> {
    // The path encoding, the entry tier (which holds the type rules), the analyzer set, the
    // options fingerprint, and the analyzer count.
    if !read_more(file, header, 1 + ENTRY_TIER_BYTES + 1 + 8 + 1)? {
        return Ok(false);
    }
    let analyzers = usize::from(*header.last().expect("the analyzer count"));
    if analyzers > MAX_ANALYZERS {
        return Ok(false);
    }
    for _ in 0..analyzers {
        let Some(length) = read_length(file, header, MAX_ANALYZER_ID_BYTES)? else {
            return Ok(false);
        };
        if !read_more(file, header, length + 2)? {
            return Ok(false);
        }
    }
    let Some(root) = read_length(file, header, MAX_PATH_BYTES)? else { return Ok(false) };
    Ok(read_more(file, header, root)? && read_more(file, header, 8)?)
}

/// Read a four-byte length into `header` and return it, when the file holds it and it is at
/// most `max`.
fn read_length(file: &mut fs::File, header: &mut Vec<u8>, max: usize) -> io::Result<Option<usize>> {
    if !read_more(file, header, 4)? {
        return Ok(None);
    }
    let bytes = header[header.len() - 4..].try_into().expect("four bytes");
    Ok(usize::try_from(u32::from_le_bytes(bytes)).ok().filter(|length| *length <= max))
}

/// Append exactly `count` more bytes of `file` to `buffer`, or return `false` when it ends
/// first.
fn read_more(file: &mut fs::File, buffer: &mut Vec<u8>, count: usize) -> io::Result<bool> {
    let start = buffer.len();
    buffer.resize(start + count, 0);
    match file.read_exact(&mut buffer[start..]) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
            buffer.truncate(start);
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

/// Whether a file ends with the sidecar trailer, which is what a truncated write destroys.
fn sidecar_trailer_intact(file: &mut fs::File) -> io::Result<bool> {
    let footer = u64::try_from(CHECKSUM_BYTES + TRAILER.len()).expect("a small footer");
    if file.metadata()?.len() < footer {
        return Ok(false);
    }
    file.seek(SeekFrom::End(-i64::try_from(TRAILER.len()).expect("a small trailer")))?;
    let mut trailer = [0u8; TRAILER.len()];
    match file.read_exact(&mut trailer) {
        Ok(()) => Ok(&trailer == TRAILER),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(error),
    }
}

fn put_record(buffer: &mut Vec<u8>, path: &Path, record: &FileAnalysis) -> Result<()> {
    crate::snapshot::put_os_str(buffer, path.as_os_str())?;
    put_fingerprint(buffer, record.fingerprint);
    buffer.extend_from_slice(&record.bytes.to_le_bytes());
    put_bounded_bytes(buffer, record.classification.file_type.as_str().as_bytes(), MAX_TYPE_BYTES)?;
    buffer.push(family_code(record.classification.family));
    buffer.push(source_code(record.classification.source));
    buffer.push(confidence_code(record.classification.confidence));
    buffer.push(flags_code(record.classification.flags));
    put_metrics(buffer, record.metrics);
    buffer.push(coverage_code(record.coverage));
    put_bounded_bytes(buffer, record.error.as_deref().unwrap_or("").as_bytes(), MAX_ERROR_BYTES)
}

/// Write the content tier identity after the prologue and path encoding: the entry tier's
/// fixed-width fields, which hold the type rules, then the analyzer set, the options
/// fingerprint, and the analyzers.
fn put_identity(buffer: &mut Vec<u8>, identity: &ContentTierIdentity) -> Result<()> {
    buffer.extend_from_slice(&identity.entries.encode());
    put_profile(buffer, identity.analysis);
    buffer.extend_from_slice(&identity.provenance.options_fingerprint.0.to_le_bytes());
    put_analyzers(buffer, &identity.provenance.analyzers)
}

/// Read what [`put_identity`] wrote, under the prologue's `engine` fingerprint.
fn read_identity(reader: &mut Reader<'_>, engine: u64) -> Option<ContentTierIdentity> {
    let entries =
        EntryTierIdentity::decode(engine, reader.take(ENTRY_TIER_BYTES)?.try_into().ok()?)?;
    Some(ContentTierIdentity {
        entries,
        analysis: read_profile(reader.u8()?)?,
        provenance: AnalyzerProvenance {
            options_fingerprint: super::OptionsFingerprint(reader.u64()?),
            analyzers: read_analyzers(reader)?,
        },
    })
}

/// Header-only sidecar parse. Records are decoded one at a time by [`read_record`].
struct RecordStream<'a> {
    reader: Reader<'a>,
    remaining: u64,
    profile: AnalysisSet,
    provenance: ContentProvenance,
}

/// Parse a sidecar for `root` whose content tier identity equals `wanted`.
fn parse_header<'a>(
    image: &'a [u8],
    root: &Path,
    wanted: &ContentTierIdentity,
) -> Option<RecordStream<'a>> {
    let payload = integrity_payload(image)?;
    let mut reader = Reader::new(payload.get(MAGIC.len()..)?);
    if reader.u32()? != FORMAT_VERSION {
        return None;
    }
    let engine = reader.u64()?;
    if reader.u8()? != crate::snapshot::path_encoding() {
        return None;
    }
    // Records of any other identity answer another request: a miss, whether they came
    // from another engine or entry tier, which holds their type rules, or another analyzer
    // set, version, or option.
    let identity = read_identity(&mut reader, engine)?;
    if identity != *wanted {
        return None;
    }
    let (profile, provenance) = (identity.analysis, identity.record_provenance());
    if reader.os_string()?.as_os_str() != root.as_os_str() {
        return None;
    }
    let count = reader.u64()?;
    if count > MAX_RECORDS {
        return None;
    }
    Some(RecordStream { reader, remaining: count, profile, provenance })
}

fn read_record(stream: &mut RecordStream<'_>) -> Option<(PathBuf, FileAnalysis)> {
    let relative_path = PathBuf::from(stream.reader.os_string()?);
    if !record_path_stays_inside_root(&relative_path) {
        return None;
    }
    let fingerprint = read_fingerprint(&mut stream.reader)?;
    let bytes = stream.reader.u64()?;
    let file_type = String::from_utf8(stream.reader.bytes(MAX_TYPE_BYTES)?).ok()?;
    if file_type.is_empty() {
        return None;
    }
    let classification = Classification {
        file_type: FileTypeId::from_cache(file_type),
        family: read_family(stream.reader.u8()?)?,
        source: read_source(stream.reader.u8()?)?,
        confidence: read_confidence(stream.reader.u8()?)?,
        flags: read_flags(stream.reader.u8()?)?,
    };
    let metrics = read_metrics(&mut stream.reader)?;
    let coverage = read_coverage(stream.reader.u8()?)?;
    let error = String::from_utf8(stream.reader.bytes(MAX_ERROR_BYTES)?).ok()?;
    stream.remaining = stream.remaining.saturating_sub(1);
    Some((
        relative_path,
        FileAnalysis {
            classification,
            fingerprint,
            bytes,
            profile: stream.profile,
            provenance: stream.provenance.clone(),
            metrics,
            coverage,
            error: (!error.is_empty()).then_some(error),
        },
    ))
}

/// Whether a sidecar record's path is relative and never ascends, so it names an entry
/// under the root the sidecar claims.
///
/// The sidecar is untrusted input: anything on disk can have written it. So the question
/// is asked of components, where every one must be `Normal` or `CurDir`, rather than of
/// `is_absolute`, which answers it wrongly. `..` is not absolute on any platform, and on
/// Windows neither is a rooted path with no drive (`\x`) nor a drive-relative one
/// (`C:x`), yet each names a path outside the root.
///
/// Private and stated here rather than borrowed from the index, whose own path
/// validation is free to change shape: this guard's contract is the untrusted image.
fn record_path_stays_inside_root(path: &Path) -> bool {
    path.components().all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn integrity_payload(image: &[u8]) -> Option<&[u8]> {
    let footer = CHECKSUM_BYTES.checked_add(TRAILER.len())?;
    if image.len() < MAGIC.len() + footer || image.get(..MAGIC.len())? != MAGIC {
        return None;
    }
    let payload_len = image.len().checked_sub(footer)?;
    if image.get(payload_len + CHECKSUM_BYTES..)? != TRAILER {
        return None;
    }
    let expected_checksum =
        u32::from_le_bytes(image.get(payload_len..payload_len + CHECKSUM_BYTES)?.try_into().ok()?);
    let payload = image.get(..payload_len)?;
    (crate::snapshot::crc32c(payload) == expected_checksum).then_some(payload)
}

fn put_fingerprint(buffer: &mut Vec<u8>, value: Fingerprint) {
    buffer.extend_from_slice(&value.size.to_le_bytes());
    buffer.extend_from_slice(&value.mtime_ns.to_le_bytes());
    buffer.extend_from_slice(&value.ctime_ns.to_le_bytes());
    buffer.extend_from_slice(&value.inode.to_le_bytes());
    buffer.extend_from_slice(&value.dev.to_le_bytes());
}

fn read_fingerprint(reader: &mut Reader<'_>) -> Option<Fingerprint> {
    Some(Fingerprint {
        size: reader.u64()?,
        mtime_ns: reader.i64()?,
        ctime_ns: reader.i64()?,
        inode: reader.u64()?,
        dev: reader.u64()?,
    })
}

fn put_metrics(buffer: &mut Vec<u8>, value: MetricValues) {
    for metric in [
        value.physical_lines,
        value.blank_lines,
        value.nonblank_lines,
        value.raw_words,
        value.code_lines,
        value.comment_lines,
        value.code_blank_lines,
        value.paragraphs,
        value.visible_words,
        value.logical_word_stats.wide_chars,
        value.logical_word_stats.nonwide_tokens,
        value.logical_word_stats.nonwide_chars,
        value.visible_logical_word_stats.wide_chars,
        value.visible_logical_word_stats.nonwide_tokens,
        value.visible_logical_word_stats.nonwide_chars,
    ] {
        buffer.extend_from_slice(&metric.to_le_bytes());
    }
}

fn read_metrics(reader: &mut Reader<'_>) -> Option<MetricValues> {
    Some(MetricValues {
        physical_lines: reader.u64()?,
        blank_lines: reader.u64()?,
        nonblank_lines: reader.u64()?,
        raw_words: reader.u64()?,
        code_lines: reader.u64()?,
        comment_lines: reader.u64()?,
        code_blank_lines: reader.u64()?,
        paragraphs: reader.u64()?,
        visible_words: reader.u64()?,
        logical_word_stats: LogicalWordStats {
            wide_chars: reader.u64()?,
            nonwide_tokens: reader.u64()?,
            nonwide_chars: reader.u64()?,
        },
        visible_logical_word_stats: LogicalWordStats {
            wide_chars: reader.u64()?,
            nonwide_tokens: reader.u64()?,
            nonwide_chars: reader.u64()?,
        },
    })
}

fn put_analyzers(buffer: &mut Vec<u8>, analyzers: &[(AnalyzerId, AnalyzerVersion)]) -> Result<()> {
    let count = u8::try_from(analyzers.len())
        .map_err(|_| Error::Snapshot("too many content analyzers".into()))?;
    buffer.push(count);
    for (id, version) in analyzers {
        put_bounded_bytes(buffer, id.0.as_bytes(), MAX_ANALYZER_ID_BYTES)?;
        buffer.extend_from_slice(&version.0.to_le_bytes());
    }
    Ok(())
}

fn read_analyzers(reader: &mut Reader<'_>) -> Option<Vec<(AnalyzerId, AnalyzerVersion)>> {
    let count = usize::from(reader.u8()?);
    if count > MAX_ANALYZERS {
        return None;
    }
    let mut analyzers = Vec::with_capacity(count);
    for _ in 0..count {
        let id = String::from_utf8(reader.bytes(MAX_ANALYZER_ID_BYTES)?).ok()?;
        let known = match id.as_str() {
            "content-basic-v1" => super::CONTENT_BASIC,
            "code-sloc-v1" => super::CODE_SLOC,
            "text-logical-v1" => super::TEXT_LOGICAL,
            "markdown-prose-v1" => super::MARKDOWN_PROSE,
            _ => return None,
        };
        analyzers.push((known, AnalyzerVersion(reader.u16()?)));
    }
    Some(analyzers)
}

fn put_bounded_bytes(buffer: &mut Vec<u8>, bytes: &[u8], max: usize) -> Result<()> {
    if bytes.len() > max {
        return Err(Error::Snapshot("content sidecar string exceeds limit".into()));
    }
    let length = u32::try_from(bytes.len())
        .map_err(|_| Error::Snapshot("content sidecar string length overflow".into()))?;
    buffer.extend_from_slice(&length.to_le_bytes());
    buffer.extend_from_slice(bytes);
    Ok(())
}

fn put_profile(buffer: &mut Vec<u8>, profile: AnalysisSet) {
    buffer.push(profile.bits());
}

fn read_profile(code: u8) -> Option<AnalysisSet> {
    AnalysisSet::from_bits(code)
}

fn family_code(value: ContentFamily) -> u8 {
    match value {
        ContentFamily::Code => 0,
        ContentFamily::Prose => 1,
        ContentFamily::Markup => 2,
        ContentFamily::Data => 3,
        ContentFamily::Binary => 4,
        ContentFamily::Unknown => 5,
    }
}

fn read_family(code: u8) -> Option<ContentFamily> {
    match code {
        0 => Some(ContentFamily::Code),
        1 => Some(ContentFamily::Prose),
        2 => Some(ContentFamily::Markup),
        3 => Some(ContentFamily::Data),
        4 => Some(ContentFamily::Binary),
        5 => Some(ContentFamily::Unknown),
        _ => None,
    }
}

fn source_code(value: DetectionSource) -> u8 {
    match value {
        DetectionSource::ExactFilename => 0,
        DetectionSource::CompoundExtension => 1,
        DetectionSource::Extension => 2,
        DetectionSource::Shebang => 3,
        DetectionSource::ContentProbe => 4,
        DetectionSource::Unknown => 5,
        DetectionSource::Modeline => 6,
        DetectionSource::AmbiguousContent => 7,
        DetectionSource::FormatSignature => 8,
    }
}

fn read_source(code: u8) -> Option<DetectionSource> {
    match code {
        0 => Some(DetectionSource::ExactFilename),
        1 => Some(DetectionSource::CompoundExtension),
        2 => Some(DetectionSource::Extension),
        3 => Some(DetectionSource::Shebang),
        4 => Some(DetectionSource::ContentProbe),
        5 => Some(DetectionSource::Unknown),
        6 => Some(DetectionSource::Modeline),
        7 => Some(DetectionSource::AmbiguousContent),
        8 => Some(DetectionSource::FormatSignature),
        _ => None,
    }
}

fn flags_code(value: ClassificationFlags) -> u8 {
    u8::from(value.generated)
        | (u8::from(value.vendored) << 1)
        | (u8::from(value.documentation) << 2)
}

fn read_flags(code: u8) -> Option<ClassificationFlags> {
    (code & !0b111 == 0).then_some(ClassificationFlags {
        generated: code & 0b001 != 0,
        vendored: code & 0b010 != 0,
        documentation: code & 0b100 != 0,
    })
}

fn confidence_code(value: DetectionConfidence) -> u8 {
    match value {
        DetectionConfidence::Certain => 0,
        DetectionConfidence::High => 1,
        DetectionConfidence::Heuristic => 2,
    }
}

fn read_confidence(code: u8) -> Option<DetectionConfidence> {
    match code {
        0 => Some(DetectionConfidence::Certain),
        1 => Some(DetectionConfidence::High),
        2 => Some(DetectionConfidence::Heuristic),
        _ => None,
    }
}

fn coverage_code(value: CoverageReason) -> u8 {
    match value {
        CoverageReason::Analyzed => 0,
        CoverageReason::Binary => 1,
        CoverageReason::InvalidUtf8 => 2,
        CoverageReason::Unsupported => 4,
        CoverageReason::IoError => 5,
        CoverageReason::ChangedDuringRead => 6,
    }
}

fn read_coverage(code: u8) -> Option<CoverageReason> {
    match code {
        0 => Some(CoverageReason::Analyzed),
        1 => Some(CoverageReason::Binary),
        2 => Some(CoverageReason::InvalidUtf8),
        4 => Some(CoverageReason::Unsupported),
        5 => Some(CoverageReason::IoError),
        6 => Some(CoverageReason::ChangedDuringRead),
        _ => None,
    }
}

struct Reader<'a> {
    remaining: &'a [u8],
}

impl<'a> Reader<'a> {
    fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }

    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let (value, rest) = self.remaining.split_at_checked(count)?;
        self.remaining = rest;
        Some(value)
    }

    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|value| value[0])
    }

    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn i64(&mut self) -> Option<i64> {
        Some(i64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn bytes(&mut self, max: usize) -> Option<Vec<u8>> {
        let length = usize::try_from(self.u32()?).ok()?;
        (length <= max).then(|| self.take(length).map(<[u8]>::to_vec)).flatten()
    }

    fn os_string(&mut self) -> Option<OsString> {
        let bytes = self.bytes(MAX_PATH_BYTES)?;
        #[cfg(unix)]
        {
            Some(decode_os_string(&bytes))
        }
        #[cfg(not(unix))]
        {
            decode_os_string(&bytes)
        }
    }

    fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}

#[cfg(unix)]
fn decode_os_string(bytes: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(bytes.to_vec())
}

#[cfg(windows)]
fn decode_os_string(bytes: &[u8]) -> Option<OsString> {
    use std::os::windows::ffi::OsStringExt;
    // `usize::is_multiple_of` is stable since 1.87 and this crate's MSRV is 1.85, so a
    // Windows user on the declared minimum could not build it. Nothing caught that
    // because the MSRV job runs on ubuntu, where this function does not exist.
    if bytes.len() % 2 != 0 {
        return None;
    }
    let units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    Some(OsString::from_wide(&units))
}

#[cfg(not(any(unix, windows)))]
fn decode_os_string(bytes: &[u8]) -> Option<OsString> {
    String::from_utf8(bytes.to_vec()).ok().map(OsString::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use crate::scan::ScanConfig;

    fn analyzed_index() -> (tempfile::TempDir, Index, AnalysisRequest) {
        let root = tempfile::tempdir().expect("root");
        fs::write(root.path().join("notes.md"), "<!-- @generated -->\none two\n").expect("write");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        let request = AnalysisRequest {
            profile: AnalysisSet::NONE.with_lines(),
            ..AnalysisRequest::default()
        };
        super::super::analyze_index(&mut index, request);
        (root, index, request)
    }

    /// A prose file and a code file, so a `code`-only request is genuinely narrower than
    /// `all` rather than accidentally equivalent on this fixture.
    ///
    /// The sidecar lives outside the scanned tree: writing it inside would make the cache
    /// file itself an unanalyzed candidate and quietly defeat any "opened no file" claim.
    fn containment_fixture(profile: AnalysisSet) -> (tempfile::TempDir, tempfile::TempDir, Index) {
        let root = tempfile::tempdir().expect("root");
        let cache_dir = tempfile::tempdir().expect("cache dir");
        fs::write(root.path().join("notes.md"), "<!-- @generated -->\none two\n").expect("write");
        fs::write(root.path().join("main.rs"), "fn main() {\n    // hi\n}\n").expect("write");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        super::super::analyze_index(&mut index, request_for(profile));
        (root, cache_dir, index)
    }

    fn request_for(profile: AnalysisSet) -> AnalysisRequest {
        AnalysisRequest { profile, ..AnalysisRequest::default() }
    }

    /// Load the sidecar at `cache` for `request`'s identity in `index`.
    fn load(index: &mut Index, request: AnalysisRequest, cache: &Path) -> ContentCacheLoad {
        let wanted = index.content_identity(request.profile);
        load_content_cache(index, &wanted, cache).expect("load")
    }

    #[test]
    fn fractional_record_durations_are_converted_after_accumulation() {
        let mut total = Duration::ZERO;
        for _ in 0..1_000 {
            add_duration(&mut total, Duration::from_nanos(900));
        }

        assert_eq!(duration_micros(total), 900);
        assert_eq!(duration_micros(Duration::from_nanos(900)), 0);
    }

    /// Restore rebuilds nested directory roll-ups, not only the root.
    ///
    /// The probe content digest hashes the root roll-up; the `ContentIndex` unit test is
    /// the in-memory H115 check. This is the sidecar-boundary half of that claim.
    #[test]
    fn sidecar_restore_rebuilds_nested_directory_rollups() {
        let root = tempfile::tempdir().expect("root");
        let cache_dir = tempfile::tempdir().expect("cache dir");
        fs::create_dir_all(root.path().join("a/b")).expect("dirs");
        fs::write(root.path().join("notes.md"), "one two\n").expect("write");
        fs::write(root.path().join("a/keep.rs"), "fn keep() {}\n").expect("write");
        fs::write(root.path().join("a/b/nested.rs"), "fn nested() {}\n").expect("write");
        let request = request_for(AnalysisSet::NONE.with_lines());
        let (mut analyzed, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        super::super::analyze_index(&mut analyzed, request);
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&analyzed, &cache).expect("save");

        let (mut restored, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        let loaded = load(&mut restored, request, &cache);
        assert!(loaded.usable && loaded.hits == 3, "{loaded:?}");
        for dir in ["", "a", "a/b"] {
            assert_eq!(
                restored.content_rollup(Path::new(dir)),
                analyzed.content_rollup(Path::new(dir)),
                "{dir:?} roll-up"
            );
        }
    }

    /// A sidecar serves exactly the analyzer set it was written for. A wider one holds
    /// metrics the narrower request did not ask for and would report them, and the wider
    /// set's label, as its answer; so it is a clean miss, and every file is read again
    /// under the narrower set, as a cold run reads it.
    #[test]
    fn a_wider_sidecar_is_a_clean_miss_for_a_narrower_request() {
        let (root, cache_dir, index) = containment_fixture(AnalysisSet::ALL);
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&index, &cache).expect("save");

        let narrower = request_for(AnalysisSet::NONE.with_code());
        let (mut restored, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        let loaded = load(&mut restored, narrower, &cache);

        assert_eq!(loaded, ContentCacheLoad::default(), "a wider sidecar must miss");
        assert_eq!(
            restored.pending_analysis_candidates(narrower).len(),
            2,
            "a missed request reads every file"
        );
        assert!(restored.content().is_none(), "a miss leaves no content tier behind");
    }

    /// Neither a narrower sidecar nor one of an incomparable set answers a request: each
    /// lacks metrics the request needs or holds ones it did not ask for.
    #[test]
    fn another_analyzer_set_is_a_clean_miss() {
        let code = AnalysisSet::NONE.with_code();
        let words = AnalysisSet::NONE.with_words();
        for (stored, wanted) in [(code, AnalysisSet::ALL), (code, words), (words, code)] {
            let (root, cache_dir, index) = containment_fixture(stored);
            let cache = cache_dir.path().join("content.cache");
            save_content_cache(&index, &cache).expect("save");

            let (mut restored, _) =
                crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
            assert_eq!(
                load(&mut restored, request_for(wanted), &cache),
                ContentCacheLoad::default(),
                "a {stored:?} sidecar must miss a {wanted:?} request"
            );
            let (mut same, _) =
                crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
            let hit = load(&mut same, request_for(stored), &cache);
            assert!(hit.usable && hit.hits == 2, "its own set still restores: {hit:?}");
        }
    }

    /// One sidecar per root holds one analyzer set, so answering another set replaces it.
    /// That costs the wider set's next run a re-read, never its correctness.
    #[test]
    fn a_different_analyzer_set_replaces_the_sidecar() {
        let (root, cache_dir, index) = containment_fixture(AnalysisSet::ALL);
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&index, &cache).expect("save");

        let narrower = request_for(AnalysisSet::NONE.with_code());
        let (mut restored, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        load(&mut restored, narrower, &cache);
        let analysis = super::super::analyze_index(&mut restored, narrower);
        assert_eq!(analysis.applied, 2, "the narrower run reads every file");
        save_content_cache(&restored, &cache).expect("resave");

        let (mut wide, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert_eq!(
            load(&mut wide, request_for(AnalysisSet::ALL), &cache),
            ContentCacheLoad::default(),
            "the wider set was replaced"
        );
        let (mut narrow, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        let hit = load(&mut narrow, narrower, &cache);
        assert!(hit.usable && hit.hits == 2, "by the narrower one: {hit:?}");
    }

    /// Byte offset of the engine fingerprint: after the magic and the format version, where
    /// a snapshot's prologue puts it.
    const ENGINE_OFFSET: usize = MAGIC.len() + 4;

    /// Byte offset of the path encoding, which follows the engine fingerprint.
    const PATH_ENCODING_OFFSET: usize = ENGINE_OFFSET + 8;

    /// Recompute a rewritten image's checksum, so the rewrite is the only thing wrong with it.
    fn reseal(image: &mut [u8]) {
        let payload_len = image.len() - CHECKSUM_BYTES - TRAILER.len();
        let checksum = crate::snapshot::crc32c(&image[..payload_len]);
        image[payload_len..payload_len + CHECKSUM_BYTES].copy_from_slice(&checksum.to_le_bytes());
    }

    #[test]
    fn corruption_is_a_clean_miss() {
        let (root, index, request) = analyzed_index();
        let cache = root.path().join("content.cache");
        save_content_cache(&index, &cache).expect("save");
        let mut bytes = fs::read(&cache).expect("read");
        assert_eq!(bytes[PATH_ENCODING_OFFSET], crate::snapshot::path_encoding());
        // A header byte flipped without resealing: the checksum no longer matches.
        bytes[PATH_ENCODING_OFFSET] ^= 0xff;
        fs::write(&cache, bytes).expect("corrupt");
        let (mut restored, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert_eq!(load(&mut restored, request, &cache), ContentCacheLoad::default());
    }

    /// A sidecar's records answer only the engine and entry tier they were analyzed under.
    /// Another engine's sidecar, one of another format, and one analyzed over another scope
    /// are clean misses even though every record's path and fingerprint still match; one
    /// analyzed with `.gitignore` observation off answers a request with it on, because no
    /// metric depends on observation.
    #[test]
    fn a_sidecar_from_another_engine_or_scope_is_a_clean_miss() {
        let (root, index, request) = analyzed_index();
        let cache_dir = tempfile::tempdir().expect("cache dir");
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&index, &cache).expect("save");
        let saved = fs::read(&cache).expect("read");
        let load_into = |config: &ScanConfig| {
            let (mut restored, _) =
                crate::scan::scan_into_index(root.path(), config).expect("scan");
            load(&mut restored, request, &cache)
        };
        let hit = load_into(&ScanConfig::default());
        assert!(hit.usable && hit.hits == 1, "the saved sidecar restores: {hit:?}");

        let mut other_engine = saved.clone();
        for byte in &mut other_engine[ENGINE_OFFSET..ENGINE_OFFSET + 8] {
            *byte = !*byte;
        }
        reseal(&mut other_engine);
        let mut older_format = saved.clone();
        older_format[MAGIC.len()..ENGINE_OFFSET].copy_from_slice(&4_u32.to_le_bytes());
        reseal(&mut older_format);
        for (name, image) in [("another engine", other_engine), ("format 4", older_format)] {
            fs::write(&cache, image).expect("rewrite");
            assert_eq!(load_into(&ScanConfig::default()), ContentCacheLoad::default(), "{name}");
        }

        fs::write(&cache, &saved).expect("restore");
        for config in [
            ScanConfig { max_depth: Some(4), ..ScanConfig::default() },
            ScanConfig { exclude_special: true, ..ScanConfig::default() },
        ] {
            assert_eq!(load_into(&config), ContentCacheLoad::default(), "{:?}", config.scope());
        }
        let blind = ScanConfig { read_controls: false, ..ScanConfig::default() };
        assert_eq!(load_into(&blind), hit, "observation is not part of the content identity");
    }

    /// The sidecar reads its entry tier through the shared codec, so a sealed image whose
    /// entry tier holds a byte no encoder writes is a clean miss rather than an identity
    /// that happens to compare unequal.
    #[test]
    fn a_sidecar_entry_tier_no_encoder_writes_is_a_clean_miss() {
        let (root, index, request) = analyzed_index();
        let cache_dir = tempfile::tempdir().expect("cache dir");
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&index, &cache).expect("save");
        let saved = fs::read(&cache).expect("read");
        let reload = || {
            let (mut restored, _) =
                crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
            load(&mut restored, request, &cache)
        };
        assert!(reload().usable, "the saved sidecar restores");

        // The entry tier follows the path encoding; its scope flags follow the depth bound.
        let flags_at = PATH_ENCODING_OFFSET + 1 + crate::stored_state::BOUND_BYTES;
        let mut unknown_flag = saved;
        assert_eq!(unknown_flag[flags_at], 0, "the default scope sets no flag");
        unknown_flag[flags_at] |= 1 << 7;
        reseal(&mut unknown_flag);
        fs::write(&cache, unknown_flag).expect("rewrite");
        assert_eq!(reload(), ContentCacheLoad::default());
    }

    /// Records answer only the analyzer versions and options they were counted under. A
    /// sidecar whose analyzers are another version, or whose options differ, is a clean
    /// miss even though its analyzer set, entries, and every record's fingerprint match.
    #[test]
    fn an_analyzer_version_change_invalidates_records() {
        let (root, index, request) = analyzed_index();
        let cache_dir = tempfile::tempdir().expect("cache dir");
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&index, &cache).expect("save");
        let saved = fs::read(&cache).expect("read");
        let reload = |cache: &Path| {
            let (mut restored, _) =
                crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
            load(&mut restored, request, cache)
        };
        let hit = reload(&cache);
        assert!(hit.usable && hit.hits == 1, "the saved sidecar restores: {hit:?}");

        // After the path encoding: the entry tier, which holds the type rules, the analyzer
        // set, the options fingerprint, the analyzer count, then the one analyzer's
        // length-prefixed id and its version.
        let options_at = PATH_ENCODING_OFFSET + 1 + ENTRY_TIER_BYTES + 1;
        let count_at = options_at + 8;
        let version_at = count_at + 1 + 4 + super::super::CONTENT_BASIC.0.len();
        assert_eq!(saved[count_at], 1, "a lines request runs one analyzer");
        assert_eq!(&saved[count_at + 5..version_at], super::super::CONTENT_BASIC.0.as_bytes());
        assert_eq!(saved[version_at..version_at + 2], 1_u16.to_le_bytes());

        let mut other_version = saved.clone();
        other_version[version_at..version_at + 2].copy_from_slice(&2_u16.to_le_bytes());
        reseal(&mut other_version);
        let mut other_options = saved.clone();
        other_options[options_at] ^= 1;
        reseal(&mut other_options);
        for (name, image) in [("another version", other_version), ("other options", other_options)]
        {
            fs::write(&cache, image).expect("rewrite");
            assert_eq!(reload(&cache), ContentCacheLoad::default(), "{name}");
        }
        fs::write(&cache, &saved).expect("restore");
        assert_eq!(reload(&cache), hit, "the unchanged image still restores");
    }

    /// A save writes a record only for a file this pass verified. Records restored beside
    /// a snapshot over a subtree whose verification was withdrawn, here by a pass that
    /// began over `sub` and has not finished, describe retained facts nobody re-checked,
    /// so they stay out of the sidecar while records for the verified rest of the tree
    /// are written.
    #[test]
    fn records_under_an_unverified_subtree_are_not_written() {
        let root = tempfile::tempdir().expect("root");
        let store = tempfile::tempdir().expect("cache dir");
        fs::write(root.path().join("top.md"), "one two\n").expect("write");
        fs::create_dir(root.path().join("sub")).expect("mkdir");
        fs::write(root.path().join("sub").join("inner.md"), "three\n").expect("write");
        let config = ScanConfig::default();
        let lines = request_for(AnalysisSet::NONE.with_lines());
        let (snapshot_path, cache) = (store.path().join("tree.fdu"), store.path().join("first"));

        let (mut scanned, _) = crate::scan::scan_into_index(root.path(), &config).expect("scan");
        super::super::analyze_index(&mut scanned, lines);
        crate::snapshot::save(&scanned, &snapshot_path).expect("save snapshot");
        save_content_cache(&scanned, &cache).expect("save sidecar");

        let mut restored = crate::snapshot::load_with_types(&snapshot_path, config.types_shared())
            .expect("load")
            .expect("a usable snapshot");
        assert_eq!(load(&mut restored, lines, &cache).hits, 2);
        crate::scan::reconcile(&mut restored, &config, &mut |_| {}).expect("reconcile");
        restored.begin_reconcile(Path::new("sub")).expect("withdraw trust over sub");

        let content = restored.content().expect("content");
        let writable = |path: &str| {
            let record = content.file(Path::new(path)).expect("a restored record");
            crate::stored_state::content_record_writable(&restored, Path::new(path), record)
        };
        assert!(writable("top.md"), "a verified file's record is written");
        assert!(!writable("sub/inner.md"), "a record under an unverified subtree is not");

        let rewritten = store.path().join("second");
        save_content_cache(&restored, &rewritten).expect("resave");
        let (mut fresh, _) = crate::scan::scan_into_index(root.path(), &config).expect("scan");
        let loaded = load(&mut fresh, lines, &rewritten);
        assert!(loaded.usable && loaded.hits == 1, "only the verified record: {loaded:?}");
        let fresh_content = fresh.content().expect("content");
        assert!(fresh_content.file(Path::new("top.md")).is_some());
        assert!(fresh_content.file(Path::new("sub/inner.md")).is_none());
    }

    /// Re-address the sidecar's one record, leaving the image otherwise valid.
    ///
    /// The path is swapped in its stored encoding and the checksum recomputed, so the only
    /// thing wrong with the result is where the record claims to live.
    fn readdress_record(image: &[u8], from: &Path, to: &Path) -> Vec<u8> {
        let encode = |path: &Path| {
            let mut bytes = Vec::new();
            crate::snapshot::put_os_str(&mut bytes, path.as_os_str()).expect("encode");
            bytes
        };
        let (from, to) = (encode(from), encode(to));
        let payload = integrity_payload(image).expect("a valid sidecar");
        let at = payload.windows(from.len()).position(|window| window == from).expect("the path");
        assert_eq!(
            payload.windows(from.len()).rposition(|window| window == from),
            Some(at),
            "the record's path must appear once, or the rewrite is ambiguous"
        );
        let mut rewritten = [&payload[..at], to.as_slice(), &payload[at + from.len()..]].concat();
        let checksum = crate::snapshot::crc32c(&rewritten);
        rewritten.extend_from_slice(&checksum.to_le_bytes());
        rewritten.extend_from_slice(TRAILER);
        rewritten
    }

    fn two_record_sidecar() -> (tempfile::TempDir, tempfile::TempDir, AnalysisRequest, PathBuf) {
        let root = tempfile::tempdir().expect("root");
        let cache_dir = tempfile::tempdir().expect("cache dir");
        fs::write(root.path().join("a.md"), "one\n").expect("write first");
        fs::write(root.path().join("z.md"), "two\n").expect("write second");
        let request = request_for(AnalysisSet::NONE.with_lines());
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        super::super::analyze_index(&mut index, request);
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&index, &cache).expect("save");
        (root, cache_dir, request, cache)
    }

    fn assert_late_stream_miss_recovers(
        root: &tempfile::TempDir,
        request: AnalysisRequest,
        cache: &Path,
    ) {
        let (mut restored, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert_eq!(load(&mut restored, request, cache), ContentCacheLoad::default());
        assert!(restored.content().is_none(), "a late miss exposes no accepted prefix");
        assert!(
            restored.content_rollup(Path::new("")).is_none(),
            "a late miss exposes no roll-up from the accepted prefix"
        );

        let analyzed = super::super::analyze_index(&mut restored, request);
        assert_eq!(analyzed.applied, 2, "both files are reanalyzed after the miss");
        assert_eq!(restored.content().expect("reanalyzed content").len(), 2);
        assert_eq!(
            restored.content_rollup(Path::new("")).expect("rebuilt root roll-up").total.files,
            2
        );
    }

    #[test]
    fn malformed_second_record_rolls_back_the_valid_prefix() {
        let (root, _cache_dir, request, cache) = two_record_sidecar();
        let image = fs::read(&cache).expect("read");
        let malformed = readdress_record(&image, Path::new("z.md"), Path::new("../outside.md"));
        fs::write(&cache, malformed).expect("rewrite");

        assert_late_stream_miss_recovers(&root, request, &cache);
    }

    #[test]
    fn checksummed_trailing_bytes_roll_back_all_records() {
        let (root, _cache_dir, request, cache) = two_record_sidecar();
        let image = fs::read(&cache).expect("read");
        let mut payload = integrity_payload(&image).expect("valid sidecar").to_vec();
        payload.extend_from_slice(b"trailing");
        let checksum = crate::snapshot::crc32c(&payload);
        payload.extend_from_slice(&checksum.to_le_bytes());
        payload.extend_from_slice(TRAILER);
        fs::write(&cache, payload).expect("rewrite");

        assert_late_stream_miss_recovers(&root, request, &cache);
    }

    /// A sidecar is untrusted, and each record names a path under the root it claims.
    ///
    /// The guard used to ask `is_absolute`, which is the wrong question for "does this stay
    /// inside". `..` is not absolute on any platform, and on Windows neither is a rooted
    /// path with no drive (`\rooted`) or a drive-relative one (`C:relative`). A record that
    /// leaves the root now makes the whole sidecar a clean miss, like any other malformed
    /// image, while an ordinary relative record still restores.
    #[test]
    fn a_record_that_leaves_the_root_is_a_clean_miss() {
        // (record path, whether the sidecar restores)
        let mut cases =
            vec![("notes.md", true), ("../escape.md", false), ("nested/../../escape.md", false)];
        #[cfg(unix)]
        cases.push(("/absolute.md", false));
        #[cfg(windows)]
        cases.extend([
            (r"C:\absolute.md", false),
            (r"\rooted-without-drive.md", false),
            ("C:drive-relative.md", false),
        ]);

        for (record_path, restores) in cases {
            // The rule the parser asks, on the bare path: every component must be normal.
            assert_eq!(
                record_path_stays_inside_root(Path::new(record_path)),
                restores,
                "{record_path:?}"
            );

            let (root, index, request) = analyzed_index();
            let cache = root.path().join("content.cache");
            save_content_cache(&index, &cache).expect("save");
            let image = fs::read(&cache).expect("read");
            let rewritten = readdress_record(&image, Path::new("notes.md"), Path::new(record_path));
            fs::write(&cache, rewritten).expect("re-address");

            let (mut restored, _) =
                crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
            let loaded = load(&mut restored, request, &cache);
            if restores {
                assert!(loaded.usable, "{record_path:?} must restore: {loaded:?}");
                assert_eq!(loaded.hits, 1, "{record_path:?} must restore: {loaded:?}");
            } else {
                assert_eq!(
                    loaded,
                    ContentCacheLoad::default(),
                    "{record_path:?} leaves the root, so the sidecar must be a clean miss"
                );
            }
        }
    }

    #[test]
    fn sidecar_name_preserves_the_metadata_snapshot_name() {
        assert_eq!(content_cache_path(Path::new("tree.fdu")), PathBuf::from("tree.fdu.content"));
    }
}
