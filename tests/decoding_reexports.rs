// SPDX-License-Identifier: CC0-1.0

use core::marker::PhantomData;

fn assert_same_type<T>(_: PhantomData<T>, _: PhantomData<T>) {}

#[test]
fn error_types_are_shared_with_decoding_crate() {
    assert_same_type(
        PhantomData::<hex_conservative::InvalidCharError>,
        PhantomData::<hex_conservative_decoding::InvalidCharError>,
    );
    assert_same_type(
        PhantomData::<hex_conservative::OddLengthStringError>,
        PhantomData::<hex_conservative_decoding::OddLengthStringError>,
    );
    assert_same_type(
        PhantomData::<hex_conservative::InvalidLengthError>,
        PhantomData::<hex_conservative_decoding::InvalidLengthError>,
    );
    assert_same_type(
        PhantomData::<hex_conservative::DecodeFixedLengthBytesError>,
        PhantomData::<hex_conservative_decoding::DecodeFixedLengthBytesError>,
    );
    assert_same_type(
        PhantomData::<hex_conservative::DecodeVariableLengthBytesError>,
        PhantomData::<hex_conservative_decoding::DecodeVariableLengthBytesError>,
    );
    assert_same_type(
        PhantomData::<hex_conservative::error::InvalidCharError>,
        PhantomData::<hex_conservative_decoding::error::InvalidCharError>,
    );
}

#[test]
fn iterator_types_are_shared_with_decoding_crate() {
    assert_same_type(
        PhantomData::<hex_conservative::HexToBytesIter<core::array::IntoIter<[u8; 2], 1>>>,
        PhantomData::<hex_conservative_decoding::HexToBytesIter<core::array::IntoIter<[u8; 2], 1>>>,
    );
    assert_same_type(
        PhantomData::<hex_conservative::HexSliceToBytesIter<'static>>,
        PhantomData::<hex_conservative_decoding::HexSliceToBytesIter<'static>>,
    );
}
