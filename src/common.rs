use std::io::Read;

use borsh::{BorshDeserialize, BorshSerialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTimeUtc(pub DateTime<Utc>);

impl DateTimeUtc {
    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.9f+00:00";

    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn to_rfc3339(&self) -> String {
        self.0.format(DateTimeUtc::FORMAT).to_string()
    }

    /// Parses a rfc3339 string, or returns an error.
    pub fn from_rfc3339(s: &str) -> Result<Self, chrono::format::ParseError> {
        use chrono::format;
        use chrono::format::strftime::StrftimeItems;

        let format = StrftimeItems::new(Self::FORMAT);
        let mut parsed = format::Parsed::new();
        format::parse(&mut parsed, s, format)?;

        parsed.to_datetime_with_timezone(&chrono::Utc).map(Self)
    }
}

impl BorshSerialize for DateTimeUtc {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let raw = self.to_rfc3339();
        BorshSerialize::serialize(&raw, writer)
    }
}

impl BorshDeserialize for DateTimeUtc {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};
        let raw: String = BorshDeserialize::deserialize_reader(reader)?;
        Self::from_rfc3339(&raw).map_err(|err| Error::new(ErrorKind::InvalidData, err))
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Address([u8; tendermint::account::LENGTH]);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ChainId(pub String);
