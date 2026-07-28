/// Default waveform trace depth.
const DEFAULT_TRACE_DEPTH: u32 = 99;

/// Native waveform format generated for a DUT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceFormat {
    /// Value Change Dump.
    Vcd,

    /// Fast Signal Trace.
    Fst,
}

/// Build-time waveform tracing configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceOptions {
    /// Trace waveform format.
    pub(crate) format: TraceFormat,

    /// Trace waveform depth.
    pub(crate) depth: u32,
}

impl TraceOptions {
    /// Enables VCD tracing with a sensible default depth.
    #[must_use]
    pub const fn vcd() -> Self {
        Self {
            format: TraceFormat::Vcd,
            depth: DEFAULT_TRACE_DEPTH,
        }
    }

    /// Enables FST tracing with a sensible default depth.
    #[must_use]
    pub const fn fst() -> Self {
        Self {
            format: TraceFormat::Fst,
            depth: DEFAULT_TRACE_DEPTH,
        }
    }

    /// Configures waveform trace depth.
    #[must_use]
    pub const fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_TRACE_DEPTH, TraceFormat, TraceOptions};

    #[test]
    fn vcd_uses_default_depth() {
        assert_eq!(
            TraceOptions::vcd(),
            TraceOptions {
                format: TraceFormat::Vcd,
                depth: DEFAULT_TRACE_DEPTH,
            }
        );
    }

    #[test]
    fn fst_uses_default_depth() {
        assert_eq!(
            TraceOptions::fst(),
            TraceOptions {
                format: TraceFormat::Fst,
                depth: DEFAULT_TRACE_DEPTH,
            }
        );
    }

    #[test]
    fn trace_depth_overrides_default_for_each_format() {
        assert_eq!(TraceOptions::vcd().with_depth(0).depth, 0);
        assert_eq!(TraceOptions::fst().with_depth(12).depth, 12);
    }
}
