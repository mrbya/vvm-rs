/// Returns whether one name is a valid coverage identifier.
pub fn is_valid_coverage_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();

    let Some(first) = bytes.next() else {
        return false;
    };

    let valid_first = first.is_ascii_alphabetic() || first == b'_';

    valid_first && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// Returns whether one hierarchical coverage path is valid.
pub fn is_valid_coverage_path(path: &str) -> bool {
    !path.is_empty() && path.split('.').all(is_valid_coverage_identifier)
}
