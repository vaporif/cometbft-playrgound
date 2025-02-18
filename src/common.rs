use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Address([u8; tendermint::account::LENGTH]);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ChainId(String);
