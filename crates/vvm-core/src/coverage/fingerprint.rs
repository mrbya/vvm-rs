use core::fmt;
use core::str::FromStr;

use sha2::{Digest, Sha256};

use crate::coverage::{
    BinKind, BinMatcherKind, CoverageGroupSnapshot, CoverageItemSnapshot, CoveragePersistenceError,
};

/// Stable structural fingerprint of one coverage-group definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoverageDefinitionFingerprint {
    /// SHA-256 digest of canonical definition metadata.
    digest: [u8; 32],
}

impl CoverageDefinitionFingerprint {
    /// Fingerprint algorithm name.
    pub const ALGORITHM: &'static str = "sha256";
    /// Canonical fingerprint format version.
    pub const VERSION: u32 = 1;
    /// Returns the raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.digest
    }
    /// Calculates a fingerprint from one immutable group snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error when canonical collection sizes cannot fit in `u64`.
    pub fn from_group(group: &CoverageGroupSnapshot) -> Result<Self, CoveragePersistenceError> {
        let mut encoder = Encoder::new();
        encoder.string(group.definition_name())?;
        encoder.u64(group.definition_revision());
        encoder.list_len(group.items().len())?;

        for item in group.items() {
            encode_item(&mut encoder, item)?;
        }

        let digest = Sha256::digest(encoder.bytes);
        let mut bytes = [0_u8; 32];
        bytes.copy_from_slice(&digest);
        Ok(Self { digest: bytes })
    }
}

impl fmt::Display for CoverageDefinitionFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("sha256-v1:")?;
        for byte in self.digest {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for CoverageDefinitionFingerprint {
    type Err = CoveragePersistenceError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some(hex) = value.strip_prefix("sha256-v1:") else {
            return Err(invalid(value));
        };
        if hex.len() != 64
            || !hex.bytes().all(|byte| {
                byte.is_ascii_digit() || (byte.is_ascii_lowercase() && byte.is_ascii_hexdigit())
            })
        {
            return Err(invalid(value));
        }

        let mut digest = [0_u8; 32];
        for (index, chunk) in hex.as_bytes().chunks_exact(2).enumerate() {
            let text = std::str::from_utf8(chunk).map_err(|_error| invalid(value))?;
            let byte = u8::from_str_radix(text, 16).map_err(|_error| invalid(value))?;
            let destination = digest.get_mut(index).ok_or_else(|| invalid(value))?;
            *destination = byte;
        }
        Ok(Self { digest })
    }
}

/// Encodes one ordered item definition.
fn encode_item(
    encoder: &mut Encoder,
    item: &CoverageItemSnapshot,
) -> Result<(), CoveragePersistenceError> {
    match *item {
        CoverageItemSnapshot::Coverpoint(ref point) => {
            encoder.u8(0);
            encoder.string(point.name())?;
            encoder.list_len(point.bins().len())?;
            for bin in point.bins() {
                encoder.u32(bin.id().ordinal());
                encoder.string(bin.name())?;
                encoder.u8(bin_kind_tag(bin.kind()));
                encoder.u64(bin.required_hits());
                encoder.u8(matcher_kind_tag(bin.matcher_kind()));
                encoder.list_len(bin.matcher_operand_count())?;
            }
        }
        CoverageItemSnapshot::Cross2(ref cross) => {
            encoder.u8(1);
            encoder.string(cross.name())?;
            encoder.string(cross.left_coverpoint_name())?;
            encoder.string(cross.right_coverpoint_name())?;
            encoder.list_len(cross.bins().len())?;
            for bin in cross.bins() {
                encoder.u32(bin.id().ordinal());
                encoder.u32(bin.left_bin_id().ordinal());
                encoder.string(bin.left_bin_name())?;
                encoder.u32(bin.right_bin_id().ordinal());
                encoder.string(bin.right_bin_name())?;
                encoder.u64(bin.required_hits());
            }
        }
    }
    Ok(())
}
/// Returns a structured fingerprint parse failure.
fn invalid(value: &str) -> CoveragePersistenceError {
    CoveragePersistenceError::InvalidFingerprint {
        value: value.to_owned(),
    }
}
/// Returns the canonical normal/ignore/illegal tag.
const fn bin_kind_tag(kind: BinKind) -> u8 {
    match kind {
        BinKind::Normal => 0,
        BinKind::Ignore => 1,
        BinKind::Illegal => 2,
    }
}
/// Returns the canonical matcher-shape tag.
const fn matcher_kind_tag(kind: BinMatcherKind) -> u8 {
    match kind {
        BinMatcherKind::Value => 0,
        BinMatcherKind::Values => 1,
        BinMatcherKind::InclusiveRange => 2,
    }
}
/// Canonical binary definition encoder.
struct Encoder {
    /// Encoded bytes.
    bytes: Vec<u8>,
}
impl Encoder {
    /// Starts an encoder with the versioned domain separator.
    fn new() -> Self {
        Self {
            bytes: b"vvm.functional-coverage.group-definition.sha256.v1\0".to_vec(),
        }
    }
    /// Encodes one byte.
    fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }
    /// Encodes one big-endian integer.
    fn u32(&mut self, value: u32) {
        self.bytes.extend(value.to_be_bytes());
    }
    /// Encodes one big-endian integer.
    fn u64(&mut self, value: u64) {
        self.bytes.extend(value.to_be_bytes());
    }
    /// Encodes one list length.
    fn list_len(&mut self, value: usize) -> Result<(), CoveragePersistenceError> {
        self.u64(u64::try_from(value).map_err(|_error| {
            CoveragePersistenceError::NumericOverflow {
                field: "fingerprint list length".to_owned(),
            }
        })?);
        Ok(())
    }
    /// Encodes one length-prefixed UTF-8 string.
    fn string(&mut self, value: &str) -> Result<(), CoveragePersistenceError> {
        self.list_len(value.len())?;
        self.bytes.extend(value.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::CoverageDefinitionFingerprint;

    #[test]
    fn fingerprint_parses_its_stable_display_form() -> Result<(), Box<dyn std::error::Error>> {
        let text = "sha256-v1:0000000000000000000000000000000000000000000000000000000000000000";
        let fingerprint = CoverageDefinitionFingerprint::from_str(text)?;

        assert_eq!(fingerprint.to_string(), text);
        assert_eq!(fingerprint.as_bytes(), &[0_u8; 32]);
        Ok(())
    }

    #[test]
    fn fingerprint_rejects_noncanonical_strings() {
        for value in [
            "sha256-v2:0000000000000000000000000000000000000000000000000000000000000000",
            "sha256-v1:000000000000000000000000000000000000000000000000000000000000000",
            "sha256-v1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "sha256-v1:000000000000000000000000000000000000000000000000000000000000000g",
        ] {
            assert!(matches!(
                CoverageDefinitionFingerprint::from_str(value),
                Err(crate::CoveragePersistenceError::InvalidFingerprint { .. })
            ));
        }
    }
}
