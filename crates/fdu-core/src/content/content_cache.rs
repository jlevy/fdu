//! Versioned content sidecar, independent of the metadata snapshot format.

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::classify::{
    Classification, ClassificationFlags, ContentFamily, DetectionConfidence, DetectionSource,
    FileTypeId,
};
use crate::{Error, Fingerprint, Index, Result};

use super::{
    AnalysisApplyOutcome, AnalysisObservation, AnalysisRequest, AnalysisSet, AnalyzerId,
    AnalyzerVersion, ContentProvenance, CoverageReason, FileAnalysis, LogicalWordStats,
    MetricValues,
};

const MAGIC: &[u8; 8] = b"FDUCTNT\0";
const TRAILER: &[u8; 8] = b"FDUCTEND";
const FORMAT_VERSION: u32 = 4;
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
}

/// Derive the content-sidecar path without changing the metadata snapshot name.
pub fn content_cache_path(snapshot_path: &Path) -> PathBuf {
    let mut name = snapshot_path.as_os_str().to_os_string();
    name.push(".content");
    PathBuf::from(name)
}

/// Persist the current profile's sparse records as a separately invalidated sidecar.
pub fn save_content_cache(index: &Index, request: AnalysisRequest, path: &Path) -> Result<()> {
    if !request.profile.is_enabled() {
        return Ok(());
    }
    // Persist what the derived tier actually holds, not what this request asked for.
    // A narrow request served from a wider cached set leaves the wider records in place,
    // and relabelling them on the way out would throw away analyzers the records still
    // carry — turning one narrow query into a permanent downgrade of the sidecar.
    let Some(content) = index.content() else {
        return Ok(());
    };
    let (Some(stored), Some(provenance)) = (content.profile(), content.provenance().cloned())
    else {
        return Ok(());
    };
    let records = content
        .records()
        .filter(|(_, record)| {
            record.provenance == provenance
                && record.profile == stored
                && !matches!(
                    record.coverage,
                    CoverageReason::IoError | CoverageReason::ChangedDuringRead
                )
        })
        .collect::<Vec<_>>();
    let record_count = u64::try_from(records.len())
        .map_err(|_| Error::Snapshot("content sidecar record count overflow".into()))?;
    if record_count > MAX_RECORDS {
        return Err(Error::Snapshot("content sidecar exceeds record limit".into()));
    }

    let mut buffer = Vec::new();
    buffer.extend_from_slice(MAGIC);
    buffer.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    buffer.push(crate::snapshot::path_encoding());
    put_profile(&mut buffer, stored);
    buffer.extend_from_slice(&provenance.type_rules_fingerprint.to_le_bytes());
    buffer.extend_from_slice(&provenance.options_fingerprint.0.to_le_bytes());
    put_analyzers(&mut buffer, &provenance.analyzers)?;
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

/// Restore matching records, returning a miss for absent, corrupt, foreign, or stale
/// sidecars.
pub fn load_content_cache(
    index: &mut Index,
    request: AnalysisRequest,
    path: &Path,
) -> Result<ContentCacheLoad> {
    if !request.profile.is_enabled() {
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
    let image = fs::read(path).map_err(|error| Error::io(path, error))?;
    let Some(records) = parse(&image, index.root_path(), request, index.types()) else {
        return Ok(ContentCacheLoad::default());
    };
    index.prepare_content_analysis(request);
    // Keyed by hash rather than by order. This map is only ever drained by lookup —
    // nothing iterates it — so its ordering was never observable, and ordering a
    // `PathBuf` costs more than it looks: `Ord` walks components, so building a tree
    // pays about log2(n) comparisons per insert and each one walks the paths again.
    //
    // Worth about 3%, not more. A flat callgrind profile of a warm 14,542-file open
    // shows `compare_components` at 34% of instructions and it is tempting to read that
    // as this map; the caller tree says this map's sort is about 0.9%, and the measured
    // change was −3.03% [−4.62%, −1.62%]. The 34% is mostly
    // `classify::classify_path_with_prefix`, which `Index::analysis_candidates` runs
    // over every file on every open — including a cache-only one, which then discards
    // the result in favour of the classification the sidecar already stored. That is
    // the real cost on this path and it is tracked separately (`fdu-926e`).
    let mut candidates = index
        .analysis_candidates(request.profile)
        .into_iter()
        .map(|candidate| (candidate.relative_path.clone(), candidate))
        .collect::<HashMap<_, _>>();
    let mut loaded = ContentCacheLoad { usable: true, ..ContentCacheLoad::default() };
    for (relative_path, analysis) in records {
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
        match index.apply_analysis(AnalysisObservation { candidate, analysis }) {
            AnalysisApplyOutcome::Applied => {
                loaded.hits = loaded.hits.saturating_add(1);
                loaded.bytes = loaded.bytes.saturating_add(bytes);
                loaded.coverage_exclusions =
                    loaded.coverage_exclusions.saturating_add(u64::from(coverage_exclusion));
            }
            AnalysisApplyOutcome::Stale => loaded.stale = loaded.stale.saturating_add(1),
        }
    }
    Ok(loaded)
}

pub(crate) fn is_recognized_content_cache(path: &Path) -> Result<bool> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(Error::io(path, error)),
    };
    if metadata.len() > MAX_CACHE_BYTES {
        return Ok(false);
    }
    let image = fs::read(path).map_err(|error| Error::io(path, error))?;
    Ok(integrity_payload(&image).is_some())
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

fn parse(
    image: &[u8],
    expected_root: &Path,
    request: AnalysisRequest,
    types: &crate::classify::TypeRegistry,
) -> Option<Vec<(PathBuf, FileAnalysis)>> {
    let payload = integrity_payload(image)?;
    let mut reader = Reader::new(payload.get(MAGIC.len()..)?);
    if reader.u32()? != FORMAT_VERSION || reader.u8()? != crate::snapshot::path_encoding() {
        return None;
    }
    let profile = read_profile(reader.u8()?)?;
    let provenance = ContentProvenance {
        type_rules_fingerprint: reader.u64()?,
        options_fingerprint: super::OptionsFingerprint(reader.u64()?),
        analyzers: read_analyzers(&mut reader)?,
    };
    if !provenance.satisfies(profile, request.profile, types.fingerprint()) {
        return None;
    }
    if reader.os_string()?.as_os_str() != expected_root.as_os_str() {
        return None;
    }
    let count = reader.u64()?;
    if count > MAX_RECORDS {
        return None;
    }
    let mut records = Vec::with_capacity(usize::try_from(count).ok()?);
    for _ in 0..count {
        let relative_path = PathBuf::from(reader.os_string()?);
        if relative_path.is_absolute() {
            return None;
        }
        let fingerprint = read_fingerprint(&mut reader)?;
        let bytes = reader.u64()?;
        let file_type = String::from_utf8(reader.bytes(MAX_TYPE_BYTES)?).ok()?;
        if file_type.is_empty() {
            return None;
        }
        // The group is resolved from the registry rather than stored: it is an index
        // into that registry, and a sidecar written under other rules has already been
        // rejected on the fingerprint above. Storing it would add a format field whose
        // only correct value is the one this lookup returns.
        let group = types.group_of_type(&file_type);
        let classification = Classification {
            file_type: FileTypeId::from_cache(file_type),
            family: read_family(reader.u8()?)?,
            source: read_source(reader.u8()?)?,
            confidence: read_confidence(reader.u8()?)?,
            flags: read_flags(reader.u8()?)?,
            group,
        };
        let metrics = read_metrics(&mut reader)?;
        let coverage = read_coverage(reader.u8()?)?;
        let error = String::from_utf8(reader.bytes(MAX_ERROR_BYTES)?).ok()?;
        records.push((
            relative_path,
            FileAnalysis {
                classification,
                fingerprint,
                bytes,
                profile,
                provenance: provenance.clone(),
                metrics,
                coverage,
                error: (!error.is_empty()).then_some(error),
            },
        ));
    }
    reader.is_empty().then_some(records)
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
    // `usize::is_multiple_of` is stable since 1.87, and this crate's MSRV was 1.85 when
    // this was written -- a Windows user on the declared minimum could not have built it,
    // and nothing caught that because the MSRV job runs on ubuntu, where this function
    // does not exist. The floor is 1.88 now, so the method is available and clippy, which
    // is MSRV-aware, asks for it. `make cross-lint` is what makes that reachable at all.
    if !bytes.len().is_multiple_of(2) {
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

    /// The regression this replaces: reuse tested `record.profile == request.profile`, so
    /// a sidecar holding every analyzer forced a complete re-read for a narrower request
    /// whose every metric it already carried.
    #[test]
    fn a_wider_sidecar_answers_a_narrower_request_without_reading_anything() {
        let (root, cache_dir, index) = containment_fixture(AnalysisSet::ALL);
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&index, request_for(AnalysisSet::ALL), &cache).expect("save");

        let narrower = request_for(AnalysisSet::NONE.with_code());
        let (mut restored, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        let loaded = load_content_cache(&mut restored, narrower, &cache).expect("load");

        assert!(loaded.usable, "a superset sidecar must be usable: {loaded:?}");
        assert_eq!(loaded.hits, 2, "both records restore: {loaded:?}");
        assert!(
            restored.pending_analysis_candidates(narrower).is_empty(),
            "a satisfied request must open no file"
        );
    }

    /// Containment has a direction. A narrower sidecar is missing metrics the wider
    /// request needs, so it must miss rather than under-report them as absent.
    #[test]
    fn a_narrower_sidecar_does_not_answer_a_wider_request() {
        let (root, cache_dir, index) = containment_fixture(AnalysisSet::NONE.with_code());
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&index, request_for(AnalysisSet::NONE.with_code()), &cache)
            .expect("save");

        let (mut restored, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert_eq!(
            load_content_cache(&mut restored, request_for(AnalysisSet::ALL), &cache).expect("load"),
            ContentCacheLoad::default(),
            "a subset sidecar must be a clean miss"
        );
    }

    /// Serving a narrow request from a wide tier must not relabel the tier on the way
    /// out: one `--analyze code` run would otherwise permanently downgrade a sidecar that
    /// still holds every analyzer.
    #[test]
    fn serving_a_narrow_request_preserves_the_wider_stored_set() {
        let (root, cache_dir, index) = containment_fixture(AnalysisSet::ALL);
        let cache = cache_dir.path().join("content.cache");
        save_content_cache(&index, request_for(AnalysisSet::ALL), &cache).expect("save");

        let narrower = request_for(AnalysisSet::NONE.with_code());
        let (mut restored, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        load_content_cache(&mut restored, narrower, &cache).expect("load");
        save_content_cache(&restored, narrower, &cache).expect("resave");

        let (mut reloaded, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        let wide =
            load_content_cache(&mut reloaded, request_for(AnalysisSet::ALL), &cache).expect("load");
        assert!(wide.usable, "the wider set survived a narrow round trip: {wide:?}");
        assert_eq!(wide.hits, 2, "and still carries every record: {wide:?}");
    }

    #[test]
    fn corruption_is_a_clean_miss() {
        let (root, index, request) = analyzed_index();
        let cache = root.path().join("content.cache");
        save_content_cache(&index, request, &cache).expect("save");
        let mut bytes = fs::read(&cache).expect("read");
        bytes[MAGIC.len() + 4] ^= 0xff;
        fs::write(&cache, bytes).expect("corrupt");
        let (mut restored, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert_eq!(
            load_content_cache(&mut restored, request, &cache).expect("load"),
            ContentCacheLoad::default()
        );
    }

    #[test]
    fn sidecar_name_preserves_the_metadata_snapshot_name() {
        assert_eq!(content_cache_path(Path::new("tree.fdu")), PathBuf::from("tree.fdu.content"));
    }
}
