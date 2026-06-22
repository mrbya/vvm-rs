use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use crate::builder::{ensure_unique_paths, resolve_directory, resolve_file, validate_identifier};
use crate::error::BuildError;
use crate::verilator::{generated_sources, model_command};

fn touch(path: &Path) -> std::io::Result<()> {
    fs::write(path, b"")
}

#[test]
fn accepts_valid_ascii_identifiers() {
    for identifier in ["counter", "_counter", "counter_2"] {
        validate_identifier("identifier", identifier).expect("validation should pass");
    }
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

    resolve_file(directory.path(), Path::new("counter.sv"), "HDL source file")
        .expect("validation should pass");
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
fn constructs_verilator_model_command_in_expected_order() {
    let executable = OsStr::new("verilator");
    let output_dir = Path::new("/tmp/out");
    let sources = vec![PathBuf::from("rtl/a.sv"), PathBuf::from("rtl/b.sv")];
    let extra_arguments = vec![OsString::from("--timing"), OsString::from("-Wall")];

    let command = model_command(
        executable,
        "counter",
        "Vcounter",
        output_dir,
        &sources,
        &extra_arguments,
    );

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
        OsString::from("--timing"),
        OsString::from("-Wall"),
        PathBuf::from("rtl/a.sv").into_os_string(),
        PathBuf::from("rtl/b.sv").into_os_string(),
    ];

    assert_eq!(actual_arguments, expected_arguments);
}

#[test]
fn discovered_generated_sources_are_sorted_and_filtered() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = tempdir()?;
    let files = [
        "Vcounter__Syms.cpp",
        "Vcounter.cpp",
        "Vcounter__ALL.cpp",
        "Vcounter.h",
        "README.txt",
    ];

    for file_name in files {
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

    let result = generated_sources(directory.path(), "Vcounter");

    assert!(matches!(result, Err(BuildError::NoGeneratedSources { .. })));
    Ok(())
}

#[test]
fn returns_io_error_for_nonexistent_generated_source_directory() {
    let directory = Path::new("/definitely/nonexistent/vvm-build-test-directory");

    let result = generated_sources(directory, "Vcounter");

    assert!(matches!(result, Err(BuildError::Io { .. })));
}

#[test]
fn returns_io_error_for_unreadable_generated_source_directory_input()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempdir()?;
    let file_path = directory.path().join("not-a-directory");
    touch(&file_path)?;

    let result = generated_sources(&file_path, "Vcounter");

    assert!(matches!(result, Err(BuildError::Io { .. })));
    Ok(())
}
