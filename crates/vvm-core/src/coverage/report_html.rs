use std::fmt::Write as _;

use super::report::{include_coverpoint_bin, include_cross_bin, ratio_text, short_ratio_text};
use super::report_text::status_text;
use super::{CoverageBinDetail, CoveragePercentage, CoverageReport};
use crate::{BinKind, CoverageItemSnapshot};

/// Renders the complete deterministic self-contained HTML document.
pub(super) fn render(report: CoverageReport<'_>) -> String {
    let mut output = String::from(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta \
         name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>VVM \
         Functional Coverage</title>\n  <style>\n:root { color-scheme: light dark; --bg: #fff; \
         --fg: #18212b; --muted: #667; --border: #bcc; --bad: #9b1c1c; --good: #176b3a; }\n@media \
         (prefers-color-scheme: dark) { :root { --bg: #18212b; --fg: #f5f7fa; --muted: #b5bec8; \
         --border: #52606d; --bad: #ff9999; --good: #89db9f; } }\nbody { max-width: 1100px; \
         margin: 2rem auto; padding: 0 1rem; background: var(--bg); color: var(--fg); font: 16px \
         system-ui, sans-serif; } table { width: 100%; border-collapse: collapse; margin: 1rem 0; \
         } th, td { border: 1px solid var(--border); padding: .45rem; text-align: left; } .grid { \
         display: grid; grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr)); gap: .75rem; \
         } .card { border: 1px solid var(--border); padding: .75rem; } .excluded { opacity: .72; \
         } .covered { color: var(--good); } .uncovered, .illegal { color: var(--bad); } progress \
         { width: 12rem; }\n  </style>\n</head>\n<body>\n<header><h1>VVM Functional \
         Coverage</h1></header>\n<main>\n",
    );
    render_summary(&mut output, report);
    if report.options().includes_inputs() {
        render_inputs(&mut output, report);
    }
    for (group_index, group) in report.merge().groups().iter().enumerate() {
        render_group(&mut output, report, group_index, group);
    }
    output.push_str("</main>\n<footer><p>");
    push_html_escaped(&mut output, &report.gitlab_metric());
    output.push_str("</p></footer>\n</body>\n</html>\n");
    output
}

/// Appends the semantic overall summary section.
fn render_summary(output: &mut String, report: CoverageReport<'_>) {
    let summary = report.merge().summary();
    let percentage = report.percentage();
    writeln!(
        output,
        "<section aria-labelledby=\"overall\"><h2 id=\"overall\">Overall</h2><div \
         class=\"grid\"><div class=\"card\"><strong>{percentage}</strong><br><progress \
         max=\"10000\" value=\"{}\">{percentage}</progress><br>{}</div><div \
         class=\"card\">Covered bins: {}</div><div class=\"card\">Uncovered bins: {}</div><div \
         class=\"card\">Included artifacts: {}</div><div class=\"card\">Excluded artifacts: \
         {}</div><div class=\"card\">Group instances: {}</div></div><p>Merge policy: \
         {}</p><p>Statuses: {} passed, {} failed, {} errored. Structure: {} coverpoints, {} \
         crosses.</p></section>",
        percentage.basis_points(),
        ratio_text(summary.coverage()),
        summary.coverage().covered(),
        summary.coverage().uncovered(),
        summary.included_artifact_count(),
        summary.excluded_artifact_count(),
        summary.group_count(),
        report.merge().policy(),
        summary.passed_artifact_count(),
        summary.failed_artifact_count(),
        summary.errored_artifact_count(),
        summary.coverpoint_count(),
        summary.cross_count()
    )
    .expect("writing to String cannot fail");
}

/// Appends input provenance when it is enabled.
fn render_inputs(output: &mut String, report: CoverageReport<'_>) {
    output.push_str(
        "<section aria-labelledby=\"inputs\"><h2 \
         id=\"inputs\">Inputs</h2><table><thead><tr><th>Contribution</th><th>Test</th><th>Status</\
         th><th>Producer</th><th>Replay \
         token</th><th>Coverage</th><th>Bins</th></tr></thead><tbody>",
    );
    for input in report.merge().inputs() {
        let class = if input.included() {
            "included"
        } else {
            "excluded"
        };
        write!(
            output,
            "<tr class=\"{class}\"><td>{}</td><td>",
            if input.included() {
                "included"
            } else {
                "excluded"
            }
        )
        .expect("writing to String cannot fail");
        push_html_escaped(output, input.test_name());
        output.push_str("</td><td>");
        push_html_escaped(output, status_text(input.test_status()));
        output.push_str("</td><td>");
        push_html_escaped(output, input.producer_version());
        output.push_str("</td><td>");
        if let Some(token) = input.replay_token() {
            push_html_escaped(output, &token.to_string());
        }
        output.push_str("</td><td>");
        push_html_escaped(
            output,
            &CoveragePercentage::from_ratio(input.summary().coverage()).to_string(),
        );
        output.push_str("</td><td>");
        push_html_escaped(output, &short_ratio_text(input.summary().coverage()));
        output.push_str("</td></tr>");
    }
    output.push_str("</tbody></table></section>\n");
}

/// Appends one group and its definition-ordered item sections.
fn render_group(
    output: &mut String,
    report: CoverageReport<'_>,
    group_index: usize,
    group: &crate::CoverageArtifactGroup,
) {
    write!(output, "<section id=\"group-{group_index}\"><h2>")
        .expect("writing to String cannot fail");
    push_html_escaped(output, group.instance_path());
    output.push_str("</h2><p>Definition: ");
    push_html_escaped(output, group.definition_name());
    write!(
        output,
        ", revision {}. Coverage: {}. Items: {}; coverpoints: {}; crosses: {}.</p>",
        group.definition_revision(),
        ratio_text(group.coverage()),
        group.summary().item_count(),
        group.summary().coverpoint_count(),
        group.summary().cross_count()
    )
    .expect("writing to String cannot fail");
    if report.options().includes_fingerprints() {
        output.push_str("<p>Fingerprint: ");
        push_html_escaped(output, &group.definition_fingerprint().to_string());
        output.push_str("</p>");
    }
    for (item_index, item) in group.snapshot().items().iter().enumerate() {
        render_item(
            output,
            report.options().bin_detail(),
            group_index,
            item_index,
            item,
        );
    }
    output.push_str("</section>\n");
}

/// Appends one coverpoint or cross item section.
fn render_item(
    output: &mut String,
    detail: CoverageBinDetail,
    group_index: usize,
    item_index: usize,
    item: &CoverageItemSnapshot,
) {
    write!(
        output,
        "<section id=\"item-{group_index}-{item_index}\"><h3>"
    )
    .expect("writing to String cannot fail");
    push_html_escaped(output, item.name());
    output.push_str("</h3>");
    match *item {
        CoverageItemSnapshot::Coverpoint(ref point) => {
            write!(
                output,
                "<p>Coverpoint. Coverage: {}. Samples: {}. Ignored: {}. Illegal: {}. Unmatched: \
                 {}.</p>",
                ratio_text(point.coverage()),
                point.sample_count(),
                point.ignored_sample_count(),
                point.illegal_sample_count(),
                point.unmatched_sample_count()
            )
            .expect("writing to String cannot fail");
            render_point_bins(output, detail, point);
        }
        CoverageItemSnapshot::Cross2(ref cross) => {
            output.push_str("<p>Cross. Coverage: ");
            push_html_escaped(output, &ratio_text(cross.coverage()));
            write!(
                output,
                ". Samples: {}. Skipped: {}. Axes: ",
                cross.sample_count(),
                cross.skipped_sample_count()
            )
            .expect("writing to String cannot fail");
            push_html_escaped(output, cross.left_coverpoint_name());
            output.push_str(" x ");
            push_html_escaped(output, cross.right_coverpoint_name());
            output.push_str(".</p>");
            render_cross_bins(output, detail, cross);
        }
    }
    output.push_str("</section>");
}

/// Appends selected coverpoint bins in declaration order.
fn render_point_bins(
    output: &mut String,
    detail: CoverageBinDetail,
    point: &crate::CoverpointSnapshot,
) {
    if matches!(detail, CoverageBinDetail::None) {
        return;
    }
    let bins = point
        .bins()
        .iter()
        .filter(|bin| include_coverpoint_bin(detail, bin))
        .collect::<Vec<_>>();
    if bins.is_empty() {
        return;
    }
    output.push_str(
        "<details open><summary>Bins</summary><table><thead><tr><th>Bin</th><th>Role</\
         th><th>Matcher</th><th>Hits</th><th>Required</th><th>Status</th></tr></thead><tbody>",
    );
    for bin in bins {
        output.push_str("<tr><td>");
        push_html_escaped(output, bin.name());
        output.push_str("</td><td>");
        push_html_escaped(output, &bin.kind().to_string());
        output.push_str("</td><td>");
        push_html_escaped(
            output,
            &matcher_text(bin.matcher_kind(), bin.matcher_operand_count()),
        );
        write!(
            output,
            "</td><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>",
            bin.hits(),
            bin.required_hits(),
            point_status(bin),
            point_status(bin)
        )
        .expect("writing to String cannot fail");
    }
    output.push_str("</tbody></table></details>");
}

/// Appends selected cross bins in row-major order.
fn render_cross_bins(
    output: &mut String,
    detail: CoverageBinDetail,
    cross: &crate::Cross2Snapshot,
) {
    if matches!(detail, CoverageBinDetail::None) {
        return;
    }
    let bins = cross
        .bins()
        .iter()
        .filter(|bin| include_cross_bin(detail, bin))
        .collect::<Vec<_>>();
    if bins.is_empty() {
        return;
    }
    output.push_str(
        "<details open><summary>Cross bins</summary><table><thead><tr><th>Left bin</th><th>Right \
         bin</th><th>Hits</th><th>Required</th><th>Status</th></tr></thead><tbody>",
    );
    for bin in bins {
        output.push_str("<tr><td>");
        push_html_escaped(output, bin.left_bin_name());
        output.push_str("</td><td>");
        push_html_escaped(output, bin.right_bin_name());
        write!(
            output,
            "</td><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>",
            bin.hits(),
            bin.required_hits(),
            if bin.covered() {
                "covered"
            } else {
                "uncovered"
            },
            if bin.covered() {
                "covered"
            } else {
                "uncovered"
            }
        )
        .expect("writing to String cannot fail");
    }
    output.push_str("</tbody></table></details>");
}

/// Returns the textual state for one coverpoint bin.
const fn point_status(bin: &crate::CoverpointBinSnapshot) -> &'static str {
    match bin.kind() {
        BinKind::Normal if bin.covered() => "covered",
        BinKind::Normal => "uncovered",
        BinKind::Ignore => "ignore",
        BinKind::Illegal => "illegal",
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

/// Appends dynamic text with HTML metacharacters escaped.
pub(super) fn push_html_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::push_html_escaped;
    #[test]
    fn escapes_markup() {
        let mut output = String::new();
        push_html_escaped(&mut output, "<script>alert(\"x\")</script>&'");
        assert_eq!(
            output,
            "&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt;&amp;&#39;"
        );
    }
}
