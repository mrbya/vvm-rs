# Coverage

Coverage APIs are grouped under `vvm::coverage`.

The surface includes:

- primitive bin and coverpoint construction;
- typed coverage models through `#[derive(Coverage)]`;
- sessions, snapshots, artifacts, merge, and reports through submodules.

The coverage chapters explain how to design a useful model. This API guide page
is here to show where the types live and how they relate.

Rustdoc:

- [`Coverage` derive](../api/vvm/derive.Coverage.html)
- [`vvm::coverage`](../api/vvm/coverage/index.html)
- [`Bin`](../api/vvm/coverage/struct.Bin.html)
- [`Coverpoint`](../api/vvm/coverage/struct.Coverpoint.html)
- [`Cross2`](../api/vvm/coverage/struct.Cross2.html)
- [`CoverageModel`](../api/vvm/coverage/trait.CoverageModel.html)
- [`CoverageSession`](../api/vvm/coverage/session/struct.CoverageSession.html)
- [`CoverageArtifact`](../api/vvm/coverage/artifact/struct.CoverageArtifact.html)
- [`CoverageMerge`](../api/vvm/coverage/merge/struct.CoverageMerge.html)
- [`CoverageReport`](../api/vvm/coverage/report/struct.CoverageReport.html)
