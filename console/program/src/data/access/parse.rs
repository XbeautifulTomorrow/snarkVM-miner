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

use super::*;

impl<N: Network> Parser for Access<N> {
    fn parse(string: &str) -> ParserResult<Self>
    where
        Self: Sized,
    {
        // A helper function to parse an index access.
        fn parse_index(string: &str) -> ParserResult<u32> {
            // Parse the opening bracket '['.
            let (string, _) = tag("[")(string)?;
            // Parse the digits from the string.
            let (string, primitive) = recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(string)?;
            // Parse the closing bracket ']' and return the value as a U32.
            map_res(tag("]"), |_| primitive.replace('_', "").parse())(string)
        }

        alt((
            map(parse_index, |index| Self::Index(U32::new(index))),
            map(pair(tag("."), Identifier::parse), |(_, identifier)| Self::Member(identifier)),
        ))(string)
    }
}

impl<N: Network> FromStr for Access<N> {
    type Err = Error;

    /// Parses an identifier into an access.
    #[inline]
    fn from_str(string: &str) -> Result<Self> {
        match Self::parse(string) {
            Ok((remainder, object)) => {
                // Ensure the remainder is empty.
                ensure!(remainder.is_empty(), "Failed to parse string. Found invalid character in: \"{remainder}\"");
                // Return the object.
                Ok(object)
            }
            Err(error) => bail!("Failed to parse string. {error}"),
        }
    }
}

impl<N: Network> Debug for Access<N> {
    /// Prints the access as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network> Display for Access<N> {
    /// Prints the access as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            // Prints the access index, i.e. `[0]`
            Self::Index(index) => write!(f, "[{}]", **index),
            // Prints the access member, i.e. `.foo`
            Self::Member(identifier) => write!(f, ".{}", identifier),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network::Testnet3;

    type CurrentNetwork = Testnet3;

    #[test]
    fn test_parse() -> Result<()> {
        assert_eq!(Access::parse("[0]"), Ok(("", Access::<CurrentNetwork>::Index(U32::new(0)))));
        assert_eq!(Access::parse(".data"), Ok(("", Access::<CurrentNetwork>::Member(Identifier::from_str("data")?))));
        Ok(())
    }

    #[test]
    fn test_parse_fails() -> Result<()> {
        // Must be non-empty.
        assert!(Access::<CurrentNetwork>::parse("").is_err());
        assert!(Access::<CurrentNetwork>::parse(".").is_err());
        assert!(Access::<CurrentNetwork>::parse("[]").is_err());

        // Invalid accesses.
        assert!(Access::<CurrentNetwork>::parse(".0").is_err());
        assert!(Access::<CurrentNetwork>::parse("[index]").is_err());
        assert!(Access::<CurrentNetwork>::parse("[0.0]").is_err());
        assert!(Access::<CurrentNetwork>::parse("[999999999999]").is_err());

        // Must fit within the data capacity of a base field element.
        let access =
            Access::<CurrentNetwork>::parse(".foo_bar_baz_qux_quux_quuz_corge_grault_garply_waldo_fred_plugh_xyzzy");
        assert!(access.is_err());

        Ok(())
    }

    #[test]
    fn test_display() -> Result<()> {
        assert_eq!(Access::<CurrentNetwork>::Index(U32::new(0)).to_string(), "[0]");
        assert_eq!(Access::<CurrentNetwork>::Member(Identifier::from_str("foo")?).to_string(), ".foo");
        Ok(())
    }
}