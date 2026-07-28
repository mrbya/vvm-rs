//! Functional coverage definitions, sampling, and runtime state.
//!
//! Persistence, merge, report, session, and snapshot APIs are grouped in the
//! corresponding submodules.

/// Serialized coverage artifact APIs.
pub mod artifact {
    pub use vvm_core::{
        CoverageArtifact, CoverageArtifactGroup, CoverageDefinitionFingerprint,
        CoverageIoOperation as IoOperation, CoveragePersistenceError as PersistenceError,
    };
}

/// Coverage artifact merge APIs.
pub mod merge {
    pub use vvm_core::{
        CoverageMerge, CoverageMergeCountKind as MergeCountKind,
        CoverageMergeCounterKind as MergeCounterKind, CoverageMergeError as MergeError,
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
        CoverageSession, CoverageSessionCountKind as SessionCountKind,
        CoverageSessionError as SessionError, CoverageSessionSnapshot, CoverageSessionSummary,
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
    Bin, BinId, BinKind, BinMatcherKind, CoverageBuildError as BuildError, CoverageCounterKind,
    CoverageDefinitionError as DefinitionError, CoverageGroup, CoverageGroupCountKind,
    CoverageGroupError as GroupError, CoverageGroupInstance, CoverageGroupSummary,
    CoverageGroupVisitor, CoverageInstance, CoverageItemKind, CoverageItemRef, CoverageModel,
    CoverageRatio, CoverageRuntimeError as RuntimeError, CoverageRuntimeItemKind,
    CoverageSampleDisposition, CoverageSampleError as SampleError, Coverpoint, CoverpointBin,
    CoverpointBuilder, CoverpointSample, Cross2, Cross2Builder, CrossAxis, CrossBin, CrossBinId,
    CrossBuildError, CrossCounterKind, CrossSample, CrossSampleDisposition, CrossSampleError,
};
