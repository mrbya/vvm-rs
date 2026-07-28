//! Packed bit vectors and layout-aware extraction and insertion utilities.

pub use vvm_core::{
    Bits, InvalidBitVectorWordCount as WordCountError, PackedEnumLayout, PackedEnumVariantLayout,
    PackedFieldLayout, PackedLayout, PackedLayoutError as LayoutError, PackedRange, PackedValue,
    SignedBits, UnpackedArrayIndexError as UnpackedIndexError, extract_packed, extract_signed,
    extract_signed_packed, extract_unsigned, insert_packed, insert_signed, insert_signed_packed,
    insert_unsigned,
};
