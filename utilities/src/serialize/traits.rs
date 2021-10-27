// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::SerializationError;
pub use crate::{
    io::{
        Read,
        Write,
        {self},
    },
    FromBytes,
    ToBytes,
    Vec,
};

/// Represents metadata to be appended to an object's serialization. For
/// example, when serializing elliptic curve points, one can
/// use a `Flag` to represent whether the serialization is the point
/// at infinity, or whether the `y` coordinate is positive or not.
/// These bits will be appended to the end of the point's serialization,
/// or included in a new byte, depending on space available.
///
/// This is meant to be provided to `CanonicalSerializeWithFlags` and
/// `CanonicalDeserializeWithFlags`
pub trait Flags: Default + Clone + Copy + Sized {
    /// The number of bits required to encode `Self`.
    /// This should be at most 8.
    const BIT_SIZE: usize;

    // Returns a bit mask corresponding to `self`.
    // For example, if `Self` contains two variants, there are just two possible
    // bit masks: `0` and `1 << 7`.
    fn u8_bitmask(&self) -> u8;

    // Tries to read `Self` from `value`. Should return `None` if the `Self::BIT_SIZE`
    // most-significant bits of `value` do not correspond to those generated by
    // `u8_bitmask`.
    //
    // That is, this method ignores all but the top `Self::BIT_SIZE` bits, and
    // decides whether these top bits correspond to a bitmask output by `u8_bitmask`.
    fn from_u8(value: u8) -> Option<Self>;

    // Convenience method that reads `Self` from `value`, just like `Self::from_u8`, but
    // additionally zeroes out the bits corresponding to the resulting flag in `value`.
    // If `Self::from_u8(*value)` would return `None`, then this method should
    // *not* modify `value`.
    fn from_u8_remove_flags(value: &mut u8) -> Option<Self> {
        let flags = Self::from_u8(*value);
        if let Some(f) = flags {
            *value &= !f.u8_bitmask();
        }
        flags
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Compress {
    Yes,
    No,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Validate {
    Yes,
    No,
}

pub trait Valid: Sized {
    fn check(&self) -> Result<(), SerializationError>;

    fn batch_check<'a>(batch: impl Iterator<Item = &'a Self>) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        for item in batch {
            item.check()?;
        }
        Ok(())
    }
}

/// Serializer in little endian format.
/// This trait can be derived if all fields of a struct implement
/// `CanonicalSerialize` and the `derive` feature is enabled.
///
/// # Example
/// ```
/// // The `derive` feature must be set for the derivation to work.
/// use snarkvm_utilities::serialize::*;
/// use snarkvm_utilities::errors::SerializationError;
///
/// # #[cfg(feature = "derive")]
/// #[derive(CanonicalSerialize)]
/// struct TestStruct {
///     a: u64,
///     b: (u64, (u64, u64)),
/// }
/// ```
pub trait CanonicalSerialize {
    fn serialize_with_mode<W: Write>(&self, writer: W, compress: Compress) -> Result<(), SerializationError>;

    fn serialized_size(&self, compress: Compress) -> usize;

    fn serialize_compressed<W: Write>(&self, writer: W) -> Result<(), SerializationError> {
        self.serialize_with_mode(writer, Compress::Yes)
    }

    fn compressed_size(&self) -> usize {
        self.serialized_size(Compress::Yes)
    }

    fn serialize_uncompressed<W: Write>(&self, writer: W) -> Result<(), SerializationError> {
        self.serialize_with_mode(writer, Compress::No)
    }

    fn uncompressed_size(&self) -> usize {
        self.serialized_size(Compress::No)
    }
}

/// Deserializer in little endian format.
/// This trait can be derived if all fields of a struct implement
/// `CanonicalDeserialize` and the `derive` feature is enabled.
///
/// # Example
/// ```
/// // The `derive` feature must be set for the derivation to work.
/// use snarkvm_utilities::serialize::*;
/// use snarkvm_utilities::errors::SerializationError;
///
/// # #[cfg(feature = "derive")]
/// #[derive(CanonicalDeserialize)]
/// struct TestStruct {
///     a: u64,
///     b: (u64, (u64, u64)),
/// }
/// ```
pub trait CanonicalDeserialize: Valid {
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError>;

    fn deserialize_compressed<R: Read>(reader: R) -> Result<Self, SerializationError> {
        Self::deserialize_with_mode(reader, Compress::Yes, Validate::Yes)
    }

    fn deserialize_compressed_unchecked<R: Read>(reader: R) -> Result<Self, SerializationError> {
        Self::deserialize_with_mode(reader, Compress::Yes, Validate::No)
    }

    fn deserialize_uncompressed<R: Read>(reader: R) -> Result<Self, SerializationError> {
        Self::deserialize_with_mode(reader, Compress::No, Validate::Yes)
    }

    fn deserialize_uncompressed_unchecked<R: Read>(reader: R) -> Result<Self, SerializationError> {
        Self::deserialize_with_mode(reader, Compress::No, Validate::No)
    }
}

/// Serializer in little endian format allowing to encode flags.
pub trait CanonicalSerializeWithFlags: CanonicalSerialize {
    /// Serializes `self` and `flags` into `writer`.
    fn serialize_with_flags<W: Write, F: Flags>(&self, writer: W, flags: F) -> Result<(), SerializationError>;

    /// Serializes `self` and `flags` into `writer`.
    fn serialized_size_with_flags<F: Flags>(&self) -> usize;
}

/// Deserializer in little endian format allowing flags to be encoded.
pub trait CanonicalDeserializeWithFlags: Sized {
    /// Reads `Self` and `Flags` from `reader`.
    /// Returns empty flags by default.
    fn deserialize_with_flags<R: Read, F: Flags>(reader: R) -> Result<(Self, F), SerializationError>;
}
