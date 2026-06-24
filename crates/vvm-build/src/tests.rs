use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use crate::builder::{
    Define, DutBuilder, minimum_supported_version, validate_defines, validate_identifier,
};
use crate::error::BuildError;
use crate::paths::{ensure_unique_paths, resolve_directory, resolve_file};
use crate::verilator::{
    MetadataCommand, ModelCommand, VerilatorVersion, define_argument, ensure_metadata_output,
    ensure_supported_version, generated_sources, include_argument, metadata_command, model_command,
    parse_version, select_executable,
};

fn touch(path: &Path) -> std::io::Result<()> {
    fs::write(path, b"")
}

#[test]
fn accepts_valid_ascii_identifiers() -> Result<(), BuildError> {
    for identifier in ["counter", "_counter", "counter_2"] {
        validate_identifier("identifier", identifier)?;
    }

    Ok(())
}

#[test]
fn rejects_empty_identifier() {
    assert!(matches!(
        validate_identifier("identifier", ""),
        Err(BuildError::InvalidIdentifier { .. })
    ));
}

#[test]
fn rejects_identifier_with_numeric_first_character() {
    assert!(matches!(
        validate_identifier("identifier", "2counter"),
        Err(BuildError::InvalidIdentifier { .. })
    ));
}

#[test]
fn rejects_identifier_with_punctuation() {
    for identifier in ["counter-name", "counter.name"] {
        assert!(matches!(
            validate_identifier("identifier", identifier),
            Err(BuildError::InvalidIdentifier { .. })
        ));
    }
}

#[test]
fn rejects_non_ascii_identifier() {
    assert!(matches!(
        validate_identifier("identifier", "čounter"),
        Err(BuildError::InvalidIdentifier { .. })
    ));
}

#[test]
fn resolves_relative_file_against_manifest_directory() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let manifest_dir = directory.path();
    let rtl_dir = manifest_dir.join("rtl");
    fs::create_dir_all(&rtl_dir)?;
    let source_path = rtl_dir.join("counter.sv");
    touch(&source_path)?;

    let resolved = resolve_file(manifest_dir, Path::new("rtl/counter.sv"), "HDL source file")?;

    assert_eq!(resolved, source_path.canonicalize()?);
    Ok(())
}

#[test]
fn preserves_absolute_file_paths() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let source_path = directory.path().join("counter.sv");
    touch(&source_path)?;

    let resolved = resolve_file(directory.path(), &source_path, "HDL source file")?;

    assert_eq!(resolved, source_path.canonicalize()?);
    Ok(())
}

#[test]
fn accepts_existing_file() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let source_path = directory.path().join("counter.sv");
    touch(&source_path)?;

    let _resolved = resolve_file(directory.path(), Path::new("counter.sv"), "HDL source file")?;

    Ok(())
}

#[test]
fn rejects_missing_file() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let result = resolve_file(directory.path(), Path::new("missing.sv"), "HDL source file");

    assert!(matches!(
        result,
        Err(BuildError::MissingConfiguredPath {
            role: "HDL source file",
            ..
        })
    ));
    Ok(())
}

#[test]
fn rejects_directory_when_file_is_expected() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let nested_dir = directory.path().join("rtl");
    fs::create_dir_all(&nested_dir)?;

    let result = resolve_file(directory.path(), Path::new("rtl"), "HDL source file");

    assert!(matches!(
        result,
        Err(BuildError::ConfiguredPathNotFile {
            role: "HDL source file",
            ..
        })
    ));
    Ok(())
}

#[test]
fn accepts_existing_directory() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let include_dir = directory.path().join("cpp");
    fs::create_dir_all(&include_dir)?;

    let resolved = resolve_directory(directory.path(), Path::new("cpp"), "C++ include directory")?;

    assert_eq!(resolved, include_dir.canonicalize()?);
    Ok(())
}

#[test]
fn rejects_file_when_directory_is_expected() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let file_path = directory.path().join("counter.cpp");
    touch(&file_path)?;

    let result = resolve_directory(
        directory.path(),
        Path::new("counter.cpp"),
        "C++ include directory",
    );

    assert!(matches!(
        result,
        Err(BuildError::ConfiguredPathNotDirectory {
            role: "C++ include directory",
            ..
        })
    ));
    Ok(())
}

#[test]
fn detects_duplicate_canonical_paths() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let source_path = directory.path().join("counter.sv");
    touch(&source_path)?;

    let first = resolve_file(directory.path(), Path::new("counter.sv"), "HDL source file")?;
    let second = resolve_file(
        directory.path(),
        Path::new("./counter.sv"),
        "HDL source file",
    )?;

    let result = ensure_unique_paths(&[first, second], "HDL source file");

    assert!(matches!(
        result,
        Err(BuildError::DuplicateConfiguredPath {
            role: "HDL source file",
            ..
        })
    ));
    Ok(())
}

#[test]
fn explicit_executable_override_beats_environment_value() {
    let selected = select_executable(
        Some(OsString::from("/opt/verilator/bin/verilator")),
        Some(OsString::from("verilator-from-env")),
    );

    assert_eq!(selected, OsString::from("/opt/verilator/bin/verilator"));
}

#[test]
fn environment_executable_beats_default() {
    let selected = select_executable(None, Some(OsString::from("verilator-5.040")));

    assert_eq!(selected, OsString::from("verilator-5.040"));
}

#[test]
fn default_executable_is_verilator() {
    let selected = select_executable(None, None);

    assert_eq!(selected, OsString::from("verilator"));
}

#[test]
fn hdl_include_argument_is_single_os_string() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let include_dir = directory.path().join("rtl include");
    fs::create_dir_all(&include_dir)?;

    let argument = include_argument(&include_dir);
    let expected = {
        let mut value = OsString::from("-I");
        value.push(include_dir.as_os_str());
        value
    };

    assert_eq!(argument, expected);
    Ok(())
}

#[test]
fn hdl_include_directories_preserve_order() {
    let builder = DutBuilder::new("counter")
        .hdl_include("rtl/include-a")
        .hdl_includes(["rtl/include-b", "rtl/include-c"]);

    let actual: Vec<&Path> = builder
        .hdl_include_dirs_slice()
        .iter()
        .map(PathBuf::as_path)
        .collect();
    let expected = vec![
        Path::new("rtl/include-a"),
        Path::new("rtl/include-b"),
        Path::new("rtl/include-c"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn valid_define_without_value_is_preserved() {
    let builder = DutBuilder::new("counter").define("VVM_EXAMPLE");

    assert_eq!(builder.defines_slice(), &[Define::new("VVM_EXAMPLE", None)]);
}

#[test]
fn valid_define_with_value_is_preserved() {
    let builder = DutBuilder::new("counter").define_value("COUNTER_WIDTH", "8");

    assert_eq!(
        builder.defines_slice(),
        &[Define::new("COUNTER_WIDTH", Some("8"))]
    );
}

#[test]
fn empty_define_value_is_preserved() {
    let define = Define::new("NAME", Some(""));

    assert_eq!(define_argument(&define), OsString::from("-DNAME="));
}

#[test]
fn invalid_define_name_is_rejected_by_identifier_validation() {
    assert!(matches!(
        validate_identifier("HDL definition", "counter-name"),
        Err(BuildError::InvalidIdentifier { .. })
    ));
}

#[test]
fn non_ascii_define_name_is_rejected_by_identifier_validation() {
    assert!(matches!(
        validate_identifier("HDL definition", "čounter"),
        Err(BuildError::InvalidIdentifier { .. })
    ));
}

#[test]
fn duplicate_define_name_without_values_is_rejected() {
    let defines = [Define::new("NAME", None), Define::new("NAME", None)];

    assert!(matches!(
        validate_defines(&defines),
        Err(BuildError::DuplicateDefine { name }) if name == "NAME"
    ));
}

#[test]
fn duplicate_define_name_with_identical_values_is_rejected() {
    let defines = [
        Define::new("NAME", Some("1")),
        Define::new("NAME", Some("1")),
    ];

    assert!(matches!(
        validate_defines(&defines),
        Err(BuildError::DuplicateDefine { name }) if name == "NAME"
    ));
}

#[test]
fn duplicate_define_name_with_different_values_is_rejected() {
    let defines = [
        Define::new("NAME", Some("1")),
        Define::new("NAME", Some("2")),
    ];

    assert!(matches!(
        validate_defines(&defines),
        Err(BuildError::DuplicateDefine { name }) if name == "NAME"
    ));
}

#[test]
fn define_arguments_preserve_order() {
    let defines = [
        Define::new("FIRST", None),
        Define::new("SECOND", Some("2")),
        Define::new("THIRD", Some("")),
    ];

    let actual: Vec<OsString> = defines.iter().map(define_argument).collect();
    let expected = vec![
        OsString::from("-DFIRST"),
        OsString::from("-DSECOND=2"),
        OsString::from("-DTHIRD="),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn parses_supported_verilator_versions() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("Verilator 5.000", VerilatorVersion::new(5, 0)),
        ("Verilator 5.040 2025-01-01", VerilatorVersion::new(5, 40)),
        ("Verilator 5.041 devel", VerilatorVersion::new(5, 41)),
        ("Verilator 6.000", VerilatorVersion::new(6, 0)),
    ];

    for (output, expected) in cases {
        assert_eq!(parse_version(output)?, expected);
    }

    Ok(())
}

#[test]
fn rejects_malformed_verilator_versions() {
    for output in [
        "Verilator",
        "unknown tool 5.040",
        "Verilator version-five",
        "Verilator 5",
        "Verilator 5.x",
    ] {
        assert!(matches!(
            parse_version(output),
            Err(BuildError::InvalidVerilatorVersion { .. })
        ));
    }
}

#[test]
fn verilator_version_ordering_matches_expectations() {
    assert!(VerilatorVersion::new(5, 40) > VerilatorVersion::new(5, 39));
    assert!(VerilatorVersion::new(5, 40) > VerilatorVersion::new(5, 0));
    assert!(VerilatorVersion::new(6, 0) > VerilatorVersion::new(5, 999));
    assert_eq!(VerilatorVersion::new(5, 0), minimum_supported_version());
    assert!(VerilatorVersion::new(4, 999) < minimum_supported_version());
}

#[test]
fn rejects_versions_below_minimum_supported_version() {
    assert!(matches!(
        ensure_supported_version(VerilatorVersion::new(4, 999)),
        Err(BuildError::UnsupportedVerilatorVersion { .. })
    ));
}

#[test]
fn accepts_minimum_supported_version() -> Result<(), BuildError> {
    ensure_supported_version(minimum_supported_version())?;
    Ok(())
}

#[test]
fn constructs_verilator_model_command_in_expected_order() {
    let executable = OsStr::new("verilator");
    let output_dir = Path::new("/tmp/out");
    let hdl_includes = vec![
        PathBuf::from("rtl/include a"),
        PathBuf::from("rtl/include-b"),
    ];
    let defines = vec![Define::new("ENABLE", None), Define::new("WIDTH", Some("8"))];
    let raw_arguments = vec![OsString::from("--Wall"), OsString::from("--trace")];
    let sources = vec![PathBuf::from("rtl/a.sv"), PathBuf::from("rtl/b.sv")];

    let command = model_command(&ModelCommand {
        executable,
        top_module: "counter",
        model_prefix: "Vcounter",
        output_dir,
        hdl_include_dirs: &hdl_includes,
        defines: &defines,
        extra_arguments: &raw_arguments,
        sources: &sources,
        trace: None,
    });

    assert_eq!(command.get_program(), executable);

    let actual_arguments: Vec<OsString> = command.get_args().map(OsStr::to_os_string).collect();
    let expected_arguments = vec![
        OsString::from("--cc"),
        OsString::from("--top-module"),
        OsString::from("counter"),
        OsString::from("--prefix"),
        OsString::from("Vcounter"),
        OsString::from("--Mdir"),
        output_dir.as_os_str().to_os_string(),
        OsString::from("--emit-accessors"),
        include_argument(Path::new("rtl/include a")),
        include_argument(Path::new("rtl/include-b")),
        define_argument(&Define::new("ENABLE", None)),
        define_argument(&Define::new("WIDTH", Some("8"))),
        OsString::from("--Wall"),
        OsString::from("--trace"),
        PathBuf::from("rtl/a.sv").into_os_string(),
        PathBuf::from("rtl/b.sv").into_os_string(),
    ];

    assert_eq!(actual_arguments, expected_arguments);
}

#[test]
fn singular_and_plural_source_methods_append_in_order() {
    let builder = DutBuilder::new("counter")
        .source("a.sv")
        .sources(["b.sv", "c.sv"]);

    let actual: Vec<&Path> = builder
        .sources_slice()
        .iter()
        .map(PathBuf::as_path)
        .collect();
    let expected = vec![Path::new("a.sv"), Path::new("b.sv"), Path::new("c.sv")];

    assert_eq!(actual, expected);
}

#[test]
fn singular_and_plural_cpp_source_methods_append_in_order() {
    let builder = DutBuilder::new("counter")
        .cpp_source("a.cpp")
        .cpp_sources(["b.cpp", "c.cpp"]);

    let actual: Vec<&Path> = builder
        .cpp_sources_slice()
        .iter()
        .map(PathBuf::as_path)
        .collect();
    let expected = vec![Path::new("a.cpp"), Path::new("b.cpp"), Path::new("c.cpp")];

    assert_eq!(actual, expected);
}

#[test]
fn singular_and_plural_cpp_include_methods_append_in_order() {
    let builder = DutBuilder::new("counter")
        .cpp_include("cpp/a")
        .cpp_includes(["cpp/b", "cpp/c"]);

    let actual: Vec<&Path> = builder
        .cpp_include_dirs_slice()
        .iter()
        .map(PathBuf::as_path)
        .collect();
    let expected = vec![Path::new("cpp/a"), Path::new("cpp/b"), Path::new("cpp/c")];

    assert_eq!(actual, expected);
}

#[test]
fn singular_and_plural_verilator_arg_methods_append_in_order() {
    let builder = DutBuilder::new("counter")
        .verilator_arg("--Wall")
        .verilator_args(["--trace", "--timing"]);

    let actual = builder.verilator_arguments_slice();
    let expected = [
        OsString::from("--Wall"),
        OsString::from("--trace"),
        OsString::from("--timing"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn discovered_generated_sources_are_sorted_and_filtered() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = tempdir()?;
    for file_name in [
        "Vcounter__Syms.cpp",
        "Vcounter.cpp",
        "Vcounter__ALL.cpp",
        "Vcounter.h",
        "README.txt",
    ] {
        touch(&directory.path().join(file_name))?;
    }

    let actual = generated_sources(directory.path(), "Vcounter")?;
    let expected = vec![
        directory.path().join("Vcounter.cpp"),
        directory.path().join("Vcounter__Syms.cpp"),
    ];

    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn returns_error_when_generated_sources_are_missing() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    touch(&directory.path().join("Vcounter.h"))?;

    assert!(matches!(
        generated_sources(directory.path(), "Vcounter"),
        Err(BuildError::NoGeneratedSources { .. })
    ));
    Ok(())
}

#[test]
fn returns_io_error_for_nonexistent_generated_source_directory() {
    let directory = Path::new("/definitely/nonexistent/vvm-build-test-directory");

    assert!(matches!(
        generated_sources(directory, "Vcounter"),
        Err(BuildError::Io { .. })
    ));
}

#[test]
fn returns_io_error_for_unreadable_generated_source_directory_input()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let file_path = directory.path().join("not-a-directory");
    touch(&file_path)?;

    assert!(matches!(
        generated_sources(&file_path, "Vcounter"),
        Err(BuildError::Io { .. })
    ));
    Ok(())
}

#[test]
fn constructs_metadata_command_in_expected_order() {
    let executable = OsStr::new("verilator");
    let output = Path::new("/tmp/out/counter.tree.json");
    let meta_output = Path::new("/tmp/out/counter.tree.meta.json");

    let hdl_includes = vec![
        PathBuf::from("/rtl/include-a"),
        PathBuf::from("/rtl/include b"),
    ];

    let defines = vec![
        Define::new("WIDTH", Some("8")),
        Define::new("VVM_TEST", None),
    ];

    let extra_arguments = vec![
        OsString::from("--Wall"),
        OsString::from("--language"),
        OsString::from("1800-2017"),
    ];

    let sources = vec![
        PathBuf::from("/rtl/package.sv"),
        PathBuf::from("/rtl/counter.sv"),
    ];

    let metadata = MetadataCommand {
        executable,
        top_module: "counter",
        output,
        meta_output,
        hdl_include_dirs: &hdl_includes,
        defines: &defines,
        extra_arguments: &extra_arguments,
        sources: &sources,
    };

    let command = metadata_command(&metadata);

    assert_eq!(command.get_program(), OsStr::new("verilator"));

    let actual = command.get_args().map(OsStr::to_owned).collect::<Vec<_>>();

    let expected = vec![
        OsString::from("--json-only"),
        OsString::from("--json-only-output"),
        output.as_os_str().to_owned(),
        OsString::from("--json-only-meta-output"),
        meta_output.as_os_str().to_owned(),
        OsString::from("--no-json-edit-nums"),
        OsString::from("--top-module"),
        OsString::from("counter"),
        OsString::from("-I/rtl/include-a"),
        OsString::from("-I/rtl/include b"),
        OsString::from("-DWIDTH=8"),
        OsString::from("-DVVM_TEST"),
        OsString::from("--Wall"),
        OsString::from("--language"),
        OsString::from("1800-2017"),
        OsString::from("/rtl/package.sv"),
        OsString::from("/rtl/counter.sv"),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn accepts_existing_metadata_output_file() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let output = directory.path().join("counter.tree.json");

    touch(&output)?;

    ensure_metadata_output(&output, "AST metadata output")?;

    Ok(())
}

#[test]
fn rejects_missing_metadata_output() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let output = directory.path().join("missing.tree.json");

    let result = ensure_metadata_output(&output, "AST metadata output");

    assert!(matches!(
        result,
        Err(BuildError::MissingMetadataOutput {
            role: "AST metadata output",
            ..
        })
    ));

    Ok(())
}

#[test]
fn rejects_directory_as_metadata_output() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let output = directory.path().join("counter.tree.json");

    fs::create_dir_all(&output)?;

    let result = ensure_metadata_output(&output, "AST metadata output");

    assert!(matches!(
        result,
        Err(BuildError::MetadataOutputNotFile {
            role: "AST metadata output",
            ..
        })
    ));

    Ok(())
}
