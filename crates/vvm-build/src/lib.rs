//! Build-time Verilator and bridge generation support for VVM.
//!
//! Add this crate to `[build-dependencies]` and configure [`DutBuilder`] from
//! a consumer `build.rs`. It invokes Verilator, normalizes its metadata,
//! generates the native adapter and Rust wrapper, and compiles the bridge.
//! The consumer includes the result with `vvm::include_dut!`. See the guide at
//! <https://byacrates.gitlab.io/vvm-rs/user-guide.html> for complete setup.

#![allow(clippy::module_name_repetitions)]
// clippy WARN level lints
#![warn(
    missing_docs,
    //clippy::cargo,
    clippy::pedantic,
    clippy::nursery,
    clippy::dbg_macro,
    clippy::unwrap_used,
    clippy::integer_division,
    clippy::large_include_file,
    clippy::map_err_ignore,
    clippy::missing_docs_in_private_items,
    clippy::panic,
    clippy::todo,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unreachable
)]
// clippy WARN level lints, that can be upgraded to DENY if preferred
#![warn(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::modulo_arithmetic,
    clippy::as_conversions,
    clippy::assertions_on_result_states,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::pattern_type_mismatch,
    clippy::string_slice,
    clippy::try_err
)]
// clippy DENY level lints, they always have a quick fix that should be preferred
#![deny(
    clippy::wildcard_imports,
    clippy::multiple_inherent_impl,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::string_add,
    clippy::implicit_clone,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::verbose_file_reads
)]

/// Verilator DUT build configuration.
pub(crate) mod builder;
/// Rerun-if directives.
pub(crate) mod cargo;
/// Adapter and CXX Bridge code generation orchestration.
pub(crate) mod codegen;
/// Checked external command execution.
pub(crate) mod command;
/// Build error definitions.
pub(crate) mod error;
/// Verilator metadata ingestion and normalization.
pub(crate) mod metadata;
/// cxx-build/native source compilation.
pub(crate) mod native;
/// Path resolution, validation and deduplication.
pub(crate) mod paths;
/// Trace dump api
pub(crate) mod trace;
/// Verilator discovery, metadata and model generation.
pub(crate) mod verilator;

// Re-exports
pub use builder::DutBuilder;
pub use error::{BuildError, BuildResult, BuildStage};
pub use trace::TraceOptions;

// Tests
#[cfg(test)]
mod tests;
