# Rustdoc Style

This document captures the prevailing inline Rust documentation style used by the hand-written Rust sources under `crates/`.

Scope:
- Applies to non-generated Rust sources only.
- Excludes tool-generated code.
- Describes the dominant style, not every minor local variation.

## Core Style

- Document broadly and consistently. The crates enable `missing_docs` and `clippy::missing_docs_in_private_items`, so the style assumes that most public and private items are documented.
- Explain intent, boundaries, and behavior, not just names. The docs consistently answer why a module or function exists and how callers should think about it.
- Prefer precise, maintainers-oriented language over marketing language.
- Keep wording stable and concrete. Terms such as `stable`, `deterministic`, `transport-neutral`, `canonical`, `deliberately`, and `intentionally` should appear often and reflect the tone of the codebase.

## Module-Level Docs

- Use `//!` for crate roots and for modules that define a meaningful subsystem or architectural boundary.
- Start with a one-line summary sentence.
- Follow with one or more short paragraphs that explain the module's role in the pipeline, what it owns, and what it deliberately does not own.
- Use numbered lists for flows, phases, or pipelines.
- Use bullet lists for design constraints, responsibilities, or key choices.
- Link to related modules with intra-doc links when they help explain layering.

Typical module doc shape:

```rust
//! One-line summary.
//!
//! Short explanation of the module's role.
//!
//! The module is organized around this flow:
//!
//! 1. first step,
//! 2. second step,
//! 3. third step.
//!
//! Key design choices:
//!
//! - choice one.
//! - choice two.
```

Representative example:
```rust
//! Syntax-layer parsing, typed CST traversal, and lightweight syntax indexing.
//!
//! This crate is the structural foundation for the rest of the workspace.
//! It intentionally stays close to the concrete syntax tree (CST) produced by
//! tree-sitter and avoids embedding DKB business rules that belong to
//! `dkb-analysis`.
//!
//! The hand-written modules in this crate are organized around the lifecycle of
//! one parsed document snapshot:
//!
//! - [`parse`] turns source text into a tree and collects recoverable syntax
//!   problems.
//! - [`walk`] provides lightweight typed traversal helpers over the generated
//!   `type-sitter` wrappers.
//! - [`index`] condenses the parsed structure into a query-friendly
//!   document-local index that downstream crates can reuse without repeatedly
//!   walking the `CST`.
//!
//! In other words, this crate answers structural questions such as "what node is
//! here?", "what spans belong to this `_main_` line?", or "where are packet-type
//! tokens in this document?". It intentionally does **not** answer semantic
//! questions such as whether a role token is valid in context, whether relation
//! names are duplicated, or how a line should be presented to LSP clients. Those
//! concerns belong to `dkb-analysis` and `dkb-lsp`.
//!
//! At a high level, this crate provides three capabilities:
//!
//! - **Parsing** via [`parse`]: convert source text into a tree-sitter tree and
//!   collect recoverable syntax problems.
//! - **Typed traversal** via [`walk`]: expose ergonomic helpers over generated
//!   `type-sitter` node wrappers.
//! - **Structural indexing** via [`index`]: build a document-local `SyntaxIndex`
//!   that captures spans and token classes used by analysis and LSP features.
//!
//! Design constraints:
//!
//! - Keep parser behavior tolerant and lossless.
//! - Represent structure faithfully, even for partially invalid inputs.
//! - Reserve semantic interpretation and validation for downstream crates.

## Item-Level Docs

- Use `///` for enums, structs, functions, methods, constants, modules, fields, and enum variants.
- Begin with a short summary sentence.
- Add a follow-up paragraph when the item has non-obvious behavior, a design constraint, or an important usage expectation.
- Prefer describing observable behavior and invariants over restating the signature.
- Single-line `///` comments are reserved for short non-function items such as struct fields or enum variants.

Typical function or method shape:

```rust
/// One-line summary.
///
/// Optional rationale or behavior notes.
///
/// # Arguments
///
/// - `arg_name`: What this argument means.
///
/// # Returns
///
/// Returns the value or effect in caller-facing terms.
///
/// # Errors
/// Returns `SomeError` when the operation cannot succeed.
```

Representative example:
```rust
/// Issues a GET request to `{GRAPH_BASE}{path}` with a valid bearer token.
///
/// This is the primary entry point for Graph API requests that use relative paths.
/// The `path` argument is appended directly to `GRAPH_BASE`, so it must begin
/// with a `/` (e.g. `"/me/planner/plans"`).
///
/// Internally delegates to [`GraphClient::get_url`] with the fully-formed URL, which
/// handles `401` retry and `429` backoff.
///
/// # Arguments
///
/// - `path`: A Graph API path starting with `/` (e.g. `"/me/planner/plans"`).
///
/// # Returns
///
/// `Ok(T)` — the deserialised response body on success.
///
/// # Errors
///
/// Returns an error if:
/// - a valid bearer token cannot be obtained from [`AuthManager`],
/// - the HTTP request fails (network error, DNS, etc.),
/// - the Graph API returns a non-2xx status (see `parse_response`), or
/// - the response body cannot be deserialised as `T`.
pub async fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
    // ...
}
```

## Structs, Enums, Fields, and Variants

- Give structs and enums a short summary plus a brief explanation when the type has a specific role in a pipeline or transport boundary.
- Document each field with a short noun-phrase or sentence.
- Document each enum variant individually.
- For enums, variant docs often describe both the meaning and the condition that triggers that variant when helpful.

Representative examples:
```rust
/// Semantic token class used for editor highlighting layers.
///
/// These kinds are crate-local classifications; a future protocol handler can map
/// them onto whichever LSP semantic token legend the server chooses to expose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticTokenKind {
    /// Section-header text fragments.
    SectionHeaderText,
    /// `_main_` keyword token.
    MainKeyword,
    /// `_main_` role token.
    MainRole,
    /// Relation-name token.
    RelationName,
    /// Relation packet-type token.
    PacketType,
    /// Numeric literal token.
    Number,
    /// String literal token.
    String,
}
```

```rust
/// Top level CLI struct for `kaze` command configuration.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "kaze",
    about = "Zephyr build system companion.",
    version,
    propagate_version = true,
)]
pub struct Cli {
    /// Pre-clean the active build dir
    #[arg(short = 'c', long = "clean")]
    pub preclean: bool,

    /// Run command for all configured profiles (overrides --profile/default)
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Select project profile defined in kaze.toml
    #[arg(short = 'p', long = "profile")]
    pub profile: Option<String>,

    /// Zephyr OS board/target (overrides config)
    #[arg(short = 'b', long = "board")]
    pub board: Option<String>,

    // ...
}
```

```rust
/// Stable semantic diagnostic code identifiers.
///
/// These codes should remain stable over time even if message text evolves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticDiagnosticCode {
    /// Empty section header.
    ///
    /// Section header text is either missing or comprised of whitespace
    /// characters only.
    EmptySectionHeader,
    /// Missing test line.
    ///
    /// Non-empty section is missing its `_main_` test line.
    MissingTestLine,
    /// `_main_` role token is not recognized.
    ///
    /// Triggered when role text does not begin with `M` or `S`.
    UnknownMainRole,
    /// A number literal could not be parsed.
    ///
    /// Triggered when decimal/hex normalization fails.
    InvalidNumberLiteral,

    // ...
}
```

## Standard Sections

- `# Arguments` is commonly used for functions and methods with parameters.
- `# Returns` is commonly used even for straightforward constructors and helpers.
- `# Errors` is used for fallible APIs and omitted for infallible ones.
- `# Examples` is used for core public API and non-trivial API.
- `# Panics` when panics are possible or intentionally used.
- `# Safety` are also absent from the current hand-written sources.

Section formatting rules:
- Put a blank doc-comment line before each heading.
- Use Markdown headings exactly as section labels.
- Under `# Arguments`, use flat bullet lists with backticked parameter names.
- Under `# Returns`, describe the returned value from the caller's perspective.
- Under `# Errors`, describe the failure condition, not just the error type name.

## Markdown and Formatting Conventions

- Use blank `///` or `//!` lines to separate paragraphs and sections.
- Wrap code identifiers, literal tokens, module names, and grammar terms in backticks.
- Use intra-doc links for related types and modules when they improve navigation.
- Prefer complete sentences.
- Keep summaries concise, but allow explanatory paragraphs where behavior would otherwise be ambiguous.

## Tone

- Write in a factual, engineering-oriented voice.
- Prefer wording that makes constraints explicit.
- Explain tradeoffs directly when they matter.
- Avoid promotional phrasing, vague adjectives, and conversational filler.

Examples of characteristic phrasing:
- "This crate is the structural foundation..."
- "The crate intentionally tolerates incomplete data."
- "This is the canonical constructor..."
- "Tokens are sorted deterministically..."

## When File-Level Docs Are Optional

- Crate roots and major subsystem modules usually have `//!` docs.
- Small leaf modules may omit file-level docs when their purpose is already clear from the surrounding module structure.
- Even when a leaf file omits `//!` docs, its major items are still typically documented.
- Test support files may have little or no rustdoc.

## Recommended House Template

Use this template when writing new docs in the same style:

```rust
//! Subsystem summary.
//!
//! Short explanation of responsibility and boundaries.
//!
//! Optional flow or design notes.

/// Item summary.
///
/// Optional behavior, rationale, or invariant notes.
///
/// # Arguments
///
/// - `input`: Meaning of the input.
///
/// # Returns
///
/// Returns the caller-visible result.
///
/// # Errors
/// Returns `SomeError` when a documented failure condition occurs.
fn example(input: &str) -> Result<String, SomeError> {
    unimplemented!()
}
```

## Summary

The dominant style under `crates/` is structured, high-coverage rustdoc with these traits:
- `//!` for architectural and module context.
- `///` for item contracts and field/variant descriptions.
- A summary-first layout followed by rationale.
- Regular use of `# Arguments`, `# Returns`, and `# Errors`.
- Strong preference for explaining intent and constraints.
- Almost no example-driven documentation.
