// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod bytes;
mod parse;
mod serialize;

use crate::{ElementType, U32};
use snarkvm_console_network::prelude::*;

use core::fmt::{Debug, Display};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ArrayType<N: Network> {
    /// The type of the elements in the array.
    element_type: ElementType<N>,
    /// The length of the array.
    length: U32<N>,
}

impl<N: Network> ArrayType<N> {
    /// Constructs a new array type.
    pub fn new(element_type: ElementType<N>, length: U32<N>) -> Result<Self> {
        ensure!(*length != 0, "The array must have at least one element");
        ensure!(
            *length as usize <= N::MAX_ARRAY_ENTRIES,
            "The array must have at most {} elements",
            N::MAX_ARRAY_ENTRIES
        );
        Ok(Self { element_type, length })
    }

    /// Returns the element type.
    pub fn element_type(&self) -> &ElementType<N> {
        &self.element_type
    }

    /// Returns the length of the array.
    pub fn length(&self) -> &U32<N> {
        &self.length
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network::Testnet3;

    use crate::{Identifier, LiteralType};
    use core::str::FromStr;

    type CurrentNetwork = Testnet3;

    #[test]
    fn test_array_type() -> Result<()> {
        // Test literal array types.
        let type_ = ArrayType::<CurrentNetwork>::from_str("[field; 4]")?;
        assert_eq!(type_, ArrayType::<CurrentNetwork>::new(ElementType::from(LiteralType::Field), U32::new(4))?);
        assert_eq!(
            type_.to_bytes_le()?,
            ArrayType::<CurrentNetwork>::from_bytes_le(&type_.to_bytes_le()?)?.to_bytes_le()?
        );
        assert_eq!(type_.element_type(), &ElementType::from(LiteralType::Field));
        assert_eq!(type_.length(), &U32::new(4));

        // Test struct array types.
        let type_ = ArrayType::<CurrentNetwork>::from_str("[foo; 1]")?;
        assert_eq!(
            type_,
            ArrayType::<CurrentNetwork>::new(ElementType::from(Identifier::from_str("foo")?), U32::new(1))?
        );
        assert_eq!(
            type_.to_bytes_le()?,
            ArrayType::<CurrentNetwork>::from_bytes_le(&type_.to_bytes_le()?)?.to_bytes_le()?
        );
        assert_eq!(type_.element_type(), &ElementType::from(Identifier::from_str("foo")?));
        assert_eq!(type_.length(), &U32::new(1));

        // Test array type with maximum length.
        let type_ = ArrayType::<CurrentNetwork>::from_str("[scalar; 4294967295]")?;
        assert_eq!(
            type_,
            ArrayType::<CurrentNetwork>::new(ElementType::from(LiteralType::Scalar), U32::new(4294967295))?
        );
        assert_eq!(
            type_.to_bytes_le()?,
            ArrayType::<CurrentNetwork>::from_bytes_le(&type_.to_bytes_le()?)?.to_bytes_le()?
        );
        assert_eq!(type_.element_type(), &ElementType::from(LiteralType::Scalar));
        assert_eq!(type_.length(), &U32::new(4294967295));

        Ok(())
    }

    #[test]
    fn test_array_type_fails() -> Result<()> {
        let type_ = ArrayType::<CurrentNetwork>::from_str("[field; 0]");
        assert!(type_.is_err());

        let type_ = ArrayType::<CurrentNetwork>::from_str("[field; 4294967296]");
        assert!(type_.is_err());

        let type_ = ArrayType::<CurrentNetwork>::from_str("[foo; -1]");
        assert!(type_.is_err());

        Ok(())
    }
}