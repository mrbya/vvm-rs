use std::fmt::Write as _;

use super::report::{include_coverpoint_bin, include_cross_bin, ratio_text, short_ratio_text};
use super::{CoverageBinDetail, CoverageReport};
use crate::{BinKind, CoverageItemSnapshot, TestStatus};

/// Renders the complete deterministic plain-text document.
pub(super) fn render(report: CoverageReport<'_>) -> String {
    let mut output = String::from("VVM Functional Coverage\n=======================\n\n");
    let summary = report.merge().summary();
    write!(
        output,
        "Overall\n-------\nCoverage: {}\nPolicy: {}\nArtifacts: {} total, {} included, {} \
         excluded\nStatuses: {} passed, {} failed, {} errored\nStructure: {} groups, {} \
         coverpoints, {} crosses\n",
        ratio_text(summary.coverage()),
        report.merge().policy(),
        summary.artifact_count(),
        summary.included_artifact_count(),
        summary.excluded_artifact_count(),
        summary.passed_artifact_count(),
        summary.failed_artifact_count(),
        summary.errored_artifact_count(),
        summary.group_count(),
        summary.coverpoint_count(),
        summary.cross_count()
    )
    .expect("writing to String cannot fail");

    if report.options().includes_inputs() {
        output.push_str("\nInputs\n------\n");
        for input in report.merge().inputs() {
            let contribution = if input.included() {
                "included"
            } else {
                "excluded"
            };
            writeln!(
                output,
                "{contribution}  {}  {}  {}",
                input.test_name(),
                status_text(input.test_status()),
                short_ratio_text(input.summary().coverage())
            )
            .expect("writing to String cannot fail");
            if let Some(token) = input.replay_token() {
                writeln!(output, "  Replay: {token}").expect("writing to String cannot fail");
            }
            writeln!(output, "  Producer: {}", input.producer_version())
                .expect("writing to String cannot fail");
        }
    }

    for group in report.merge().groups() {
        write!(
            output,
            "\n{}\n  Definition: {}, revision {}\n  Coverage: {}\n",
            group.instance_path(),
            group.definition_name(),
            group.definition_revision(),
            ratio_text(group.coverage())
        )
        .expect("writing to String cannot fail");
        if report.options().includes_fingerprints() {
            writeln!(output, "  Fingerprint: {}", group.definition_fingerprint())
                .expect("writing to String cannot fail");
        }
        for item in group.snapshot().items() {
            match *item {
                CoverageItemSnapshot::Coverpoint(ref point) => {
                    render_coverpoint(&mut output, report.options().bin_detail(), point);
                }
                CoverageItemSnapshot::Cross2(ref cross) => {
                    render_cross(&mut output, report.options().bin_detail(), cross);
                }
            }
        }
    }

    output.push('\n');
    output.push_str(&report.gitlab_metric());
    output.push('\n');
    output
}

/// Appends one coverpoint section and its selected bins.
fn render_coverpoint(
    output: &mut String,
    detail: CoverageBinDetail,
    point: &crate::CoverpointSnapshot,
) {
    write!(output, "  {} [coverpoint]\n    Coverage: {}\n    Samples: {}\n    Ignored: {}\n    Illegal: {}\n    Unmatched: {}\n", point.name(), ratio_text(point.coverage()), point.sample_count(), point.ignored_sample_count(), point.illegal_sample_count(), point.unmatched_sample_count()).expect("writing to String cannot fail");
    if matches!(detail, CoverageBinDetail::None) {
        return;
    }

    let normal_label = if matches!(detail, CoverageBinDetail::All) {
        "Bins"
    } else {
        "Uncovered bins"
    };
    let normal = point
        .bins()
        .iter()
        .filter(|bin| include_coverpoint_bin(detail, bin) && bin.kind() != BinKind::Illegal)
        .collect::<Vec<_>>();
    if !normal.is_empty() {
        writeln!(output, "    {normal_label}:").expect("writing to String cannot fail");
        for bin in normal {
            render_coverpoint_bin(output, bin);
        }
    }
    let illegal = point
        .bins()
        .iter()
        .filter(|bin| include_coverpoint_bin(detail, bin) && bin.kind() == BinKind::Illegal)
        .collect::<Vec<_>>();
    if !illegal.is_empty() {
        let label = if matches!(detail, CoverageBinDetail::All) {
            "Illegal bins"
        } else {
            "Hit illegal bins"
        };
        writeln!(output, "    {label}:").expect("writing to String cannot fail");
        for bin in illegal {
            render_coverpoint_bin(output, bin);
        }
    }
}

/// Appends one selected coverpoint bin.
fn render_coverpoint_bin(output: &mut String, bin: &crate::CoverpointBinSnapshot) {
    match bin.kind() {
        BinKind::Normal => writeln!(
            output,
            "      {}: {}, {}/{} hits [{}]",
            bin.name(),
            matcher_text(bin.matcher_kind(), bin.matcher_operand_count()),
            bin.hits(),
            bin.required_hits(),
            if bin.covered() {
                "covered"
            } else {
                "uncovered"
            }
        ),
        BinKind::Ignore => writeln!(output, "      {}: {} hits [ignore]", bin.name(), bin.hits()),
        BinKind::Illegal => writeln!(
            output,
            "      {}: {} hits [illegal]",
            bin.name(),
            bin.hits()
        ),
    }
    .expect("writing to String cannot fail");
}

/// Appends one cross section and its selected generated bins.
fn render_cross(output: &mut String, detail: CoverageBinDetail, cross: &crate::Cross2Snapshot) {
    write!(
        output,
        "  {} [cross]\n    Coverage: {}\n    Samples: {}\n    Skipped: {}\n    Axes: {} x {}\n",
        cross.name(),
        ratio_text(cross.coverage()),
        cross.sample_count(),
        cross.skipped_sample_count(),
        cross.left_coverpoint_name(),
        cross.right_coverpoint_name()
    )
    .expect("writing to String cannot fail");
    let bins = cross
        .bins()
        .iter()
        .filter(|bin| include_cross_bin(detail, bin))
        .collect::<Vec<_>>();
    if !bins.is_empty() {
        let label = if matches!(detail, CoverageBinDetail::All) {
            "Bins"
        } else {
            "Uncovered bins"
        };
        writeln!(output, "    {label}:").expect("writing to String cannot fail");
        for bin in bins {
            writeln!(
                output,
                "      {} x {}: {}/{} hits [{}]",
                bin.left_bin_name(),
                bin.right_bin_name(),
                bin.hits(),
                bin.required_hits(),
                if bin.covered() {
                    "covered"
                } else {
                    "uncovered"
                }
            )
            .expect("writing to String cannot fail");
        }
    }
}

/// Returns the stable lower-case reporting label for one test status.
pub(super) const fn status_text(status: TestStatus) -> &'static str {
    match status {
        TestStatus::Passed => "passed",
        TestStatus::Failed => "failed",
        TestStatus::Error => "error",
    }
}

/// Formats persisted matcher shape without unavailable operand values.
fn matcher_text(kind: crate::BinMatcherKind, operands: usize) -> String {
    match kind {
        crate::BinMatcherKind::Value => "value".to_owned(),
        crate::BinMatcherKind::Values => format!("values({operands})"),
        crate::BinMatcherKind::InclusiveRange => "inclusive_range".to_owned(),
    }
}
