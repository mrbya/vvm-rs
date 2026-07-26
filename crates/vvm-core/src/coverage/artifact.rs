use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    CoverageDefinitionFingerprint, CoverageGroupSnapshot, CoverageGroupSummary,
    CoveragePersistenceError, CoverageRatio, CoverageSessionSnapshot, CoverageSessionSummary,
    ReplayToken, TestStatus,
};

/// One persisted coverage-group instance and its definition fingerprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageArtifactGroup {
    /// Captured immutable group state.
    snapshot: CoverageGroupSnapshot,
    /// Structural definition fingerprint.
    definition_fingerprint: CoverageDefinitionFingerprint,
}

impl CoverageArtifactGroup {
    /// Reconstructs one artifact group from validated merged data.
    pub(crate) const fn from_parts(
        snapshot: CoverageGroupSnapshot,
        definition_fingerprint: CoverageDefinitionFingerprint,
    ) -> Self {
        Self {
            snapshot,
            definition_fingerprint,
        }
    }
    /// Returns the definition name.
    #[must_use]
    pub fn definition_name(&self) -> &str {
        self.snapshot.definition_name()
    }
    /// Returns the definition revision.
    #[must_use]
    pub const fn definition_revision(&self) -> u64 {
        self.snapshot.definition_revision()
    }
    /// Returns the definition fingerprint.
    #[must_use]
    pub const fn definition_fingerprint(&self) -> CoverageDefinitionFingerprint {
        self.definition_fingerprint
    }
    /// Returns the instance path.
    #[must_use]
    pub fn instance_path(&self) -> &str {
        self.snapshot.instance_path()
    }
    /// Returns the immutable group snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &CoverageGroupSnapshot {
        &self.snapshot
    }
    /// Returns exact group summary.
    #[must_use]
    pub const fn summary(&self) -> CoverageGroupSummary {
        self.snapshot.summary()
    }
    /// Returns exact group coverage.
    #[must_use]
    pub const fn coverage(&self) -> CoverageRatio {
        self.snapshot.coverage()
    }
}

/// Versioned persisted functional-coverage artifact from one VVM test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageArtifact {
    /// Producing VVM crate version.
    producer_version: Arc<str>,
    /// Stable test name.
    test_name: Arc<str>,
    /// Test execution status.
    test_status: TestStatus,
    /// Replay metadata when available.
    replay_token: Option<ReplayToken>,
    /// Captured groups in session order.
    groups: Box<[CoverageArtifactGroup]>,
    /// Exact aggregate session summary.
    summary: CoverageSessionSummary,
}

impl CoverageArtifact {
    /// Consumes only the captured group records for offline merging.
    pub(crate) fn into_groups(self) -> Box<[CoverageArtifactGroup]> {
        self.groups
    }
    /// Persisted artifact format name.
    pub const FORMAT_NAME: &'static str = "vvm-functional-coverage";
    /// Supported JSON schema version.
    pub const SCHEMA_VERSION: u32 = 1;
    /// Standard artifact suffix.
    pub const FILE_SUFFIX: &'static str = ".vvmcov.json";
    /// Converts a completed session into a persisted artifact model.
    ///
    /// # Errors
    ///
    /// Returns an error when a group fingerprint cannot be calculated.
    pub fn from_session(
        test_status: TestStatus,
        replay_token: Option<ReplayToken>,
        session: CoverageSessionSnapshot,
    ) -> Result<Self, CoveragePersistenceError> {
        let (test_name, groups, summary) = session.into_parts();
        let groups = groups
            .into_vec()
            .into_iter()
            .map(|snapshot| {
                let definition_fingerprint = CoverageDefinitionFingerprint::from_group(&snapshot)?;
                Ok(CoverageArtifactGroup {
                    snapshot,
                    definition_fingerprint,
                })
            })
            .collect::<Result<Vec<_>, CoveragePersistenceError>>()?
            .into_boxed_slice();

        Ok(Self {
            producer_version: Arc::from(env!("CARGO_PKG_VERSION")),
            test_name,
            test_status,
            replay_token,
            groups,
            summary,
        })
    }
    /// Returns schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        Self::SCHEMA_VERSION
    }
    /// Returns producer version.
    #[must_use]
    pub fn producer_version(&self) -> &str {
        &self.producer_version
    }
    /// Returns test name.
    #[must_use]
    pub fn test_name(&self) -> &str {
        &self.test_name
    }
    /// Returns test status.
    #[must_use]
    pub const fn test_status(&self) -> TestStatus {
        self.test_status
    }
    /// Returns replay token.
    #[must_use]
    pub const fn replay_token(&self) -> Option<ReplayToken> {
        self.replay_token
    }
    /// Returns groups in capture order.
    #[must_use]
    pub fn groups(&self) -> &[CoverageArtifactGroup] {
        &self.groups
    }
    /// Finds a group by path.
    #[must_use]
    pub fn group(&self, path: &str) -> Option<&CoverageArtifactGroup> {
        self.groups
            .iter()
            .find(|group| group.instance_path() == path)
    }
    /// Returns session summary.
    #[must_use]
    pub const fn summary(&self) -> CoverageSessionSummary {
        self.summary
    }
    /// Returns session coverage.
    #[must_use]
    pub const fn coverage(&self) -> CoverageRatio {
        self.summary.coverage()
    }
    /// Serializes compact schema-v1 JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when JSON encoding fails.
    pub fn to_json(&self) -> Result<String, CoveragePersistenceError> {
        serde_json::to_string(&self.json_value())
            .map_err(|source| CoveragePersistenceError::JsonEncode { source })
    }
    /// Serializes pretty schema-v1 JSON with exactly one trailing newline.
    ///
    /// # Errors
    ///
    /// Returns an error when JSON encoding fails.
    pub fn to_json_pretty(&self) -> Result<String, CoveragePersistenceError> {
        let mut text = serde_json::to_string_pretty(&self.json_value())
            .map_err(|source| CoveragePersistenceError::JsonEncode { source })?;
        text.push('\n');
        Ok(text)
    }
    /// Deserializes and semantically validates schema-v1 JSON.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON, an unsupported schema, or invalid
    /// persisted coverage data.
    pub fn from_json(json: &str) -> Result<Self, CoveragePersistenceError> {
        let document = serde_json::from_str::<ArtifactDto>(json)
            .map_err(|source| CoveragePersistenceError::JsonDecode { source })?;

        Self::from_dto(document)
    }
    /// Reads and semantically validates an artifact file.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read or the artifact is invalid.
    pub fn read_from(path: impl AsRef<Path>) -> Result<Self, CoveragePersistenceError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| CoveragePersistenceError::Io {
            operation: crate::CoverageIoOperation::Read,
            path: path.to_owned(),
            source,
        })?;

        Self::from_json(&text)
    }
    /// Atomically writes this artifact without overwriting a destination.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization or a filesystem operation fails.
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), CoveragePersistenceError> {
        let path = path.as_ref();
        let bytes = self.to_json_pretty()?.into_bytes();
        let parent = path
            .parent()
            .ok_or_else(|| CoveragePersistenceError::InvalidData {
                path: path.display().to_string(),
                reason: "artifact path has no parent directory".to_owned(),
            })?;

        fs::create_dir_all(parent).map_err(|source| CoveragePersistenceError::Io {
            operation: crate::CoverageIoOperation::CreateDirectory,
            path: parent.to_owned(),
            source,
        })?;

        if path.exists() {
            return Err(CoveragePersistenceError::DestinationExists {
                path: path.to_owned(),
            });
        }

        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| CoveragePersistenceError::InvalidData {
                path: path.display().to_string(),
                reason: "artifact filename is not valid UTF-8".to_owned(),
            })?;

        let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = parent.join(format!(
            ".{filename}.{}.{}.tmp",
            std::process::id(),
            sequence
        ));

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|source| CoveragePersistenceError::Io {
                operation: crate::CoverageIoOperation::CreateTemporaryFile,
                path: temporary.clone(),
                source,
            })?;

        let result = file
            .write_all(&bytes)
            .map_err(|source| CoveragePersistenceError::Io {
                operation: crate::CoverageIoOperation::Write,
                path: temporary.clone(),
                source,
            })
            .and_then(|()| {
                file.sync_all()
                    .map_err(|source| CoveragePersistenceError::Io {
                        operation: crate::CoverageIoOperation::Synchronize,
                        path: temporary.clone(),
                        source,
                    })
            })
            .and_then(|()| {
                fs::rename(&temporary, path).map_err(|source| CoveragePersistenceError::Io {
                    operation: crate::CoverageIoOperation::Rename,
                    path: path.to_owned(),
                    source,
                })
            });

        if result.is_err() {
            drop(fs::remove_file(&temporary));
        }

        result
    }
    /// Builds the schema-v1 JSON value.
    fn json_value(&self) -> Value {
        json!({ "format": Self::FORMAT_NAME, "schema_version": Self::SCHEMA_VERSION, "producer": { "name": "vvm-rs", "version": self.producer_version.as_ref() }, "test": { "name": self.test_name.as_ref(), "status": status_text(self.test_status), "replay_token": self.replay_token.map(|token| token.to_string()) }, "summary": summary_json(self.summary), "groups": self.groups.iter().map(group_json).collect::<Vec<_>>() })
    }
    /// Reconstructs a validated artifact from its wire representation.
    fn from_dto(document: ArtifactDto) -> Result<Self, CoveragePersistenceError> {
        if document.format != Self::FORMAT_NAME {
            return Err(CoveragePersistenceError::InvalidFormat {
                found: document.format,
            });
        }

        if document.schema_version != Self::SCHEMA_VERSION {
            return Err(CoveragePersistenceError::UnsupportedSchemaVersion {
                found: document.schema_version,
                supported: Self::SCHEMA_VERSION,
            });
        }

        if document.producer.name != "vvm-rs" || document.producer.version.is_empty() {
            return Err(invalid(
                "producer",
                "expected producer name `vvm-rs` and a non-empty version",
            ));
        }

        if !crate::registry::is_valid_test_name(&document.test.name) {
            return Err(invalid("test.name", "invalid test name"));
        }

        let test_status = parse_status(&document.test.status)?;
        let replay_token = document
            .test
            .replay_token
            .as_deref()
            .map(|value| {
                ReplayToken::from_str(value)
                    .map_err(|_error| invalid("test.replay_token", "invalid replay token"))
            })
            .transpose()?;

        let groups = document
            .groups
            .into_iter()
            .enumerate()
            .map(|(index, group)| group_from_dto(group, index))
            .collect::<Result<Vec<_>, _>>()?;

        if groups.is_empty() {
            return Err(invalid(
                "groups",
                "an artifact must contain at least one group",
            ));
        }

        let summary = session_summary(&groups)?;
        validate_session_summary(&document.summary, summary)?;

        // Construct only after all persisted fields and aggregate state validate.
        Ok(Self {
            producer_version: Arc::from(document.producer.version),
            test_name: Arc::from(document.test.name),
            test_status,
            replay_token,
            groups: groups.into_boxed_slice(),
            summary,
        })
    }
}

/// Process-local suffix used only to avoid temporary-file collisions.
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);
/// Returns the stable persisted status string.
pub const fn status_text(status: TestStatus) -> &'static str {
    match status {
        TestStatus::Passed => "passed",
        TestStatus::Failed => "failed",
        TestStatus::Error => "error",
    }
}
/// Converts an exact ratio to schema JSON.
pub fn ratio_json(value: CoverageRatio) -> Value {
    json!({ "covered": value.covered(), "uncovered": value.uncovered(), "total": value.total() })
}
/// Converts a session summary to schema JSON.
pub fn summary_json(value: CoverageSessionSummary) -> Value {
    json!({ "groups": value.group_count(), "items": value.item_count(), "coverpoints": value.coverpoint_count(), "crosses": value.cross_count(), "coverage": ratio_json(value.coverage()) })
}
/// Converts a captured group to schema JSON.
pub fn group_json(group: &CoverageArtifactGroup) -> Value {
    json!({ "definition": { "name": group.definition_name(), "revision": group.definition_revision(), "fingerprint": group.definition_fingerprint().to_string() }, "instance_path": group.instance_path(), "summary": { "items": group.summary().item_count(), "coverpoints": group.summary().coverpoint_count(), "crosses": group.summary().cross_count(), "coverage": ratio_json(group.coverage()) }, "items": group.snapshot().items().iter().map(item_json).collect::<Vec<_>>() })
}
/// Converts a captured item to schema JSON.
fn item_json(item: &crate::CoverageItemSnapshot) -> Value {
    match *item {
        crate::CoverageItemSnapshot::Coverpoint(ref point) => {
            json!({ "kind": "coverpoint", "name": point.name(), "samples": point.sample_count(), "ignored_samples": point.ignored_sample_count(), "illegal_samples": point.illegal_sample_count(), "unmatched_samples": point.unmatched_sample_count(), "coverage": ratio_json(point.coverage()), "bins": point.bins().iter().map(|bin| json!({ "id": bin.id().ordinal(), "name": bin.name(), "kind": bin.kind().to_string(), "matcher": { "kind": bin.matcher_kind().to_string(), "operand_count": bin.matcher_operand_count() }, "hits": bin.hits(), "required_hits": bin.required_hits() })).collect::<Vec<_>>() })
        }
        crate::CoverageItemSnapshot::Cross2(ref cross) => {
            json!({ "kind": "cross2", "name": cross.name(), "left_coverpoint": cross.left_coverpoint_name(), "right_coverpoint": cross.right_coverpoint_name(), "samples": cross.sample_count(), "skipped_samples": cross.skipped_sample_count(), "coverage": ratio_json(cross.coverage()), "bins": cross.bins().iter().map(|bin| json!({ "id": bin.id().ordinal(), "left_bin_id": bin.left_bin_id().ordinal(), "left_bin_name": bin.left_bin_name(), "right_bin_id": bin.right_bin_id().ordinal(), "right_bin_name": bin.right_bin_name(), "hits": bin.hits(), "required_hits": bin.required_hits() })).collect::<Vec<_>>() })
        }
    }
}

/// Strict schema-v1 artifact document.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactDto {
    /// Format identifier.
    format: String,
    /// Schema version.
    schema_version: u32,
    /// Producer metadata.
    producer: ProducerDto,
    /// Test metadata.
    test: TestDto,
    /// Aggregate summary.
    summary: SessionSummaryDto,
    /// Captured groups.
    groups: Vec<GroupDto>,
}

/// Strict producer metadata schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducerDto {
    /// Producer name.
    name: String,
    /// Producer version.
    version: String,
}

/// Strict test metadata schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TestDto {
    /// Stable test name.
    name: String,
    /// Persisted test status.
    status: String,
    /// Optional replay token.
    replay_token: Option<String>,
}

/// Strict exact-ratio schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RatioDto {
    /// Covered normal-bin count.
    covered: usize,
    /// Uncovered normal-bin count.
    uncovered: usize,
    /// Total normal-bin count.
    total: usize,
}

/// Strict session-summary schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionSummaryDto {
    /// Group count.
    groups: usize,
    /// Item count.
    items: usize,
    /// Coverpoint count.
    coverpoints: usize,
    /// Cross count.
    crosses: usize,
    /// Aggregate coverage.
    coverage: RatioDto,
}

/// Strict group schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupDto {
    /// Definition metadata.
    definition: DefinitionDto,
    /// Instance path.
    instance_path: String,
    /// Group summary.
    summary: GroupSummaryDto,
    /// Group items.
    items: Vec<ItemDto>,
}

/// Strict definition schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DefinitionDto {
    /// Definition name.
    name: String,
    /// Definition revision.
    revision: u64,
    /// Definition fingerprint.
    fingerprint: String,
}

/// Strict group-summary schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GroupSummaryDto {
    /// Item count.
    items: usize,
    /// Coverpoint count.
    coverpoints: usize,
    /// Cross count.
    crosses: usize,
    /// Aggregate coverage.
    coverage: RatioDto,
}

/// Strict tagged item schema.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ItemDto {
    /// Coverpoint snapshot.
    Coverpoint {
        /// Item name.
        name: String,
        /// Sample count.
        samples: u64,
        /// Ignored sample count.
        ignored_samples: u64,
        /// Illegal sample count.
        illegal_samples: u64,
        /// Unmatched sample count.
        unmatched_samples: u64,
        /// Item coverage.
        coverage: RatioDto,
        /// Coverpoint bins.
        bins: Vec<CoverpointBinDto>,
    },
    /// Two-way cross snapshot.
    Cross2 {
        /// Item name.
        name: String,
        /// Left source coverpoint.
        left_coverpoint: String,
        /// Right source coverpoint.
        right_coverpoint: String,
        /// Sample count.
        samples: u64,
        /// Skipped sample count.
        skipped_samples: u64,
        /// Item coverage.
        coverage: RatioDto,
        /// Generated cross bins.
        bins: Vec<CrossBinDto>,
    },
}

/// Strict coverpoint-bin schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverpointBinDto {
    /// Declaration ordinal.
    id: u32,
    /// Bin name.
    name: String,
    /// Bin role.
    kind: String,
    /// Matcher shape.
    matcher: MatcherDto,
    /// Hit count.
    hits: u64,
    /// Required hit count.
    required_hits: u64,
}

/// Strict matcher schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MatcherDto {
    /// Matcher kind.
    kind: String,
    /// Matcher operand count.
    operand_count: usize,
}

/// Strict cross-bin schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CrossBinDto {
    /// Row-major ordinal.
    id: u32,
    /// Left source bin ordinal.
    left_bin_id: u32,
    /// Left source bin name.
    left_bin_name: String,
    /// Right source bin ordinal.
    right_bin_id: u32,
    /// Right source bin name.
    right_bin_name: String,
    /// Hit count.
    hits: u64,
    /// Required hit count.
    required_hits: u64,
}

/// Builds and validates one persisted group.
pub fn group_from_dto(
    group: GroupDto,
    index: usize,
) -> Result<CoverageArtifactGroup, CoveragePersistenceError> {
    let path = format!("groups[{index}]");
    if !crate::coverage::identifier::is_valid_coverage_identifier(&group.definition.name) {
        return Err(invalid(
            &format!("{path}.definition.name"),
            "invalid definition name",
        ));
    }
    if !crate::coverage::identifier::is_valid_coverage_path(&group.instance_path) {
        return Err(invalid(
            &format!("{path}.instance_path"),
            "invalid instance path",
        ));
    }

    let items = group
        .items
        .into_iter()
        .enumerate()
        .map(|(item_index, item)| item_from_dto(item, &format!("{path}.items[{item_index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    validate_item_names(&items, &path)?;
    validate_cross_sources(&items, &path)?;

    let summary = group_summary(&items)?;
    validate_group_summary(&group.summary, summary, &path)?;
    let snapshot = crate::CoverageGroupSnapshot::from_parts(
        Arc::from(group.definition.name),
        group.definition.revision,
        Arc::from(group.instance_path.clone()),
        items.into_boxed_slice(),
        summary,
    );
    let stored = CoverageDefinitionFingerprint::from_str(&group.definition.fingerprint)?;
    let computed = CoverageDefinitionFingerprint::from_group(&snapshot)?;
    if stored != computed {
        return Err(CoveragePersistenceError::FingerprintMismatch {
            instance_path: group.instance_path,
            stored,
            computed,
        });
    }

    Ok(CoverageArtifactGroup {
        snapshot,
        definition_fingerprint: stored,
    })
}

/// Reconstructs and validates one tagged coverage item.
fn item_from_dto(
    item: ItemDto,
    path: &str,
) -> Result<crate::CoverageItemSnapshot, CoveragePersistenceError> {
    match item {
        ItemDto::Coverpoint {
            name,
            samples,
            ignored_samples,
            illegal_samples,
            unmatched_samples,
            coverage,
            bins,
        } => {
            validate_name(&name, &format!("{path}.name"))?;
            let classified = ignored_samples
                .checked_add(illegal_samples)
                .and_then(|value| value.checked_add(unmatched_samples));
            if classified.is_none_or(|value| value > samples) {
                return Err(invalid(path, "sample classifications exceed samples"));
            }
            let bins = bins
                .into_iter()
                .enumerate()
                .map(|(index, bin)| coverpoint_bin_from_dto(bin, index, path))
                .collect::<Result<Vec<_>, _>>()?;
            validate_coverpoint_bins(&bins, samples, path)?;
            let computed = coverpoint_ratio(&bins)?;
            validate_ratio(&coverage, computed, &format!("{path}.coverage"))?;
            Ok(crate::CoverageItemSnapshot::Coverpoint(
                crate::CoverpointSnapshot::from_parts(
                    Arc::from(name),
                    bins.into_boxed_slice(),
                    samples,
                    ignored_samples,
                    illegal_samples,
                    unmatched_samples,
                    computed,
                ),
            ))
        }
        ItemDto::Cross2 {
            name,
            left_coverpoint,
            right_coverpoint,
            samples,
            skipped_samples,
            coverage,
            bins,
        } => {
            validate_name(&name, &format!("{path}.name"))?;
            validate_name(&left_coverpoint, &format!("{path}.left_coverpoint"))?;
            validate_name(&right_coverpoint, &format!("{path}.right_coverpoint"))?;
            if skipped_samples > samples {
                return Err(invalid(path, "skipped samples exceed samples"));
            }
            let bins = bins
                .into_iter()
                .enumerate()
                .map(|(index, bin)| cross_bin_from_dto(bin, index, path))
                .collect::<Result<Vec<_>, _>>()?;
            validate_cross_bins(&bins, samples, path)?;
            let computed = cross_ratio(&bins)?;
            validate_ratio(&coverage, computed, &format!("{path}.coverage"))?;
            Ok(crate::CoverageItemSnapshot::Cross2(
                crate::Cross2Snapshot::from_parts(
                    Arc::from(name),
                    Arc::from(left_coverpoint),
                    Arc::from(right_coverpoint),
                    bins.into_boxed_slice(),
                    samples,
                    skipped_samples,
                    computed,
                ),
            ))
        }
    }
}

/// Reconstructs one coverpoint bin and validates its definition fields.
fn coverpoint_bin_from_dto(
    bin: CoverpointBinDto,
    index: usize,
    path: &str,
) -> Result<crate::CoverpointBinSnapshot, CoveragePersistenceError> {
    let field = format!("{path}.bins[{index}]");
    if bin.id != u32::try_from(index).map_err(|_error| overflow(&field))? {
        return Err(invalid(
            &field,
            "bin IDs must be consecutive declaration ordinals",
        ));
    }
    validate_name(&bin.name, &format!("{field}.name"))?;
    let kind = match bin.kind.as_str() {
        "normal" => crate::BinKind::Normal,
        "ignore" => crate::BinKind::Ignore,
        "illegal" => crate::BinKind::Illegal,
        _ => return Err(invalid(&format!("{field}.kind"), "unknown bin kind")),
    };
    let matcher_kind = match bin.matcher.kind.as_str() {
        "value" => crate::BinMatcherKind::Value,
        "values" => crate::BinMatcherKind::Values,
        "inclusive_range" => crate::BinMatcherKind::InclusiveRange,
        _ => {
            return Err(invalid(
                &format!("{field}.matcher.kind"),
                "unknown matcher kind",
            ));
        }
    };
    let valid_operands = match matcher_kind {
        crate::BinMatcherKind::Value => bin.matcher.operand_count == 1,
        crate::BinMatcherKind::Values => bin.matcher.operand_count > 0,
        crate::BinMatcherKind::InclusiveRange => bin.matcher.operand_count == 2,
    };
    if !valid_operands {
        return Err(invalid(
            &format!("{field}.matcher.operand_count"),
            "invalid matcher operand count",
        ));
    }
    if bin.required_hits == 0 {
        return Err(invalid(
            &format!("{field}.required_hits"),
            "required hits must be non-zero",
        ));
    }
    if !matches!(kind, crate::BinKind::Normal) && bin.required_hits != 1 {
        return Err(invalid(
            &format!("{field}.required_hits"),
            "non-normal bins require exactly one hit",
        ));
    }

    Ok(crate::CoverpointBinSnapshot::from_parts(
        crate::BinId::new(bin.id),
        Arc::from(bin.name),
        kind,
        matcher_kind,
        bin.matcher.operand_count,
        bin.hits,
        bin.required_hits,
    ))
}

/// Reconstructs one cross bin and validates its declaration ordinal.
fn cross_bin_from_dto(
    bin: CrossBinDto,
    index: usize,
    path: &str,
) -> Result<crate::CrossBinSnapshot, CoveragePersistenceError> {
    let field = format!("{path}.bins[{index}]");
    if bin.id != u32::try_from(index).map_err(|_error| overflow(&field))? {
        return Err(invalid(
            &field,
            "bin IDs must be consecutive row-major ordinals",
        ));
    }
    validate_name(&bin.left_bin_name, &format!("{field}.left_bin_name"))?;
    validate_name(&bin.right_bin_name, &format!("{field}.right_bin_name"))?;
    if bin.required_hits == 0 {
        return Err(invalid(
            &format!("{field}.required_hits"),
            "required hits must be non-zero",
        ));
    }

    Ok(crate::CrossBinSnapshot::from_parts(
        crate::CrossBinId::new(bin.id),
        crate::BinId::new(bin.left_bin_id),
        Arc::from(bin.left_bin_name),
        crate::BinId::new(bin.right_bin_id),
        Arc::from(bin.right_bin_name),
        bin.hits,
        bin.required_hits,
    ))
}

/// Calculates exact coverpoint coverage from normal bins.
fn coverpoint_ratio(
    bins: &[crate::CoverpointBinSnapshot],
) -> Result<CoverageRatio, CoveragePersistenceError> {
    let covered = bins.iter().filter(|bin| bin.covered()).count();
    let uncovered = bins
        .iter()
        .filter(|bin| bin.kind() == crate::BinKind::Normal && !bin.covered())
        .count();
    let total = covered
        .checked_add(uncovered)
        .ok_or_else(|| overflow("coverpoint coverage"))?;
    Ok(CoverageRatio::new(covered, uncovered, total))
}

/// Calculates exact cross coverage from generated bins.
fn cross_ratio(
    bins: &[crate::CrossBinSnapshot],
) -> Result<CoverageRatio, CoveragePersistenceError> {
    let covered = bins.iter().filter(|bin| bin.covered()).count();
    let uncovered = bins
        .len()
        .checked_sub(covered)
        .ok_or_else(|| invalid("cross coverage", "covered bins exceed total bins"))?;
    Ok(CoverageRatio::new(covered, uncovered, bins.len()))
}

/// Validates the bin invariants established by a live coverpoint builder.
fn validate_coverpoint_bins(
    bins: &[crate::CoverpointBinSnapshot],
    samples: u64,
    path: &str,
) -> Result<(), CoveragePersistenceError> {
    let mut names = std::collections::BTreeSet::new();
    let normal_bins = bins
        .iter()
        .filter(|bin| bin.kind() == crate::BinKind::Normal)
        .count();

    if normal_bins == 0 {
        return Err(invalid(path, "coverpoint must contain a normal bin"));
    }
    for bin in bins {
        if !names.insert(bin.name()) {
            return Err(invalid(path, "duplicate coverpoint bin name"));
        }
        if bin.hits() > samples {
            return Err(invalid(path, "bin hits exceed samples"));
        }
    }
    Ok(())
}

/// Validates the generated-bin invariants established by a live cross builder.
fn validate_cross_bins(
    bins: &[crate::CrossBinSnapshot],
    samples: u64,
    path: &str,
) -> Result<(), CoveragePersistenceError> {
    let Some(first) = bins.first() else {
        return Err(invalid(path, "cross must contain generated bins"));
    };
    let required_hits = first.required_hits();

    for bin in bins {
        if bin.hits() > samples {
            return Err(invalid(path, "cross bin hits exceed samples"));
        }
        if bin.required_hits() != required_hits {
            return Err(invalid(path, "cross bins must have one hit threshold"));
        }
    }
    Ok(())
}

/// Calculates one exact group summary from reconstructed items.
fn group_summary(
    items: &[crate::CoverageItemSnapshot],
) -> Result<CoverageGroupSummary, CoveragePersistenceError> {
    let coverpoints = items
        .iter()
        .filter(|item| item.as_coverpoint().is_some())
        .count();
    let crosses = items
        .iter()
        .filter(|item| item.as_cross2().is_some())
        .count();
    let coverage = sum_ratios(
        items.iter().map(crate::CoverageItemSnapshot::coverage),
        "group coverage",
    )?;
    Ok(CoverageGroupSummary::from_parts(
        items.len(),
        coverpoints,
        crosses,
        coverage,
    ))
}

/// Calculates one exact session summary from reconstructed groups.
fn session_summary(
    groups: &[CoverageArtifactGroup],
) -> Result<CoverageSessionSummary, CoveragePersistenceError> {
    let mut paths = std::collections::BTreeSet::new();
    for group in groups {
        if !paths.insert(group.instance_path()) {
            return Err(invalid("groups", "duplicate instance path"));
        }
    }
    let items = groups
        .iter()
        .map(|group| group.summary().item_count())
        .try_fold(0_usize, |total, value| {
            total
                .checked_add(value)
                .ok_or_else(|| overflow("summary.items"))
        })?;
    let coverpoints = groups
        .iter()
        .map(|group| group.summary().coverpoint_count())
        .try_fold(0_usize, |total, value| {
            total
                .checked_add(value)
                .ok_or_else(|| overflow("summary.coverpoints"))
        })?;
    let crosses = groups
        .iter()
        .map(|group| group.summary().cross_count())
        .try_fold(0_usize, |total, value| {
            total
                .checked_add(value)
                .ok_or_else(|| overflow("summary.crosses"))
        })?;
    let coverage = sum_ratios(
        groups.iter().map(CoverageArtifactGroup::coverage),
        "summary.coverage",
    )?;
    Ok(CoverageSessionSummary::from_parts(
        groups.len(),
        items,
        coverpoints,
        crosses,
        coverage,
    ))
}

/// Adds exact coverage ratios with overflow checking.
fn sum_ratios(
    mut values: impl Iterator<Item = CoverageRatio>,
    field: &str,
) -> Result<CoverageRatio, CoveragePersistenceError> {
    values.try_fold(CoverageRatio::new(0, 0, 0), |total, value| {
        let covered = total
            .covered()
            .checked_add(value.covered())
            .ok_or_else(|| overflow(field))?;
        let uncovered = total
            .uncovered()
            .checked_add(value.uncovered())
            .ok_or_else(|| overflow(field))?;
        let bins = total
            .total()
            .checked_add(value.total())
            .ok_or_else(|| overflow(field))?;
        Ok(CoverageRatio::new(covered, uncovered, bins))
    })
}

/// Ensures item names are unique in group visitation order.
fn validate_item_names(
    items: &[crate::CoverageItemSnapshot],
    path: &str,
) -> Result<(), CoveragePersistenceError> {
    let mut names = std::collections::BTreeSet::new();
    for item in items {
        if !names.insert(item.name()) {
            return Err(invalid(path, "duplicate item name"));
        }
    }
    Ok(())
}

/// Ensures every cross refers to declared coverpoints and generated bins match them.
fn validate_cross_sources(
    items: &[crate::CoverageItemSnapshot],
    path: &str,
) -> Result<(), CoveragePersistenceError> {
    let points = items
        .iter()
        .filter_map(crate::CoverageItemSnapshot::as_coverpoint)
        .collect::<Vec<_>>();
    for cross in items
        .iter()
        .filter_map(crate::CoverageItemSnapshot::as_cross2)
    {
        let left = points
            .iter()
            .find(|point| point.name() == cross.left_coverpoint_name())
            .ok_or_else(|| invalid(path, "cross left coverpoint is not declared"))?;
        let right = points
            .iter()
            .find(|point| point.name() == cross.right_coverpoint_name())
            .ok_or_else(|| invalid(path, "cross right coverpoint is not declared"))?;
        let expected = left
            .bins()
            .iter()
            .filter(|bin| bin.kind() == crate::BinKind::Normal)
            .flat_map(|left_bin| {
                right
                    .bins()
                    .iter()
                    .filter(|bin| bin.kind() == crate::BinKind::Normal)
                    .map(move |right_bin| (left_bin, right_bin))
            });
        if cross.bins().len() != expected.clone().count() {
            return Err(invalid(
                path,
                "cross bin count does not match source normal bins",
            ));
        }
        for (index, ((left_bin, right_bin), bin)) in expected.zip(cross.bins()).enumerate() {
            if bin.id().ordinal() != u32::try_from(index).map_err(|_error| overflow(path))?
                || bin.left_bin_id() != left_bin.id()
                || bin.left_bin_name() != left_bin.name()
                || bin.right_bin_id() != right_bin.id()
                || bin.right_bin_name() != right_bin.name()
            {
                return Err(invalid(
                    path,
                    "cross bins do not match source bins in row-major order",
                ));
            }
        }
    }
    Ok(())
}

/// Validates a stored group summary against a recomputed value.
fn validate_group_summary(
    stored: &GroupSummaryDto,
    computed: CoverageGroupSummary,
    path: &str,
) -> Result<(), CoveragePersistenceError> {
    if stored.items != computed.item_count()
        || stored.coverpoints != computed.coverpoint_count()
        || stored.crosses != computed.cross_count()
    {
        return Err(invalid(
            &format!("{path}.summary"),
            "item counts disagree with items",
        ));
    }
    validate_ratio(
        &stored.coverage,
        computed.coverage(),
        &format!("{path}.summary.coverage"),
    )
}

/// Validates a stored session summary against a recomputed value.
fn validate_session_summary(
    stored: &SessionSummaryDto,
    computed: CoverageSessionSummary,
) -> Result<(), CoveragePersistenceError> {
    if stored.groups != computed.group_count()
        || stored.items != computed.item_count()
        || stored.coverpoints != computed.coverpoint_count()
        || stored.crosses != computed.cross_count()
    {
        return Err(invalid("summary", "counts disagree with groups"));
    }
    validate_ratio(&stored.coverage, computed.coverage(), "summary.coverage")
}

/// Validates that an exact stored ratio is internally consistent and recomputed.
fn validate_ratio(
    stored: &RatioDto,
    computed: CoverageRatio,
    path: &str,
) -> Result<(), CoveragePersistenceError> {
    let total = stored
        .covered
        .checked_add(stored.uncovered)
        .ok_or_else(|| overflow(path))?;
    if total != stored.total {
        return Err(invalid(path, "covered plus uncovered must equal total"));
    }
    if stored.covered != computed.covered()
        || stored.uncovered != computed.uncovered()
        || stored.total != computed.total()
    {
        return Err(invalid(path, "stored ratio disagrees with bins"));
    }
    Ok(())
}

/// Validates one persisted identifier.
fn validate_name(value: &str, path: &str) -> Result<(), CoveragePersistenceError> {
    if crate::coverage::identifier::is_valid_coverage_identifier(value) {
        Ok(())
    } else {
        Err(invalid(path, "invalid coverage identifier"))
    }
}

/// Parses a stable schema status string.
pub fn parse_status(value: &str) -> Result<TestStatus, CoveragePersistenceError> {
    match value {
        "passed" => Ok(TestStatus::Passed),
        "failed" => Ok(TestStatus::Failed),
        "error" => Ok(TestStatus::Error),
        _ => Err(invalid("test.status", "unknown test status")),
    }
}

/// Constructs a semantic validation error.
fn invalid(path: &str, reason: &str) -> CoveragePersistenceError {
    CoveragePersistenceError::InvalidData {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}

/// Constructs a numeric conversion error.
fn overflow(field: &str) -> CoveragePersistenceError {
    CoveragePersistenceError::NumericOverflow {
        field: field.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::CoverageArtifact;
    use crate::{
        Bin, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor, CoverageItemRef,
        CoverageSession, Coverpoint, Cross2, TestStatus,
    };

    struct Group {
        instance: CoverageGroupInstance,
        left: Coverpoint<u8>,
        right: Coverpoint<u8>,
        cross: Cross2,
    }

    impl CoverageGroup for Group {
        fn instance(&self) -> &CoverageGroupInstance {
            &self.instance
        }

        fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
            visitor.visit(CoverageItemRef::coverpoint(&self.left));
            visitor.visit(CoverageItemRef::coverpoint(&self.right));
            visitor.visit(CoverageItemRef::cross2(&self.cross));
        }
    }

    #[test]
    fn round_trips_and_rejects_unknown_fields() -> Result<(), Box<dyn std::error::Error>> {
        let mut left = Coverpoint::builder("left")
            .bin(Bin::value("one", 1_u8))
            .build()?;
        let mut right = Coverpoint::builder("right")
            .bin(Bin::value("two", 2_u8))
            .build()?;
        let mut cross = Cross2::builder("left_x_right", &left, &right).build()?;
        let left_sample = left.sample(&1)?;
        let right_sample = right.sample(&2)?;
        cross.sample(&left_sample, &right_sample)?;
        let group = Group {
            instance: CoverageGroupInstance::new("decoder", "dut.decoder")?,
            left,
            right,
            cross,
        };
        let mut session = CoverageSession::new("decoder_random")?;
        session.capture(&group)?;
        let snapshot = session.finish().ok_or("missing session snapshot")?;
        let artifact = CoverageArtifact::from_session(TestStatus::Passed, None, snapshot)?;

        let json = artifact.to_json()?;
        assert_eq!(CoverageArtifact::from_json(&json)?, artifact);
        let parsed_with_whitespace = CoverageArtifact::from_json(&format!("{json} "))?;
        assert_eq!(parsed_with_whitespace, artifact);
        let unknown_field = CoverageArtifact::from_json(&json.replacen('{', "{\"extra\":0,", 1))
            .expect_err("unknown fields must be rejected");
        assert!(matches!(
            unknown_field,
            crate::CoveragePersistenceError::JsonDecode { .. }
        ));
        Ok(())
    }
}
