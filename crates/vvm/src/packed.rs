//! Packed bit vectors and layout-aware extraction and insertion utilities.

pub use vvm_core::{
    Bits, InvalidBitVectorWordCount, PackedEnumLayout, PackedEnumVariantLayout, PackedFieldLayout,
    PackedLayout, PackedLayoutError, PackedRange, PackedValue, SignedBits, UnpackedArrayIndexError,
    extract_packed, extract_signed, extract_signed_packed, extract_unsigned, insert_packed,
    insert_signed, insert_signed_packed, insert_unsigned,
};
