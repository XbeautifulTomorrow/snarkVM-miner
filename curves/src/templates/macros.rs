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

// https://github.com/rust-lang/rust/issues/57966 forces us to export these and
// import them via `use crate::` syntax. It'd be nice if we were able to avoid any
// macro_use/macro_export and just import the macro
#[macro_export]
macro_rules! impl_sw_curve_serializer {
    ($params: ident) => {
        // Projective Group point implementations delegate to the Affine version
        impl<P: $params> CanonicalSerialize for Projective<P> {
            #[allow(unused_qualifications)]
            #[inline]
            fn serialize_with_mode<W: snarkvm_utilities::io::Write>(
                &self,
                writer: W,
                compress: Compress,
            ) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                CanonicalSerialize::serialize_with_mode(&Affine::<P>::from(*self), writer, compress)
            }

            #[inline]
            fn serialized_size(&self, compress: Compress) -> usize {
                Affine::<P>::from(*self).serialized_size(compress)
            }

            #[inline]
            fn uncompressed_size(&self) -> usize {
                Affine::<P>::from(*self).uncompressed_size()
            }
        }

        impl<P: $params> Valid for Projective<P> {
            fn check(&self) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                let point = Affine::<P>::from(*self);
                if point.is_on_curve() & point.is_in_correct_subgroup_assuming_on_curve() {
                    Ok(())
                } else {
                    Err(snarkvm_utilities::errors::SerializationError::InvalidData)
                }
            }
        }

        impl<P: $params> CanonicalDeserialize for Projective<P> {
            #[allow(unused_qualifications)]
            fn deserialize_with_mode<R: snarkvm_utilities::io::Read>(
                reader: R,
                compress: Compress,
                validate: Validate,
            ) -> Result<Self, snarkvm_utilities::errors::SerializationError> {
                Affine::<P>::deserialize_with_mode(reader, compress, validate).map(Into::into)
            }
        }

        impl<P: $params> CanonicalSerialize for Affine<P> {
            #[allow(unused_qualifications)]
            #[inline]
            fn serialize_with_mode<W: snarkvm_utilities::io::Write>(
                &self,
                mut writer: W,
                compress: Compress,
            ) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                match compress {
                    Compress::Yes => {
                        if self.is_zero() {
                            let flags = snarkvm_utilities::serialize::SWFlags::infinity();
                            // Serialize 0.
                            P::BaseField::zero().serialize_with_flags(writer, flags)
                        } else {
                            let flags = snarkvm_utilities::serialize::SWFlags::from_y_sign(self.y > -self.y);
                            self.x.serialize_with_flags(writer, flags)
                        }
                    }
                    Compress::No => {
                        let flags = if self.is_zero() {
                            snarkvm_utilities::serialize::SWFlags::infinity()
                        } else {
                            snarkvm_utilities::serialize::SWFlags::default()
                        };
                        self.x.serialize_uncompressed(&mut writer)?;
                        self.y.serialize_with_flags(&mut writer, flags)?;
                        Ok(())
                    }
                }
            }

            #[inline]
            fn serialized_size(&self, compress: Compress) -> usize {
                match compress {
                    Compress::Yes => self.x.serialized_size_with_flags::<SWFlags>(),
                    Compress::No => self.x.serialized_size(compress) + self.y.serialized_size_with_flags::<SWFlags>(),
                }
            }
        }

        impl<P: $params> Valid for Affine<P> {
            fn check(&self) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                if self.is_on_curve() & self.is_in_correct_subgroup_assuming_on_curve() {
                    Ok(())
                } else {
                    Err(snarkvm_utilities::errors::SerializationError::InvalidData)
                }
            }
        }

        impl<P: $params> CanonicalDeserialize for Affine<P> {
            #[allow(unused_qualifications)]
            fn deserialize_with_mode<R: snarkvm_utilities::io::Read>(
                mut reader: R,
                compress: Compress,
                validate: Validate,
            ) -> Result<Self, snarkvm_utilities::errors::SerializationError> {
                use snarkvm_utilities::serialize::SWFlags;
                let point = if let Compress::Yes = compress {
                    let (x, flags) = P::BaseField::deserialize_with_flags::<_, SWFlags>(&mut reader)?;
                    if flags.is_infinity() {
                        Self::zero()
                    } else {
                        Affine::<P>::from_x_coordinate(x, flags.is_positive().unwrap())
                            .ok_or(snarkvm_utilities::errors::SerializationError::InvalidData)?
                    }
                } else {
                    let x = P::BaseField::deserialize_uncompressed(&mut reader)?;
                    let (y, flags) = P::BaseField::deserialize_with_flags::<_, SWFlags>(&mut reader)?;
                    Affine::<P>::new(x, y, flags.is_infinity())
                };
                if !snarkvm_utilities::PROCESSING_SNARK_PARAMS.with(|p| p.load(std::sync::atomic::Ordering::Relaxed))
                    && validate == Validate::Yes
                {
                    point.check()?;
                } else {
                    snarkvm_utilities::SNARK_PARAMS_AFFINE_COUNT
                        .with(|p| p.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
                }
                Ok(point)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_edwards_curve_serializer {
    ($params: ident) => {
        impl<P: $params> CanonicalSerialize for Projective<P> {
            #[allow(unused_qualifications)]
            #[inline]
            fn serialize_with_mode<W: snarkvm_utilities::io::Write>(
                &self,
                writer: W,
                compress: Compress,
            ) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                Affine::<P>::from(*self).serialize_with_mode(writer, compress)
            }

            #[inline]
            fn serialized_size(&self, compress: Compress) -> usize {
                Affine::<P>::from(*self).serialized_size(compress)
            }
        }

        impl<P: $params> Valid for Projective<P> {
            fn check(&self) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                let point = Affine::<P>::from(*self);
                if point.is_on_curve() & point.is_in_correct_subgroup_assuming_on_curve() {
                    Ok(())
                } else {
                    Err(snarkvm_utilities::errors::SerializationError::InvalidData)
                }
            }
        }

        impl<P: $params> CanonicalDeserialize for Projective<P> {
            #[allow(unused_qualifications)]
            fn deserialize_with_mode<R: snarkvm_utilities::io::Read>(
                reader: R,
                compress: Compress,
                validate: Validate,
            ) -> Result<Self, snarkvm_utilities::errors::SerializationError> {
                Affine::<P>::deserialize_with_mode(reader, compress, validate).map(Into::into)
            }
        }

        impl<P: $params> CanonicalSerialize for Affine<P> {
            #[allow(unused_qualifications)]
            #[inline]
            fn serialize_with_mode<W: snarkvm_utilities::io::Write>(
                &self,
                mut writer: W,
                compress: Compress,
            ) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                if let Compress::Yes = compress {
                    if self.is_zero() {
                        let flags = snarkvm_utilities::serialize::EdwardsFlags::default();
                        // Serialize 0.
                        P::BaseField::zero().serialize_with_flags(&mut writer, flags)
                    } else {
                        let flags = snarkvm_utilities::serialize::EdwardsFlags::from_y_sign(self.y > -self.y);
                        self.x.serialize_with_flags(writer, flags)
                    }
                } else {
                    self.x.serialize_uncompressed(&mut writer)?;
                    self.y.serialize_uncompressed(&mut writer)?;
                    Ok(())
                }
            }

            #[inline]
            fn serialized_size(&self, compress: Compress) -> usize {
                if let Compress::Yes = compress {
                    use snarkvm_utilities::serialize::EdwardsFlags;
                    self.x.serialized_size_with_flags::<EdwardsFlags>()
                } else {
                    self.x.uncompressed_size() + self.y.uncompressed_size()
                }
            }
        }

        impl<P: $params> Valid for Affine<P> {
            #[allow(unused_qualifications)]
            fn check(&self) -> Result<(), snarkvm_utilities::errors::SerializationError> {
                if self.is_on_curve() & self.is_in_correct_subgroup_assuming_on_curve() {
                    Ok(())
                } else {
                    Err(snarkvm_utilities::errors::SerializationError::InvalidData)
                }
            }
        }

        impl<P: $params> CanonicalDeserialize for Affine<P> {
            #[allow(unused_qualifications)]
            fn deserialize_with_mode<R: snarkvm_utilities::io::Read>(
                mut reader: R,
                compress: Compress,
                validate: Validate,
            ) -> Result<Self, snarkvm_utilities::errors::SerializationError> {
                use snarkvm_utilities::{errors::SerializationError, serialize::EdwardsFlags};
                let point = if let Compress::Yes = compress {
                    let (x, flags): (P::BaseField, EdwardsFlags) = P::BaseField::deserialize_with_flags(&mut reader)?;

                    if x == P::BaseField::zero() {
                        Self::zero()
                    } else {
                        Affine::<P>::from_x_coordinate(x, flags.is_positive()).ok_or(SerializationError::InvalidData)?
                    }
                } else {
                    let x = P::BaseField::deserialize_uncompressed(&mut reader)?;
                    let y = P::BaseField::deserialize_uncompressed(&mut reader)?;
                    Affine::<P>::new(x, y)
                };

                if let Validate::Yes = validate {
                    Valid::check(&point)?;
                }

                Ok(point)
            }
        }
    };
}
