use std::{fmt::Display, num::ParseIntError, str::FromStr};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[derive(
    Clone,
    Copy,
    BorshSerialize,
    BorshDeserialize,
    BorshDeserializer,
    BorshSchema,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
)]
pub struct BlockHeight(pub u64);

impl Default for BlockHeight {
    fn default() -> Self {
        Self::sentinel()
    }
}

impl Display for BlockHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<BlockHeight> for u64 {
    fn from(height: BlockHeight) -> Self {
        height.0
    }
}

impl FromStr for BlockHeight {
    type Err = ParseIntError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(s.parse::<u64>()?))
    }
}
