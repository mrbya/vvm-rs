//! Functional coverage definitions, sampling, and runtime state.
//!
//! Persistence, merge, report, session, and snapshot APIs are grouped in the
//! corresponding submodules.

/// Serialized coverage artifact APIs.
pub mod artifact {
    pub use vvm_core::{
        CoverageArtifact, CoverageArtifactGroup, CoverageDefinitionFingerprint,
        CoverageIoOperation, CoveragePersistenceError,
    };
}

/// Coverage artifact merge APIs.
pub mod merge {
    pub use vvm_core::{
        CoverageMerge, CoverageMergeCountKind, CoverageMergeCounterKind, CoverageMergeError,
        CoverageMergeInput, CoverageMergePolicy, CoverageMergeSummary,
    };
}

/// Functional coverage report APIs.
pub mod report {
    pub use vvm_core::{
        CoverageBinDetail, CoveragePercentage, CoverageReport, CoverageReportOptions,
    };
}

/// Coverage capture-session APIs.
pub mod session {
    pub use vvm_core::{
        CoverageSession, CoverageSessionCountKind, CoverageSessionError, CoverageSessionSnapshot,
        CoverageSessionSummary,
    };
}

/// Immutable functional coverage snapshot APIs.
pub mod snapshot {
    pub use vvm_core::{
        CoverageGroupSnapshot, CoverageItemSnapshot, CoverpointBinSnapshot, CoverpointSnapshot,
        Cross2Snapshot, CrossBinSnapshot,
    };
}

pub use vvm_core::{
    Bin, BinId, BinKind, BinMatcherKind, CoverageBuildError, CoverageCounterKind,
    CoverageDefinitionError, CoverageGroup, CoverageGroupCountKind, CoverageGroupError,
    CoverageGroupInstance, CoverageGroupSummary, CoverageGroupVisitor, CoverageInstance,
    CoverageItemKind, CoverageItemRef, CoverageModel, CoverageRatio, CoverageRuntimeError,
    CoverageRuntimeItemKind, CoverageSampleDisposition, CoverageSampleError, Coverpoint,
    CoverpointBin, CoverpointBuilder, CoverpointSample, Cross2, Cross2Builder, CrossAxis, CrossBin,
    CrossBinId, CrossBuildError, CrossCounterKind, CrossSample, CrossSampleDisposition,
    CrossSampleError,
};
