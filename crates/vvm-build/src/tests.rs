use crate::validate_identifier;

#[test]
fn accepts_cpp_identifiers() {
    for identifier in ["counter", "_counter", "counter_2"] {
        validate_identifier("test", identifier).expect("identifier validation should pass");
    }
}

#[test]
fn rejects_invalid_cpp_identifiers() {
    for identifier in ["", "2counter", "counter-name"] {
        validate_identifier("test", identifier)
            .expect_err("invalid identifiers should err validation");
    }
}
