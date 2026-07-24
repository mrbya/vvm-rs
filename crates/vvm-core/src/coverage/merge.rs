use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    BinKind, CoverageArtifact, CoverageArtifactGroup, CoverageDefinitionFingerprint,
    CoverageGroupSnapshot, CoverageGroupSummary, CoverageItemSnapshot, CoverageMergeCountKind,
    CoverageMergeCounterKind, CoverageMergeError, CoveragePersistenceError, CoverageRatio,
    CoverageSessionSummary, CoverpointBinSnapshot, CoverpointSnapshot, Cross2Snapshot,
    CrossBinSnapshot, ReplayToken, TestStatus,
};

/// Determines which persisted test outcomes contribute counters.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageMergePolicy {
    /// Only successful tests contribute.
    #[default]
    PassedOnly,
    /// Successful and verification-failed tests contribute.
    PassedAndFailed,
    /// Passed, failed, and errored tests contribute.
    All,
}

impl CoverageMergePolicy {
    /// Constructs the default passed-only policy.
    #[must_use]
    pub const fn passed_only() -> Self {
        Self::PassedOnly
    }
    /// Includes passed and failed tests.
    #[must_use]
    pub const fn passed_and_failed() -> Self {
        Self::PassedAndFailed
    }
    /// Includes every persisted status.
    #[must_use]
    pub const fn all() -> Self {
        Self::All
    }
    /// Returns whether one status contributes counters.
    #[must_use]
    pub const fn includes(self, status: TestStatus) -> bool {
        match self {
            Self::PassedOnly => matches!(status, TestStatus::Passed),
            Self::PassedAndFailed => matches!(status, TestStatus::Passed | TestStatus::Failed),
            Self::All => true,
        }
    }
}

impl fmt::Display for CoverageMergePolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match *self {
            Self::PassedOnly => "passed_only",
            Self::PassedAndFailed => "passed_and_failed",
            Self::All => "all",
        })
    }
}

impl FromStr for CoverageMergePolicy {
    type Err = CoveragePersistenceError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "passed_only" => Ok(Self::PassedOnly),
            "passed_and_failed" => Ok(Self::PassedAndFailed),
            "all" => Ok(Self::All),
            _ => Err(CoveragePersistenceError::InvalidData {
                path: "policy".to_owned(),
                reason: format!("unknown merge policy `{value}`"),
            }),
        }
    }
}

/// Metadata about one per-test artifact supplied to a merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageMergeInput {
    /// Producing VVM version.
    producer_version: Arc<str>,
    /// Stable VVM test name.
    test_name: Arc<str>,
    /// Persisted test status.
    test_status: TestStatus,
    /// Replay token when present.
    replay_token: Option<ReplayToken>,
    /// Whether this artifact contributed counters.
    included: bool,
    /// Original per-test session summary.
    summary: CoverageSessionSummary,
}

impl CoverageMergeInput {
    /// Returns the producing VVM version.
    #[must_use]
    pub fn producer_version(&self) -> &str {
        &self.producer_version
    }
    /// Returns the stable VVM test name.
    #[must_use]
    pub fn test_name(&self) -> &str {
        &self.test_name
    }
    /// Returns the persisted test status.
    #[must_use]
    pub const fn test_status(&self) -> TestStatus {
        self.test_status
    }
    /// Returns the replay token when present.
    #[must_use]
    pub const fn replay_token(&self) -> Option<ReplayToken> {
        self.replay_token
    }
    /// Returns whether this artifact contributed counters.
    #[must_use]
    pub const fn included(&self) -> bool {
        self.included
    }
    /// Returns the original per-test session summary.
    #[must_use]
    pub const fn summary(&self) -> CoverageSessionSummary {
        self.summary
    }
}

/// Exact summary of one deterministic multi-artifact merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoverageMergeSummary {
    /// All input artifacts.
    artifacts: usize,
    /// Artifacts included by policy.
    included_artifacts: usize,
    /// Artifacts excluded by policy.
    excluded_artifacts: usize,
    /// Passed input artifacts.
    passed_artifacts: usize,
    /// Failed input artifacts.
    failed_artifacts: usize,
    /// Errored input artifacts.
    errored_artifacts: usize,
    /// Unique merged group instances.
    groups: usize,
    /// Unique merged items.
    items: usize,
    /// Unique merged coverpoints.
    coverpoints: usize,
    /// Unique merged crosses.
    crosses: usize,
    /// Exact merged bin coverage.
    coverage: CoverageRatio,
}

impl CoverageMergeSummary {
    /// Returns all input artifacts.
    #[must_use]
    pub const fn artifact_count(self) -> usize {
        self.artifacts
    }
    /// Returns artifacts included by policy.
    #[must_use]
    pub const fn included_artifact_count(self) -> usize {
        self.included_artifacts
    }
    /// Returns artifacts excluded by policy.
    #[must_use]
    pub const fn excluded_artifact_count(self) -> usize {
        self.excluded_artifacts
    }
    /// Returns passed input artifacts.
    #[must_use]
    pub const fn passed_artifact_count(self) -> usize {
        self.passed_artifacts
    }
    /// Returns failed input artifacts.
    #[must_use]
    pub const fn failed_artifact_count(self) -> usize {
        self.failed_artifacts
    }
    /// Returns errored input artifacts.
    #[must_use]
    pub const fn errored_artifact_count(self) -> usize {
        self.errored_artifacts
    }
    /// Returns unique merged group instances.
    #[must_use]
    pub const fn group_count(self) -> usize {
        self.groups
    }
    /// Returns unique merged items.
    #[must_use]
    pub const fn item_count(self) -> usize {
        self.items
    }
    /// Returns unique merged coverpoints.
    #[must_use]
    pub const fn coverpoint_count(self) -> usize {
        self.coverpoints
    }
    /// Returns unique merged crosses.
    #[must_use]
    pub const fn cross_count(self) -> usize {
        self.crosses
    }
    /// Returns exact merged bin coverage.
    #[must_use]
    pub const fn coverage(self) -> CoverageRatio {
        self.coverage
    }
    /// Returns whether every merged bin is covered.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.coverage.is_complete()
    }
}

/// Deterministically merged functional coverage from multiple validated per-test artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageMerge {
    /// Producing VVM version.
    producer_version: Arc<str>,
    /// Applied status-inclusion policy.
    policy: CoverageMergePolicy,
    /// Canonically ordered input records.
    inputs: Box<[CoverageMergeInput]>,
    /// Merged groups ordered by instance path.
    groups: Box<[CoverageArtifactGroup]>,
    /// Recomputed exact summary.
    summary: CoverageMergeSummary,
}

impl CoverageMerge {
    /// Merged-document format name.
    pub const FORMAT_NAME: &'static str = "vvm-functional-coverage-merge";
    /// Supported merged schema version.
    pub const SCHEMA_VERSION: u32 = 1;
    /// Standard merged artifact suffix.
    pub const FILE_SUFFIX: &'static str = ".vvmcov-merged.json";

    /// Merges validated per-test artifacts deterministically.
    ///
    /// # Errors
    ///
    /// Returns an error for empty input, excluded input, incompatible definitions,
    /// persistence conversion failures, or checked-counter overflow.
    pub fn from_artifacts(
        policy: CoverageMergePolicy,
        artifacts: impl IntoIterator<Item = CoverageArtifact>,
    ) -> Result<Self, CoverageMergeError> {
        let mut artifacts = artifacts
            .into_iter()
            .map(|artifact| {
                let key = artifact
                    .to_json()
                    .map_err(|source| CoverageMergeError::Persistence { source })?;
                Ok((key, artifact))
            })
            .collect::<Result<Vec<_>, CoverageMergeError>>()?;
        if artifacts.is_empty() {
            return Err(CoverageMergeError::NoInputArtifacts);
        }

        artifacts.sort_unstable_by(|left, right| left.0.cmp(&right.0));

        let mut inputs = Vec::with_capacity(artifacts.len());
        let mut groups = BTreeMap::<String, (String, CoverageArtifactGroup)>::new();
        let mut counts = MergeCounts::default();

        for (_key, artifact) in artifacts {
            let producer_version: Arc<str> = Arc::from(artifact.producer_version());
            let test_name: Arc<str> = Arc::from(artifact.test_name());
            let status = artifact.test_status();
            let replay_token = artifact.replay_token();
            let summary = artifact.summary();
            let artifact_groups = artifact.into_groups();
            let included = policy.includes(status);
            counts.record(status, included)?;
            inputs.push(CoverageMergeInput {
                producer_version,
                test_name: Arc::clone(&test_name),
                test_status: status,
                replay_token,
                included,
                summary,
            });

            if !included {
                continue;
            }

            for incoming in artifact_groups.into_vec() {
                let path = incoming.instance_path().to_owned();
                if let Some(entry) = groups.get(&path) {
                    let existing_test = &entry.0;
                    let existing = &entry.1;
                    let merged = merge_group(existing, &incoming, existing_test, &test_name)?;
                    groups.insert(path, (existing_test.clone(), merged));
                } else {
                    groups.insert(path, (test_name.to_string(), incoming));
                }
            }
        }
        if counts.included == 0 {
            return Err(CoverageMergeError::NoIncludedArtifacts { policy });
        }

        let groups = groups
            .into_values()
            .map(|(_test, group)| group)
            .collect::<Vec<_>>();
        let summary = merge_summary(&groups, counts)?;
        Ok(Self {
            producer_version: Arc::from(env!("CARGO_PKG_VERSION")),
            policy,
            inputs: inputs.into_boxed_slice(),
            groups: groups.into_boxed_slice(),
            summary,
        })
    }

    /// Reads and merges explicit per-test artifact files.
    ///
    /// # Errors
    ///
    /// Returns an error for empty or duplicate paths, invalid artifacts, or merge failures.
    pub fn from_files(
        policy: CoverageMergePolicy,
        paths: impl IntoIterator<Item = PathBuf>,
    ) -> Result<Self, CoverageMergeError> {
        let mut paths = paths.into_iter().collect::<Vec<_>>();
        if paths.is_empty() {
            return Err(CoverageMergeError::NoInputArtifacts);
        }
        paths.sort_unstable();
        for pair in paths.windows(2) {
            let Some((first, remaining)) = pair.split_first() else {
                continue;
            };
            let Some(second) = remaining.first() else {
                continue;
            };
            if first == second {
                return Err(CoverageMergeError::DuplicateInputPath {
                    path: first.clone(),
                });
            }
        }
        let artifacts = paths
            .into_iter()
            .map(|path| {
                CoverageArtifact::read_from(&path)
                    .map_err(|source| CoverageMergeError::ReadArtifact { path, source })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::from_artifacts(policy, artifacts)
    }

    /// Serializes compact merged schema-v1 JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when JSON encoding fails.
    pub fn to_json(&self) -> Result<String, CoveragePersistenceError> {
        serde_json::to_string(&self.json_value())
            .map_err(|source| CoveragePersistenceError::JsonEncode { source })
    }

    /// Serializes pretty merged schema-v1 JSON with exactly one trailing newline.
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

    /// Parses and semantically validates merged schema-v1 JSON.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed, unsupported, or inconsistent documents.
    pub fn from_json(json: &str) -> Result<Self, CoveragePersistenceError> {
        let document = serde_json::from_str::<MergeDto>(json)
            .map_err(|source| CoveragePersistenceError::JsonDecode { source })?;

        Self::from_dto(document)
    }

    /// Atomically writes this merged document without overwriting a destination.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization or a filesystem operation fails.
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), CoveragePersistenceError> {
        let path = path.as_ref();
        let bytes = self.to_json_pretty()?.into_bytes();
        let parent = path
            .parent()
            .ok_or_else(|| invalid_data(path, "merged artifact path has no parent directory"))?;

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
            .ok_or_else(|| invalid_data(path, "merged artifact filename is not valid UTF-8"))?;
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

    /// Reads and validates one merged coverage document.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read or the document is invalid.
    pub fn read_from(path: impl AsRef<Path>) -> Result<Self, CoveragePersistenceError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| CoveragePersistenceError::Io {
            operation: crate::CoverageIoOperation::Read,
            path: path.to_owned(),
            source,
        })?;

        Self::from_json(&text)
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
    /// Returns applied status-inclusion policy.
    #[must_use]
    pub const fn policy(&self) -> CoverageMergePolicy {
        self.policy
    }
    /// Returns canonically ordered input records.
    #[must_use]
    pub fn inputs(&self) -> &[CoverageMergeInput] {
        &self.inputs
    }
    /// Returns merged groups ordered by instance path.
    #[must_use]
    pub fn groups(&self) -> &[CoverageArtifactGroup] {
        &self.groups
    }
    /// Finds a group by exact instance path.
    #[must_use]
    pub fn group(&self, instance_path: &str) -> Option<&CoverageArtifactGroup> {
        self.groups
            .iter()
            .find(|group| group.instance_path() == instance_path)
    }
    /// Returns the recomputed exact summary.
    #[must_use]
    pub const fn summary(&self) -> CoverageMergeSummary {
        self.summary
    }
    /// Returns exact merged coverage.
    #[must_use]
    pub const fn coverage(&self) -> CoverageRatio {
        self.summary.coverage()
    }

    /// Builds the strict merged schema-v1 JSON value.
    fn json_value(&self) -> Value {
        json!({
            "format": Self::FORMAT_NAME,
            "schema_version": Self::SCHEMA_VERSION,
            "producer": { "name": "vvm-rs", "version": self.producer_version.as_ref() },
            "policy": self.policy.to_string(),
            "inputs": self.inputs.iter().map(input_json).collect::<Vec<_>>(),
            "summary": merge_summary_json(self.summary),
            "groups": self.groups.iter().map(crate::coverage::artifact::group_json).collect::<Vec<_>>(),
        })
    }

    /// Reconstructs a strictly validated merged document.
    fn from_dto(document: MergeDto) -> Result<Self, CoveragePersistenceError> {
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
        let policy = CoverageMergePolicy::from_str(&document.policy)?;
        if document.inputs.is_empty() {
            return Err(invalid(
                "inputs",
                "a merged document must contain input records",
            ));
        }
        let inputs = document
            .inputs
            .into_iter()
            .enumerate()
            .map(|(index, input)| input_from_dto(input, policy, index))
            .collect::<Result<Vec<_>, _>>()?;
        if !inputs.iter().any(CoverageMergeInput::included) {
            return Err(invalid("inputs", "the policy excludes every input"));
        }
        if inputs.windows(2).any(|pair| {
            let Some((first, remaining)) = pair.split_first() else {
                return false;
            };
            let Some(second) = remaining.first() else {
                return false;
            };
            input_key(first) > input_key(second)
        }) {
            return Err(invalid(
                "inputs",
                "input records are not in canonical order",
            ));
        }

        let groups = document
            .groups
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                let dto = serde_json::from_value::<crate::coverage::artifact::GroupDto>(value)
                    .map_err(|source| CoveragePersistenceError::JsonDecode { source })?;
                crate::coverage::artifact::group_from_dto(dto, index)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if groups.windows(2).any(|pair| {
            let Some((first, remaining)) = pair.split_first() else {
                return false;
            };
            let Some(second) = remaining.first() else {
                return false;
            };
            first.instance_path() >= second.instance_path()
        }) {
            return Err(invalid(
                "groups",
                "groups are not in strictly increasing instance-path order",
            ));
        }

        let counts = inputs
            .iter()
            .try_fold(MergeCounts::default(), |mut counts, input| {
                counts.record(input.test_status(), input.included())?;
                Ok::<_, CoverageMergeError>(counts)
            })
            .map_err(|error| match error {
                CoverageMergeError::CountOverflow { counter } => {
                    invalid("summary", &format!("count overflow for {counter}"))
                }
                _ => invalid("summary", "could not count inputs"),
            })?;
        let computed = merge_summary(&groups, counts).map_err(merge_persistence_error)?;
        let stored = merge_summary_from_dto(document.summary)?;
        if computed != stored {
            return Err(invalid(
                "summary",
                "stored summary does not match recomputed merged data",
            ));
        }
        Ok(Self {
            producer_version: Arc::from(document.producer.version),
            policy,
            inputs: inputs.into_boxed_slice(),
            groups: groups.into_boxed_slice(),
            summary: computed,
        })
    }
}

/// Process-local suffix used only to avoid temporary-file collisions.
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Checked provenance counters accumulated while processing inputs.
#[derive(Default, Clone, Copy)]
struct MergeCounts {
    /// Total artifacts.
    artifacts: usize,
    /// Included artifacts.
    included: usize,
    /// Excluded artifacts.
    excluded: usize,
    /// Passed artifacts.
    passed: usize,
    /// Failed artifacts.
    failed: usize,
    /// Errored artifacts.
    errored: usize,
}
impl MergeCounts {
    /// Adds one input status to the checked counters.
    fn record(&mut self, status: TestStatus, included: bool) -> Result<(), CoverageMergeError> {
        self.artifacts = add_count(self.artifacts, 1, CoverageMergeCountKind::Artifacts)?;
        if included {
            self.included = add_count(self.included, 1, CoverageMergeCountKind::IncludedArtifacts)?;
        } else {
            self.excluded = add_count(self.excluded, 1, CoverageMergeCountKind::ExcludedArtifacts)?;
        }
        match status {
            TestStatus::Passed => {
                self.passed = add_count(self.passed, 1, CoverageMergeCountKind::PassedArtifacts)?;
            }
            TestStatus::Failed => {
                self.failed = add_count(self.failed, 1, CoverageMergeCountKind::FailedArtifacts)?;
            }
            TestStatus::Error => {
                self.errored =
                    add_count(self.errored, 1, CoverageMergeCountKind::ErroredArtifacts)?;
            }
        }
        Ok(())
    }
}

/// Merges two structurally compatible group snapshots.
fn merge_group(
    existing: &CoverageArtifactGroup,
    incoming: &CoverageArtifactGroup,
    existing_test: &str,
    incoming_test: &str,
) -> Result<CoverageArtifactGroup, CoverageMergeError> {
    let path = existing.instance_path().to_owned();
    if existing.definition_name() != incoming.definition_name()
        || existing.definition_revision() != incoming.definition_revision()
        || existing.definition_fingerprint() != incoming.definition_fingerprint()
    {
        return Err(CoverageMergeError::IncompatibleDefinition {
            instance_path: path.into_boxed_str(),
            existing_test: existing_test.into(),
            incoming_test: incoming_test.into(),
            existing_definition: existing.definition_name().into(),
            incoming_definition: incoming.definition_name().into(),
            existing_revision: existing.definition_revision(),
            incoming_revision: incoming.definition_revision(),
            existing_fingerprint: Box::new(existing.definition_fingerprint()),
            incoming_fingerprint: Box::new(incoming.definition_fingerprint()),
        });
    }
    compare_group_structure(
        existing.snapshot(),
        incoming.snapshot(),
        existing.definition_fingerprint(),
    )?;
    let items = existing
        .snapshot()
        .items()
        .iter()
        .zip(incoming.snapshot().items())
        .map(|(left, right)| {
            merge_item(
                left,
                right,
                existing.instance_path(),
                existing.definition_fingerprint(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let summary = group_summary(&items)?;
    let snapshot = CoverageGroupSnapshot::from_parts(
        Arc::from(existing.definition_name()),
        existing.definition_revision(),
        Arc::from(existing.instance_path()),
        items.into_boxed_slice(),
        summary,
    );
    let fingerprint = CoverageDefinitionFingerprint::from_group(&snapshot)
        .map_err(|source| CoverageMergeError::Persistence { source })?;
    if fingerprint != existing.definition_fingerprint() {
        return Err(CoverageMergeError::DefinitionStructureMismatch {
            instance_path: path,
            fingerprint: existing.definition_fingerprint(),
            path: "fingerprint".to_owned(),
            reason: "recomputed fingerprint changed while merging counters".to_owned(),
        });
    }
    Ok(CoverageArtifactGroup::from_parts(snapshot, fingerprint))
}

/// Defensively compares group structure beyond its fingerprint.
fn compare_group_structure(
    existing: &CoverageGroupSnapshot,
    incoming: &CoverageGroupSnapshot,
    fingerprint: CoverageDefinitionFingerprint,
) -> Result<(), CoverageMergeError> {
    if existing.items().len() != incoming.items().len() {
        return mismatch(existing, fingerprint, "items", "item counts differ");
    }
    let instance_path = existing.instance_path();
    for (index, (existing_item, incoming_item)) in
        existing.items().iter().zip(incoming.items()).enumerate()
    {
        if existing_item.kind() != incoming_item.kind()
            || existing_item.name() != incoming_item.name()
        {
            return mismatch(
                existing,
                fingerprint,
                &format!("items[{index}]"),
                "item kind or name differs",
            );
        }
        if let (Some(existing_point), Some(incoming_point)) =
            (existing_item.as_coverpoint(), incoming_item.as_coverpoint())
        {
            compare_point(
                existing_point,
                incoming_point,
                instance_path,
                fingerprint,
                index,
            )?;
        } else if let (Some(existing_cross), Some(incoming_cross)) =
            (existing_item.as_cross2(), incoming_item.as_cross2())
        {
            compare_cross(
                existing_cross,
                incoming_cross,
                instance_path,
                fingerprint,
                index,
            )?;
        } else {
            return mismatch(
                existing,
                fingerprint,
                &format!("items[{index}]"),
                "item kind differs",
            );
        }
    }
    Ok(())
}

/// Constructs a structured definition mismatch.
fn mismatch<T>(
    group: &CoverageGroupSnapshot,
    fingerprint: CoverageDefinitionFingerprint,
    path: &str,
    reason: &str,
) -> Result<T, CoverageMergeError> {
    Err(CoverageMergeError::DefinitionStructureMismatch {
        instance_path: group.instance_path().to_owned(),
        fingerprint,
        path: path.to_owned(),
        reason: reason.to_owned(),
    })
}

/// Compares one coverpoint definition in declaration order.
fn compare_point(
    left: &CoverpointSnapshot,
    right: &CoverpointSnapshot,
    group_path: &str,
    fingerprint: CoverageDefinitionFingerprint,
    item: usize,
) -> Result<(), CoverageMergeError> {
    if left.bins().len() != right.bins().len() {
        return Err(structure_error(
            group_path,
            fingerprint,
            format!("items[{item}].bins"),
            "bin counts differ",
        ));
    }
    for (index, (left_bin, right_bin)) in left.bins().iter().zip(right.bins()).enumerate() {
        if left_bin.id() != right_bin.id()
            || left_bin.name() != right_bin.name()
            || left_bin.kind() != right_bin.kind()
            || left_bin.matcher_kind() != right_bin.matcher_kind()
            || left_bin.matcher_operand_count() != right_bin.matcher_operand_count()
            || left_bin.required_hits() != right_bin.required_hits()
        {
            return Err(structure_error(
                group_path,
                fingerprint,
                format!("items[{item}].bins[{index}]"),
                "bin definition differs",
            ));
        }
    }
    Ok(())
}

/// Compares one cross definition in row-major order.
fn compare_cross(
    left: &Cross2Snapshot,
    right: &Cross2Snapshot,
    group_path: &str,
    fingerprint: CoverageDefinitionFingerprint,
    item: usize,
) -> Result<(), CoverageMergeError> {
    if left.left_coverpoint_name() != right.left_coverpoint_name()
        || left.right_coverpoint_name() != right.right_coverpoint_name()
        || left.bins().len() != right.bins().len()
    {
        return Err(structure_error(
            group_path,
            fingerprint,
            format!("items[{item}]"),
            "cross axes or bin count differ",
        ));
    }
    for (index, (left_bin, right_bin)) in left.bins().iter().zip(right.bins()).enumerate() {
        if left_bin.id() != right_bin.id()
            || left_bin.left_bin_id() != right_bin.left_bin_id()
            || left_bin.left_bin_name() != right_bin.left_bin_name()
            || left_bin.right_bin_id() != right_bin.right_bin_id()
            || left_bin.right_bin_name() != right_bin.right_bin_name()
            || left_bin.required_hits() != right_bin.required_hits()
        {
            return Err(structure_error(
                group_path,
                fingerprint,
                format!("items[{item}].bins[{index}]"),
                "cross-bin definition differs",
            ));
        }
    }
    Ok(())
}

/// Constructs a structure mismatch for one logical definition path.
fn structure_error(
    instance_path: &str,
    fingerprint: CoverageDefinitionFingerprint,
    path: String,
    reason: &str,
) -> CoverageMergeError {
    CoverageMergeError::DefinitionStructureMismatch {
        instance_path: instance_path.to_owned(),
        fingerprint,
        path,
        reason: reason.to_owned(),
    }
}

/// Merges one already-compatible coverage item.
fn merge_item(
    left: &CoverageItemSnapshot,
    right: &CoverageItemSnapshot,
    instance_path: &str,
    fingerprint: CoverageDefinitionFingerprint,
) -> Result<CoverageItemSnapshot, CoverageMergeError> {
    if let (Some(left), Some(right)) = (left.as_coverpoint(), right.as_coverpoint()) {
        Ok(CoverageItemSnapshot::Coverpoint(merge_point(
            left,
            right,
            instance_path,
        )?))
    } else if let (Some(left), Some(right)) = (left.as_cross2(), right.as_cross2()) {
        Ok(CoverageItemSnapshot::Cross2(merge_cross(
            left,
            right,
            instance_path,
        )?))
    } else {
        Err(CoverageMergeError::DefinitionStructureMismatch {
            instance_path: instance_path.to_owned(),
            fingerprint,
            path: "items".to_owned(),
            reason: "item kind changed during merge".to_owned(),
        })
    }
}

/// Sums coverpoint counters and reconstructs its coverage ratio.
fn merge_point(
    left: &CoverpointSnapshot,
    right: &CoverpointSnapshot,
    instance_path: &str,
) -> Result<CoverpointSnapshot, CoverageMergeError> {
    let samples = add_runtime(
        left.sample_count(),
        right.sample_count(),
        instance_path,
        left.name(),
        None,
        CoverageMergeCounterKind::CoverpointSamples,
    )?;
    let ignored = add_runtime(
        left.ignored_sample_count(),
        right.ignored_sample_count(),
        instance_path,
        left.name(),
        None,
        CoverageMergeCounterKind::IgnoredSamples,
    )?;
    let illegal = add_runtime(
        left.illegal_sample_count(),
        right.illegal_sample_count(),
        instance_path,
        left.name(),
        None,
        CoverageMergeCounterKind::IllegalSamples,
    )?;
    let unmatched = add_runtime(
        left.unmatched_sample_count(),
        right.unmatched_sample_count(),
        instance_path,
        left.name(),
        None,
        CoverageMergeCounterKind::UnmatchedSamples,
    )?;
    let bins = left
        .bins()
        .iter()
        .zip(right.bins())
        .map(|(left_bin, right_bin)| {
            let hits = add_runtime(
                left_bin.hits(),
                right_bin.hits(),
                instance_path,
                left.name(),
                Some(left_bin.name().to_owned()),
                CoverageMergeCounterKind::CoverpointBinHits,
            )?;
            Ok(CoverpointBinSnapshot::from_parts(
                left_bin.id(),
                Arc::from(left_bin.name()),
                left_bin.kind(),
                left_bin.matcher_kind(),
                left_bin.matcher_operand_count(),
                hits,
                left_bin.required_hits(),
            ))
        })
        .collect::<Result<Vec<_>, CoverageMergeError>>()?;
    let coverage = point_ratio(&bins)?;
    Ok(CoverpointSnapshot::from_parts(
        Arc::from(left.name()),
        bins.into_boxed_slice(),
        samples,
        ignored,
        illegal,
        unmatched,
        coverage,
    ))
}

/// Sums cross counters and reconstructs its coverage ratio.
fn merge_cross(
    left: &Cross2Snapshot,
    right: &Cross2Snapshot,
    instance_path: &str,
) -> Result<Cross2Snapshot, CoverageMergeError> {
    let samples = add_runtime(
        left.sample_count(),
        right.sample_count(),
        instance_path,
        left.name(),
        None,
        CoverageMergeCounterKind::CrossSamples,
    )?;
    let skipped = add_runtime(
        left.skipped_sample_count(),
        right.skipped_sample_count(),
        instance_path,
        left.name(),
        None,
        CoverageMergeCounterKind::SkippedCrossSamples,
    )?;
    let bins = left
        .bins()
        .iter()
        .zip(right.bins())
        .map(|(left_bin, right_bin)| {
            let description = format!(
                "{} x {}",
                left_bin.left_bin_name(),
                left_bin.right_bin_name()
            );
            let hits = add_runtime(
                left_bin.hits(),
                right_bin.hits(),
                instance_path,
                left.name(),
                Some(description),
                CoverageMergeCounterKind::CrossBinHits,
            )?;
            Ok(CrossBinSnapshot::from_parts(
                left_bin.id(),
                left_bin.left_bin_id(),
                Arc::from(left_bin.left_bin_name()),
                left_bin.right_bin_id(),
                Arc::from(left_bin.right_bin_name()),
                hits,
                left_bin.required_hits(),
            ))
        })
        .collect::<Result<Vec<_>, CoverageMergeError>>()?;
    let coverage = cross_ratio(&bins)?;
    Ok(Cross2Snapshot::from_parts(
        Arc::from(left.name()),
        Arc::from(left.left_coverpoint_name()),
        Arc::from(left.right_coverpoint_name()),
        bins.into_boxed_slice(),
        samples,
        skipped,
        coverage,
    ))
}

/// Adds one runtime counter with merge context.
fn add_runtime(
    left: u64,
    right: u64,
    instance_path: &str,
    item: &str,
    bin: Option<String>,
    counter: CoverageMergeCounterKind,
) -> Result<u64, CoverageMergeError> {
    left.checked_add(right)
        .ok_or_else(|| CoverageMergeError::CounterOverflow {
            instance_path: instance_path.to_owned(),
            item: item.to_owned(),
            bin,
            counter,
        })
}
/// Adds one summary count with checked arithmetic.
fn add_count(
    left: usize,
    right: usize,
    counter: CoverageMergeCountKind,
) -> Result<usize, CoverageMergeError> {
    left.checked_add(right)
        .ok_or(CoverageMergeError::CountOverflow { counter })
}
/// Recomputes coverage for normal coverpoint bins.
fn point_ratio(bins: &[CoverpointBinSnapshot]) -> Result<CoverageRatio, CoverageMergeError> {
    let total = bins
        .iter()
        .filter(|bin| bin.kind() == BinKind::Normal)
        .count();
    let covered = bins.iter().filter(|bin| bin.covered()).count();
    let uncovered = total
        .checked_sub(covered)
        .ok_or(CoverageMergeError::CountOverflow {
            counter: CoverageMergeCountKind::UncoveredBins,
        })?;
    Ok(CoverageRatio::new(covered, uncovered, total))
}
/// Recomputes coverage for all generated cross bins.
fn cross_ratio(bins: &[CrossBinSnapshot]) -> Result<CoverageRatio, CoverageMergeError> {
    let covered = bins.iter().filter(|bin| bin.covered()).count();
    let uncovered = bins
        .len()
        .checked_sub(covered)
        .ok_or(CoverageMergeError::CountOverflow {
            counter: CoverageMergeCountKind::UncoveredBins,
        })?;
    Ok(CoverageRatio::new(covered, uncovered, bins.len()))
}

/// Recomputes an exact group summary from merged items.
fn group_summary(
    items: &[CoverageItemSnapshot],
) -> Result<CoverageGroupSummary, CoverageMergeError> {
    let coverpoints = items
        .iter()
        .filter(|item| item.as_coverpoint().is_some())
        .count();
    let crosses = items
        .iter()
        .filter(|item| item.as_cross2().is_some())
        .count();
    let coverage = sum_ratios(items.iter().map(CoverageItemSnapshot::coverage))?;
    Ok(CoverageGroupSummary::from_parts(
        items.len(),
        coverpoints,
        crosses,
        coverage,
    ))
}

/// Recomputes the exact merge summary from unique groups and provenance.
fn merge_summary(
    groups: &[CoverageArtifactGroup],
    counts: MergeCounts,
) -> Result<CoverageMergeSummary, CoverageMergeError> {
    let mut items = 0;
    let mut coverpoints = 0;
    let mut crosses = 0;
    for group in groups {
        items = add_count(
            items,
            group.summary().item_count(),
            CoverageMergeCountKind::Items,
        )?;
        coverpoints = add_count(
            coverpoints,
            group.summary().coverpoint_count(),
            CoverageMergeCountKind::Coverpoints,
        )?;
        crosses = add_count(
            crosses,
            group.summary().cross_count(),
            CoverageMergeCountKind::Crosses,
        )?;
    }
    let coverage = sum_ratios(groups.iter().map(CoverageArtifactGroup::coverage))?;
    Ok(CoverageMergeSummary {
        artifacts: counts.artifacts,
        included_artifacts: counts.included,
        excluded_artifacts: counts.excluded,
        passed_artifacts: counts.passed,
        failed_artifacts: counts.failed,
        errored_artifacts: counts.errored,
        groups: groups.len(),
        items,
        coverpoints,
        crosses,
        coverage,
    })
}

/// Adds exact coverage ratios with checked summary counts.
fn sum_ratios(
    mut values: impl Iterator<Item = CoverageRatio>,
) -> Result<CoverageRatio, CoverageMergeError> {
    values.try_fold(CoverageRatio::new(0, 0, 0), |total, value| {
        Ok(CoverageRatio::new(
            add_count(
                total.covered(),
                value.covered(),
                CoverageMergeCountKind::CoveredBins,
            )?,
            add_count(
                total.uncovered(),
                value.uncovered(),
                CoverageMergeCountKind::UncoveredBins,
            )?,
            add_count(
                total.total(),
                value.total(),
                CoverageMergeCountKind::TotalBins,
            )?,
        ))
    })
}

/// Strict merged schema-v1 document.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MergeDto {
    /// Format name.
    format: String,
    /// Schema version.
    schema_version: u32,
    /// Producer metadata.
    producer: MergeProducerDto,
    /// Inclusion policy.
    policy: String,
    /// Input provenance records.
    inputs: Vec<MergeInputDto>,
    /// Recomputed merged summary.
    summary: MergeSummaryDto,
    /// Shared per-test group DTO values.
    groups: Vec<Value>,
}

/// Strict producer metadata.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MergeProducerDto {
    /// Producer name.
    name: String,
    /// Producer version.
    version: String,
}

/// Strict merge input metadata.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MergeInputDto {
    /// Producing VVM version.
    producer_version: String,
    /// Persisted test metadata.
    test: MergeTestDto,
    /// Policy inclusion result.
    included: bool,
    /// Original per-test summary.
    summary: MergeSessionSummaryDto,
}

/// Strict test metadata.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MergeTestDto {
    /// Stable test name.
    name: String,
    /// Persisted test status.
    status: String,
    /// Optional replay token.
    replay_token: Option<String>,
}

/// Strict per-test session summary.
#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct MergeSessionSummaryDto {
    /// Group count.
    groups: usize,
    /// Item count.
    items: usize,
    /// Coverpoint count.
    coverpoints: usize,
    /// Cross count.
    crosses: usize,
    /// Exact coverage.
    coverage: MergeRatioDto,
}

/// Strict exact coverage ratio.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[derive(Clone, Copy)]
struct MergeRatioDto {
    /// Covered bins.
    covered: usize,
    /// Uncovered bins.
    uncovered: usize,
    /// Total bins.
    total: usize,
}

/// Strict merged summary.
#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct MergeSummaryDto {
    /// Input artifact count.
    artifacts: usize,
    /// Included artifact count.
    included_artifacts: usize,
    /// Excluded artifact count.
    excluded_artifacts: usize,
    /// Passed artifact count.
    passed_artifacts: usize,
    /// Failed artifact count.
    failed_artifacts: usize,
    /// Errored artifact count.
    errored_artifacts: usize,
    /// Unique group count.
    groups: usize,
    /// Unique item count.
    items: usize,
    /// Unique coverpoint count.
    coverpoints: usize,
    /// Unique cross count.
    crosses: usize,
    /// Exact coverage.
    coverage: MergeRatioDto,
}

/// Converts merge input provenance to its schema-v1 representation.
fn input_json(input: &CoverageMergeInput) -> Value {
    json!({
        "producer_version": input.producer_version(),
        "test": { "name": input.test_name(), "status": crate::coverage::artifact::status_text(input.test_status()), "replay_token": input.replay_token().map(|token| token.to_string()) },
        "included": input.included(),
        "summary": crate::coverage::artifact::summary_json(input.summary()),
    })
}

/// Converts a merge summary to its schema-v1 representation.
fn merge_summary_json(summary: CoverageMergeSummary) -> Value {
    json!({
        "artifacts": summary.artifact_count(),
        "included_artifacts": summary.included_artifact_count(),
        "excluded_artifacts": summary.excluded_artifact_count(),
        "passed_artifacts": summary.passed_artifact_count(),
        "failed_artifacts": summary.failed_artifact_count(),
        "errored_artifacts": summary.errored_artifact_count(),
        "groups": summary.group_count(),
        "items": summary.item_count(),
        "coverpoints": summary.coverpoint_count(),
        "crosses": summary.cross_count(),
        "coverage": crate::coverage::artifact::ratio_json(summary.coverage()),
    })
}

/// Validates one persisted merge input record.
fn input_from_dto(
    input: MergeInputDto,
    policy: CoverageMergePolicy,
    index: usize,
) -> Result<CoverageMergeInput, CoveragePersistenceError> {
    if input.producer_version.is_empty() {
        return Err(invalid(
            &format!("inputs[{index}].producer_version"),
            "producer version must not be empty",
        ));
    }
    if !crate::registry::is_valid_test_name(&input.test.name) {
        return Err(invalid(
            &format!("inputs[{index}].test.name"),
            "invalid test name",
        ));
    }
    let test_status = crate::coverage::artifact::parse_status(&input.test.status)?;
    if input.included != policy.includes(test_status) {
        return Err(invalid(
            &format!("inputs[{index}].included"),
            "inclusion flag does not match policy",
        ));
    }
    let replay_token = input
        .test
        .replay_token
        .as_deref()
        .map(|value| {
            ReplayToken::from_str(value).map_err(|_error| {
                invalid(
                    &format!("inputs[{index}].test.replay_token"),
                    "invalid replay token",
                )
            })
        })
        .transpose()?;
    let summary = session_summary_from_dto(input.summary, index)?;
    Ok(CoverageMergeInput {
        producer_version: Arc::from(input.producer_version),
        test_name: Arc::from(input.test.name),
        test_status,
        replay_token,
        included: input.included,
        summary,
    })
}

/// Reconstructs one persisted per-test session summary.
fn session_summary_from_dto(
    summary: MergeSessionSummaryDto,
    index: usize,
) -> Result<CoverageSessionSummary, CoveragePersistenceError> {
    let coverage = ratio_from_dto(
        summary.coverage,
        &format!("inputs[{index}].summary.coverage"),
    )?;
    Ok(CoverageSessionSummary::from_parts(
        summary.groups,
        summary.items,
        summary.coverpoints,
        summary.crosses,
        coverage,
    ))
}

/// Reconstructs one persisted merged summary.
fn merge_summary_from_dto(
    summary: MergeSummaryDto,
) -> Result<CoverageMergeSummary, CoveragePersistenceError> {
    let coverage = ratio_from_dto(summary.coverage, "summary.coverage")?;
    Ok(CoverageMergeSummary {
        artifacts: summary.artifacts,
        included_artifacts: summary.included_artifacts,
        excluded_artifacts: summary.excluded_artifacts,
        passed_artifacts: summary.passed_artifacts,
        failed_artifacts: summary.failed_artifacts,
        errored_artifacts: summary.errored_artifacts,
        groups: summary.groups,
        items: summary.items,
        coverpoints: summary.coverpoints,
        crosses: summary.crosses,
        coverage,
    })
}

/// Validates and reconstructs one exact ratio.
fn ratio_from_dto(
    ratio: MergeRatioDto,
    path: &str,
) -> Result<CoverageRatio, CoveragePersistenceError> {
    let combined = ratio
        .covered
        .checked_add(ratio.uncovered)
        .ok_or_else(|| invalid(path, "covered and uncovered counts overflow"))?;
    if combined != ratio.total {
        return Err(invalid(path, "covered plus uncovered must equal total"));
    }
    Ok(CoverageRatio::new(
        ratio.covered,
        ratio.uncovered,
        ratio.total,
    ))
}

/// Produces the deterministic ordering key for persisted input metadata.
fn input_key(input: &CoverageMergeInput) -> String {
    format!("{input:?}")
}

/// Constructs one semantic persistence error.
fn invalid(path: &str, reason: &str) -> CoveragePersistenceError {
    CoveragePersistenceError::InvalidData {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}

/// Constructs one persistence error associated with a filesystem path.
fn invalid_data(path: &Path, reason: &str) -> CoveragePersistenceError {
    invalid(&path.display().to_string(), reason)
}

/// Converts an internal merge failure into persisted-document validation data.
fn merge_persistence_error(error: CoverageMergeError) -> CoveragePersistenceError {
    match error {
        CoverageMergeError::Persistence { source } => source,
        CoverageMergeError::CountOverflow { counter } => {
            invalid("summary", &format!("count overflow for {counter}"))
        }
        _ => invalid("summary", "could not recompute merged summary"),
    }
}

#[cfg(test)]
mod tests {
    use super::{CoverageMerge, CoverageMergeError, CoverageMergePolicy};
    use crate::{
        Bin, CoverageArtifact, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor,
        CoverageItemRef, CoverageSession, Coverpoint, TestStatus,
    };

    struct Group {
        instance: CoverageGroupInstance,
        point: Coverpoint<u8>,
    }

    impl CoverageGroup for Group {
        fn instance(&self) -> &CoverageGroupInstance {
            &self.instance
        }

        fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
            visitor.visit(CoverageItemRef::coverpoint(&self.point));
        }
    }

    fn artifact(
        test_name: &str,
        status: TestStatus,
        samples: usize,
    ) -> Result<CoverageArtifact, Box<dyn std::error::Error>> {
        let mut point = Coverpoint::builder("opcode")
            .bin(Bin::value("read", 1_u8).at_least(2))
            .build()?;

        for _ in 0..samples {
            point.sample(&1)?;
        }

        let group = Group {
            instance: CoverageGroupInstance::new("decoder", "dut.decoder")?,
            point,
        };
        let mut session = CoverageSession::new(test_name)?;
        session.capture(&group)?;
        let snapshot = session.finish().ok_or("missing session snapshot")?;

        Ok(CoverageArtifact::from_session(status, None, snapshot)?)
    }

    #[test]
    fn default_policy_is_passed_only() {
        assert_eq!(
            CoverageMergePolicy::default(),
            CoverageMergePolicy::PassedOnly
        );
    }

    #[test]
    fn policy_inclusion_is_explicit() {
        assert!(CoverageMergePolicy::passed_only().includes(TestStatus::Passed));
        assert!(!CoverageMergePolicy::passed_only().includes(TestStatus::Failed));
        assert!(CoverageMergePolicy::passed_and_failed().includes(TestStatus::Failed));
        assert!(!CoverageMergePolicy::passed_and_failed().includes(TestStatus::Error));
        assert!(CoverageMergePolicy::all().includes(TestStatus::Error));
    }

    #[test]
    fn policy_round_trips_stable_strings() -> Result<(), Box<dyn std::error::Error>> {
        for policy in [
            CoverageMergePolicy::PassedOnly,
            CoverageMergePolicy::PassedAndFailed,
            CoverageMergePolicy::All,
        ] {
            assert_eq!(policy.to_string().parse::<CoverageMergePolicy>()?, policy);
        }
        Ok(())
    }

    #[test]
    fn empty_artifact_merge_fails() {
        let error = CoverageMerge::from_artifacts(CoverageMergePolicy::passed_only(), [])
            .expect_err("empty merges must fail");

        assert!(matches!(error, CoverageMergeError::NoInputArtifacts));
    }

    #[test]
    fn merged_coverpoint_hits_can_satisfy_threshold() -> Result<(), Box<dyn std::error::Error>> {
        let left = artifact("decoder_smoke", TestStatus::Passed, 1)?;
        let right = artifact("decoder_random", TestStatus::Passed, 1)?;

        let merged =
            CoverageMerge::from_artifacts(CoverageMergePolicy::passed_only(), [left, right])?;
        let group = merged.group("dut.decoder").ok_or("missing merged group")?;
        let point = group
            .snapshot()
            .item("opcode")
            .and_then(crate::CoverageItemSnapshot::as_coverpoint)
            .ok_or("missing merged coverpoint")?;

        assert_eq!(point.sample_count(), 2);
        assert_eq!(point.bins().first().ok_or("missing merged bin")?.hits(), 2);
        assert_eq!(point.coverage().covered(), 1);
        assert_eq!(merged.coverage().covered(), 1);

        let json = merged.to_json()?;
        assert_eq!(CoverageMerge::from_json(&json)?, merged);
        Ok(())
    }
}
