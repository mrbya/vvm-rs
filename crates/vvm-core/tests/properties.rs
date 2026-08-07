//! Reproducible public-contract properties for deterministic core primitives.

use std::fs;
use std::path::PathBuf;

use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, RngSeed};
use serde_json::Value;
use vvm_core::{
    Bin, CoverageArtifact, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor,
    CoverageItemRef, CoverageMerge, CoverageMergeError, CoverageMergePolicy,
    CoveragePersistenceError, CoverageSession, Coverpoint, Cross2, RandomContext, ReplayToken,
    Seed, SimulationTime, TestStatus, TimeStep, extract_signed, extract_unsigned, insert_signed,
    insert_unsigned,
};

fn property_config() -> ProptestConfig {
    ProptestConfig {
        cases: 64,
        rng_seed: RngSeed::Fixed(0x4f1b_12c3_d4e5_f607),
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(property_config())]

    #[test]
    fn unsigned_packed_round_trip_preserves_unaffected_bits(
        original in prop::array::uniform4(any::<u32>()),
        offset in 0_usize..=64,
        width in 1_usize..=64,
        value in any::<u64>(),
    ) {
        let mut words = original;

        insert_unsigned(&mut words, 128, offset, width, value)?;
        let extracted = extract_unsigned(&words, 128, offset, width)?;
        let expected = value & width_mask(width);
        let field_end = offset.checked_add(width).unwrap_or_default();

        prop_assert_eq!(extracted, expected);

        for bit in 0..128 {
            if bit < offset || bit >= field_end {
                prop_assert_eq!(bit_at(&words, bit), bit_at(&original, bit));
            }
        }
    }

    #[test]
    fn signed_packed_round_trip_sign_extends_low_bits(
        original in prop::array::uniform4(any::<u32>()),
        offset in 0_usize..=64,
        width in 1_usize..=63,
        value in any::<i64>(),
    ) {
        let mut words = original;

        insert_signed(&mut words, 128, offset, width, value)?;
        let extracted = extract_signed(&words, 128, offset, width)?;
        let expected = sign_extend(value, width);

        prop_assert_eq!(extracted, expected);
    }

    #[test]
    fn replay_token_reconstructs_the_same_mixed_stream(
        seed in any::<u64>(),
        operations in prop::collection::vec(0_u8..3, 0..64),
    ) {
        let mut original = RandomContext::new(Seed::new(seed));
        let token = original.replay_token();
        let mut replayed = RandomContext::from_replay(token);

        for operation in operations {
            match operation {
                0 => prop_assert_eq!(original.next_u32(), replayed.next_u32()),
                1 => prop_assert_eq!(original.next_u64(), replayed.next_u64()),
                _ => prop_assert_eq!(original.next_bool(), replayed.next_bool()),
            }
        }

        prop_assert_eq!(token, ReplayToken::new(Seed::new(seed)));
    }

    #[test]
    fn simulation_time_checked_add_matches_u64_checked_add(
        start in any::<u64>(),
        delta in 1_u64..,
    ) {
        let step = TimeStep::new(delta)?;
        let actual = SimulationTime::from_ticks(start).checked_add(step);
        let expected = start.checked_add(delta).map(SimulationTime::from_ticks);

        prop_assert_eq!(actual, expected);
    }
}

const fn width_mask(width: usize) -> u64 {
    if width == 64 {
        u64::MAX
    } else {
        match 64_usize.checked_sub(width) {
            Some(shift) => u64::MAX >> shift,
            None => 0,
        }
    }
}

const fn sign_extend(value: i64, width: usize) -> i64 {
    let mask = width_mask(width);
    let raw = value.cast_unsigned() & mask;
    let sign_bit = 1_u64 << width.saturating_sub(1);

    if raw & sign_bit == 0 {
        return raw.cast_signed();
    }

    (raw | !mask).cast_signed()
}

fn bit_at(words: &[u32; 4], bit: usize) -> bool {
    let (word, shift) = match bit {
        0..=31 => (words.first().copied().unwrap_or_default(), bit),
        32..=63 => (
            words.get(1).copied().unwrap_or_default(),
            bit.saturating_sub(32),
        ),
        64..=95 => (
            words.get(2).copied().unwrap_or_default(),
            bit.saturating_sub(64),
        ),
        _ => (
            words.get(3).copied().unwrap_or_default(),
            bit.saturating_sub(96),
        ),
    };
    let shift = u32::try_from(shift).unwrap_or_default();

    word.checked_shr(shift).is_some_and(|value| value & 1 != 0)
}

struct CoverageGroupFixture {
    instance: CoverageGroupInstance,
    left: Coverpoint<u8>,
    right: Coverpoint<u8>,
    cross: Cross2,
}

impl CoverageGroup for CoverageGroupFixture {
    fn instance(&self) -> &CoverageGroupInstance {
        &self.instance
    }

    fn visit_items(&self, visitor: &mut dyn CoverageGroupVisitor) {
        visitor.visit(CoverageItemRef::coverpoint(&self.left));
        visitor.visit(CoverageItemRef::coverpoint(&self.right));
        visitor.visit(CoverageItemRef::cross2(&self.cross));
    }
}

fn coverage_artifact(
    test_name: &str,
    status: TestStatus,
    definition: &str,
    instance_path: &str,
    samples: &[(u8, u8)],
) -> Result<CoverageArtifact, Box<dyn std::error::Error>> {
    let mut left = Coverpoint::builder("left")
        .bin(Bin::value("one", 1_u8))
        .bin(Bin::value("two", 2_u8))
        .build()?;
    let mut right = Coverpoint::builder("right")
        .bin(Bin::value("ten", 10_u8))
        .bin(Bin::value("twenty", 20_u8))
        .build()?;
    let mut cross = Cross2::builder("left_x_right", &left, &right).build()?;

    for &(left_value, right_value) in samples {
        let left_sample = left.sample(&left_value)?;
        let right_sample = right.sample(&right_value)?;

        cross.sample(&left_sample, &right_sample)?;
    }

    let group = CoverageGroupFixture {
        instance: CoverageGroupInstance::new(definition, instance_path)?,
        left,
        right,
        cross,
    };
    let mut session = CoverageSession::new(test_name)?;
    session.capture(&group)?;
    let snapshot = session
        .finish()
        .ok_or("missing coverage session snapshot")?;

    Ok(CoverageArtifact::from_session(status, None, snapshot)?)
}

fn temporary_coverage_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vvm-core-{name}-{}", std::process::id()))
}

fn coverage_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("coverage")
        .join(name)
}

fn replace_fixture_placeholders(content: &str, fingerprint: &str) -> String {
    content
        .replace("__VVM_CORE_VERSION__", env!("CARGO_PKG_VERSION"))
        .replace("__DECODER_FINGERPRINT__", fingerprint)
}

fn normalize_runtime_json(content: &str, fingerprint: &str) -> String {
    content
        .replace(env!("CARGO_PKG_VERSION"), "__VVM_CORE_VERSION__")
        .replace(fingerprint, "__DECODER_FINGERPRINT__")
}

fn normalized_json_value(
    content: &str,
    fingerprint: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(&normalize_runtime_json(
        content,
        fingerprint,
    ))?)
}

fn coverage_fixture_text(
    name: &str,
    fingerprint: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(coverage_fixture_path(name))?;

    Ok(replace_fixture_placeholders(&content, fingerprint))
}

#[test]
fn coverage_artifact_persists_atomically_and_rejects_invalid_documents()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = coverage_artifact(
        "artifact_round_trip",
        TestStatus::Passed,
        "decoder",
        "dut.decoder",
        &[(1, 10), (2, 20)],
    )?;
    let directory = temporary_coverage_path("artifact-persistence");
    let path = directory.join("nested/decoder.vvmcov.json");

    drop(fs::remove_dir_all(&directory));

    artifact.write_to(&path)?;
    assert_eq!(CoverageArtifact::read_from(&path)?, artifact);
    assert!(artifact.to_json_pretty()?.ends_with('\n'));

    let duplicate = artifact
        .write_to(&path)
        .expect_err("persistence must not overwrite an existing artifact");
    assert!(matches!(
        duplicate,
        CoveragePersistenceError::DestinationExists { path: existing } if existing == path
    ));

    let json = artifact.to_json()?;
    let invalid_summary = json.replacen("\"groups\":1", "\"groups\":2", 1);
    let invalid_status = json.replacen("\"status\":\"passed\"", "\"status\":\"unknown\"", 1);
    let fingerprint = artifact
        .groups()
        .first()
        .ok_or("missing artifact group")?
        .definition_fingerprint()
        .to_string();
    let invalid_fingerprint = json.replacen(
        &fingerprint,
        "sha256-v1:0000000000000000000000000000000000000000000000000000000000000000",
        1,
    );

    assert!(matches!(
        CoverageArtifact::from_json(&invalid_summary),
        Err(CoveragePersistenceError::InvalidData { path: stored_path, reason })
            if stored_path == "summary" && reason == "counts disagree with groups"
    ));
    assert!(matches!(
        CoverageArtifact::from_json(&invalid_status),
        Err(CoveragePersistenceError::InvalidData { path: stored_path, reason })
            if stored_path == "test.status" && reason == "unknown test status"
    ));
    assert!(matches!(
        CoverageArtifact::from_json(&invalid_fingerprint),
        Err(CoveragePersistenceError::FingerprintMismatch { instance_path, .. })
            if instance_path == "dut.decoder"
    ));

    drop(fs::remove_dir_all(&directory));

    Ok(())
}

#[test]
fn coverage_artifact_matches_committed_schema_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let artifact = coverage_artifact(
        "artifact_round_trip",
        TestStatus::Passed,
        "decoder",
        "dut.decoder",
        &[(1, 10), (2, 20)],
    )?;
    let fingerprint = artifact
        .groups()
        .first()
        .ok_or("missing artifact group")?
        .definition_fingerprint()
        .to_string();
    let expected = serde_json::from_str::<Value>(&fs::read_to_string(coverage_fixture_path(
        "artifact-v1.json",
    ))?)?;
    let actual = normalized_json_value(&artifact.to_json_pretty()?, &fingerprint)?;

    assert_eq!(actual, expected);
    assert_eq!(
        CoverageArtifact::from_json(&coverage_fixture_text("artifact-v1.json", &fingerprint)?)?,
        artifact
    );

    let directory = temporary_coverage_path("artifact-fixture");
    let path = directory.join("artifact.vvmcov.json");

    drop(fs::remove_dir_all(&directory));
    fs::create_dir_all(&directory)?;
    fs::write(
        &path,
        coverage_fixture_text("artifact-v1.json", &fingerprint)?,
    )?;

    assert_eq!(CoverageArtifact::read_from(&path)?, artifact);

    drop(fs::remove_dir_all(&directory));

    Ok(())
}

#[test]
fn coverage_artifact_rejects_unsupported_schema_fixture() -> Result<(), Box<dyn std::error::Error>>
{
    let artifact = coverage_artifact(
        "artifact_round_trip",
        TestStatus::Passed,
        "decoder",
        "dut.decoder",
        &[(1, 10), (2, 20)],
    )?;
    let fingerprint = artifact
        .groups()
        .first()
        .ok_or("missing artifact group")?
        .definition_fingerprint()
        .to_string();
    let fixture = coverage_fixture_text("artifact-unsupported-schema.json", &fingerprint)?;

    assert!(matches!(
        CoverageArtifact::from_json(&fixture),
        Err(CoveragePersistenceError::UnsupportedSchemaVersion {
            found: 99,
            supported: 1
        })
    ));

    Ok(())
}

#[test]
fn coverage_merge_applies_status_policy_to_multi_artifact_counters()
-> Result<(), Box<dyn std::error::Error>> {
    let artifacts = || {
        Ok::<_, Box<dyn std::error::Error>>([
            coverage_artifact(
                "merge_passed",
                TestStatus::Passed,
                "decoder",
                "dut.decoder",
                &[(1, 10)],
            )?,
            coverage_artifact(
                "merge_failed",
                TestStatus::Failed,
                "decoder",
                "dut.decoder",
                &[(2, 20)],
            )?,
            coverage_artifact(
                "merge_error",
                TestStatus::Error,
                "decoder",
                "dut.decoder",
                &[(1, 20)],
            )?,
        ])
    };

    let passed = CoverageMerge::from_artifacts(CoverageMergePolicy::passed_only(), artifacts()?)?;
    let passed_and_failed =
        CoverageMerge::from_artifacts(CoverageMergePolicy::passed_and_failed(), artifacts()?)?;
    let all = CoverageMerge::from_artifacts(CoverageMergePolicy::all(), artifacts()?)?;

    assert_eq!(passed.coverage().covered(), 3);
    assert_eq!(passed_and_failed.coverage().covered(), 6);
    assert_eq!(all.coverage().covered(), 7);
    assert_eq!(all.coverage().total(), 8);
    assert_eq!(passed.summary().excluded_artifact_count(), 2);
    assert_eq!(passed_and_failed.summary().included_artifact_count(), 2);
    assert_eq!(all.summary().passed_artifact_count(), 1);
    assert_eq!(all.summary().failed_artifact_count(), 1);
    assert_eq!(all.summary().errored_artifact_count(), 1);
    assert!(
        all.inputs()
            .iter()
            .all(vvm_core::CoverageMergeInput::included)
    );

    let group = all.group("dut.decoder").ok_or("missing merged group")?;
    let cross = group
        .snapshot()
        .item("left_x_right")
        .and_then(|item| item.as_cross2())
        .ok_or("missing merged cross")?;
    assert_eq!(cross.sample_count(), 3);

    let directory = temporary_coverage_path("merge-persistence");
    let path = directory.join("merged/decoder.vvmcov-merged.json");

    drop(fs::remove_dir_all(&directory));

    all.write_to(&path)?;
    assert_eq!(CoverageMerge::from_json(&all.to_json()?)?, all);
    assert_eq!(CoverageMerge::read_from(&path)?, all);
    assert!(all.to_json_pretty()?.ends_with('\n'));

    drop(fs::remove_dir_all(&directory));

    let excluded = CoverageMerge::from_artifacts(
        CoverageMergePolicy::passed_only(),
        [coverage_artifact(
            "merge_excluded",
            TestStatus::Failed,
            "decoder",
            "dut.decoder",
            &[(1, 10)],
        )?],
    )
    .expect_err("passed-only merges must reject inputs excluded by policy");
    assert!(matches!(
        excluded,
        CoverageMergeError::NoIncludedArtifacts {
            policy: CoverageMergePolicy::PassedOnly
        }
    ));

    Ok(())
}

#[test]
fn coverage_merge_rejects_same_path_with_incompatible_definitions()
-> Result<(), Box<dyn std::error::Error>> {
    let left = coverage_artifact(
        "compatible_first",
        TestStatus::Passed,
        "decoder",
        "dut.decoder",
        &[(1, 10)],
    )?;
    let right = coverage_artifact(
        "incompatible_second",
        TestStatus::Passed,
        "other_decoder",
        "dut.decoder",
        &[(1, 10)],
    )?;

    let error = CoverageMerge::from_artifacts(CoverageMergePolicy::all(), [left, right])
        .expect_err("definitions at the same instance path must agree");

    assert!(matches!(
        error,
        CoverageMergeError::IncompatibleDefinition {
            instance_path,
            existing_definition,
            incoming_definition,
            ..
        } if instance_path.as_ref() == "dut.decoder"
            && ((existing_definition.as_ref() == "decoder"
                && incoming_definition.as_ref() == "other_decoder")
                || (existing_definition.as_ref() == "other_decoder"
                    && incoming_definition.as_ref() == "decoder"))
    ));

    Ok(())
}

#[test]
fn coverage_merge_matches_committed_schema_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let merge = CoverageMerge::from_artifacts(
        CoverageMergePolicy::all(),
        [
            coverage_artifact(
                "merge_passed",
                TestStatus::Passed,
                "decoder",
                "dut.decoder",
                &[(1, 10)],
            )?,
            coverage_artifact(
                "merge_failed",
                TestStatus::Failed,
                "decoder",
                "dut.decoder",
                &[(2, 20)],
            )?,
            coverage_artifact(
                "merge_error",
                TestStatus::Error,
                "decoder",
                "dut.decoder",
                &[(1, 20)],
            )?,
        ],
    )?;
    let fingerprint = merge
        .groups()
        .first()
        .ok_or("missing merged group")?
        .definition_fingerprint()
        .to_string();
    let expected = serde_json::from_str::<Value>(&fs::read_to_string(coverage_fixture_path(
        "merge-v1.json",
    ))?)?;
    let actual = normalized_json_value(&merge.to_json_pretty()?, &fingerprint)?;

    assert_eq!(actual, expected);
    assert_eq!(
        CoverageMerge::from_json(&coverage_fixture_text("merge-v1.json", &fingerprint)?)?,
        merge
    );

    let directory = temporary_coverage_path("merge-fixture");
    let path = directory.join("merge.vvmcov-merged.json");

    drop(fs::remove_dir_all(&directory));
    fs::create_dir_all(&directory)?;
    fs::write(&path, coverage_fixture_text("merge-v1.json", &fingerprint)?)?;

    assert_eq!(CoverageMerge::read_from(&path)?, merge);

    drop(fs::remove_dir_all(&directory));

    Ok(())
}

#[test]
fn coverage_merge_rejects_unsupported_schema_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let merge = CoverageMerge::from_artifacts(
        CoverageMergePolicy::all(),
        [coverage_artifact(
            "merge_passed",
            TestStatus::Passed,
            "decoder",
            "dut.decoder",
            &[(1, 10)],
        )?],
    )?;
    let fingerprint = merge
        .groups()
        .first()
        .ok_or("missing merged group")?
        .definition_fingerprint()
        .to_string();
    let fixture = coverage_fixture_text("merge-unsupported-schema.json", &fingerprint)?;

    assert!(matches!(
        CoverageMerge::from_json(&fixture),
        Err(CoveragePersistenceError::UnsupportedSchemaVersion {
            found: 99,
            supported: 1
        })
    ));

    Ok(())
}
