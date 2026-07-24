use super::CoveragePercentage;
use crate::{BinKind, CoverageMerge, CoverageRatio, CoverpointBinSnapshot, CrossBinSnapshot};

/// Amount of per-bin detail included in rendered reports.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageBinDetail {
    /// Do not list individual bins.
    None,
    /// List uncovered coverage bins and hit illegal bins.
    #[default]
    Uncovered,
    /// List every coverpoint and cross bin.
    All,
}

/// Rendering options shared by text and HTML reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoverageReportOptions {
    /// Amount of bin-level detail.
    bin_detail: CoverageBinDetail,
    /// Whether per-test input provenance is shown.
    include_inputs: bool,
    /// Whether complete definition fingerprints are shown.
    include_fingerprints: bool,
}

impl Default for CoverageReportOptions {
    fn default() -> Self {
        Self {
            bin_detail: CoverageBinDetail::Uncovered,
            include_inputs: true,
            include_fingerprints: false,
        }
    }
}

impl CoverageReportOptions {
    /// Sets the amount of bin detail.
    #[must_use]
    pub const fn with_bin_detail(mut self, detail: CoverageBinDetail) -> Self {
        self.bin_detail = detail;
        self
    }
    /// Enables or disables input provenance.
    #[must_use]
    pub const fn with_inputs(mut self, enabled: bool) -> Self {
        self.include_inputs = enabled;
        self
    }
    /// Enables or disables full fingerprints.
    #[must_use]
    pub const fn with_fingerprints(mut self, enabled: bool) -> Self {
        self.include_fingerprints = enabled;
        self
    }
    /// Returns the configured bin detail.
    #[must_use]
    pub const fn bin_detail(self) -> CoverageBinDetail {
        self.bin_detail
    }
    /// Returns whether inputs are included.
    #[must_use]
    pub const fn includes_inputs(self) -> bool {
        self.include_inputs
    }
    /// Returns whether fingerprints are included.
    #[must_use]
    pub const fn includes_fingerprints(self) -> bool {
        self.include_fingerprints
    }
}

/// Read-only renderer over one deterministic coverage merge.
#[derive(Debug, Clone, Copy)]
pub struct CoverageReport<'a> {
    /// Reporting source.
    merge: &'a CoverageMerge,
    /// Rendering configuration.
    options: CoverageReportOptions,
}

impl<'a> CoverageReport<'a> {
    /// Constructs a report with default rendering options.
    #[must_use]
    pub const fn new(merge: &'a CoverageMerge) -> Self {
        Self::with_options(
            merge,
            CoverageReportOptions {
                bin_detail: CoverageBinDetail::Uncovered,
                include_inputs: true,
                include_fingerprints: false,
            },
        )
    }
    /// Constructs a report with explicit options.
    #[must_use]
    pub const fn with_options(merge: &'a CoverageMerge, options: CoverageReportOptions) -> Self {
        Self { merge, options }
    }
    /// Returns the merged reporting source.
    #[must_use]
    pub const fn merge(self) -> &'a CoverageMerge {
        self.merge
    }
    /// Returns the rendering options.
    #[must_use]
    pub const fn options(self) -> CoverageReportOptions {
        self.options
    }
    /// Returns the overall fixed-point percentage.
    #[must_use]
    pub fn percentage(self) -> CoveragePercentage {
        CoveragePercentage::from_ratio(self.merge.coverage())
    }
    /// Renders deterministic plain text.
    #[must_use]
    pub fn to_text(self) -> String {
        super::report_text::render(self)
    }
    /// Renders deterministic self-contained HTML.
    #[must_use]
    pub fn to_html(self) -> String {
        super::report_html::render(self)
    }
    /// Returns one GitLab-compatible metric line.
    #[must_use]
    pub fn gitlab_metric(self) -> String {
        format!("{}{}", Self::GITLAB_METRIC_PREFIX, self.percentage())
    }
}

impl CoverageReport<'_> {
    /// Prefix used by the CI metric line.
    pub const GITLAB_METRIC_PREFIX: &'static str = "VVM functional coverage: ";
    /// Recommended GitLab `coverage` expression.
    pub const GITLAB_COVERAGE_REGEX: &'static str = r"/^VVM functional coverage: \d+\.\d{2}%$/";
    /// Suggested text-report suffix.
    pub const TEXT_FILE_SUFFIX: &'static str = ".vvmcov.txt";
    /// Suggested HTML-report suffix.
    pub const HTML_FILE_SUFFIX: &'static str = ".vvmcov.html";
}

/// Selects one coverpoint bin under the shared detail policy.
pub(super) const fn include_coverpoint_bin(
    detail: CoverageBinDetail,
    bin: &CoverpointBinSnapshot,
) -> bool {
    match detail {
        CoverageBinDetail::None => false,
        CoverageBinDetail::Uncovered => {
            (matches!(bin.kind(), BinKind::Normal) && !bin.covered())
                || (matches!(bin.kind(), BinKind::Illegal) && bin.hits() > 0)
        }
        CoverageBinDetail::All => true,
    }
}

/// Selects one generated cross bin under the shared detail policy.
pub(super) const fn include_cross_bin(detail: CoverageBinDetail, bin: &CrossBinSnapshot) -> bool {
    match detail {
        CoverageBinDetail::None => false,
        CoverageBinDetail::Uncovered => !bin.covered(),
        CoverageBinDetail::All => true,
    }
}

/// Formats a percentage and its authoritative exact ratio.
pub(super) fn ratio_text(ratio: CoverageRatio) -> String {
    let bins = if ratio.total() == 1 { "bin" } else { "bins" };
    format!(
        "{} ({}/{} {bins}; {} uncovered)",
        CoveragePercentage::from_ratio(ratio),
        ratio.covered(),
        ratio.total(),
        ratio.uncovered()
    )
}

/// Formats a compact percentage and exact covered-to-total count.
pub(super) fn short_ratio_text(ratio: CoverageRatio) -> String {
    format!(
        "{} ({}/{})",
        CoveragePercentage::from_ratio(ratio),
        ratio.covered(),
        ratio.total()
    )
}

#[cfg(test)]
mod tests {
    use super::{CoverageBinDetail, CoverageReport, CoverageReportOptions};
    use crate::{
        Bin, CoverageArtifact, CoverageGroup, CoverageGroupInstance, CoverageGroupVisitor,
        CoverageItemRef, CoverageMerge, CoverageMergePolicy, CoverageSession, Coverpoint,
        TestStatus,
    };

    /// Minimal user-owned group used to exercise report output.
    struct Group {
        /// Stable group instance metadata.
        instance: CoverageGroupInstance,
        /// Explicitly owned coverpoint.
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

    /// Creates one merge containing one included and one excluded artifact.
    fn merge() -> Result<CoverageMerge, Box<dyn std::error::Error>> {
        let artifact =
            |name: &str, status| -> Result<CoverageArtifact, Box<dyn std::error::Error>> {
                let mut point = Coverpoint::builder("opcode")
                    .bin(Bin::value("read", 1_u8))
                    .bin(Bin::value("write", 2_u8).at_least(2))
                    .ignore_bin(Bin::value("reset", 0_u8))
                    .illegal_bin(Bin::value("reserved", 3_u8))
                    .build()?;
                point.sample(&1)?;
                if point.sample(&3).is_ok() {
                    return Err("illegal sample unexpectedly succeeded".into());
                }
                let group = Group {
                    instance: CoverageGroupInstance::new("decoder", "dut.decoder")?,
                    point,
                };
                let mut session = CoverageSession::new(name)?;
                session.capture(&group)?;
                let snapshot = session.finish().ok_or("missing snapshot")?;
                Ok(CoverageArtifact::from_session(status, None, snapshot)?)
            };

        CoverageMerge::from_artifacts(
            CoverageMergePolicy::passed_only(),
            [
                artifact("included", TestStatus::Passed)?,
                artifact("excluded", TestStatus::Failed)?,
            ],
        )
        .map_err(Into::into)
    }
    #[test]
    fn defaults_are_documented() {
        let options = CoverageReportOptions::default();
        assert_eq!(options.bin_detail(), CoverageBinDetail::Uncovered);
        assert!(options.includes_inputs());
        assert!(!options.includes_fingerprints());
    }
    #[test]
    fn builders_preserve_other_fields() {
        let options = CoverageReportOptions::default()
            .with_bin_detail(CoverageBinDetail::All)
            .with_inputs(false)
            .with_fingerprints(true);
        assert_eq!(options.bin_detail(), CoverageBinDetail::All);
        assert!(!options.includes_inputs());
        assert!(options.includes_fingerprints());
    }

    #[test]
    fn renderers_and_metric_are_deterministic() -> Result<(), Box<dyn std::error::Error>> {
        let merge = merge()?;
        let report = CoverageReport::new(&merge);
        let text = report.to_text();
        let html = report.to_html();

        assert_eq!(text, report.to_text());
        assert_eq!(html, report.to_html());
        assert!(text.contains("included  included  passed  50.00% (1/2)"));
        assert!(text.contains("excluded  excluded  failed  50.00% (1/2)"));
        assert!(text.contains("write: value, 0/2 hits [uncovered]"));
        assert!(text.contains("reserved: 1 hits [illegal]"));
        assert!(text.ends_with("VVM functional coverage: 50.00%\n"));
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("<progress max=\"10000\" value=\"5000\">"));
        assert!(!html.contains("<script"));
        assert_eq!(report.gitlab_metric(), "VVM functional coverage: 50.00%");
        Ok(())
    }
}
