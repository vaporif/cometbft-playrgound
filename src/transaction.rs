use std::num::NonZeroU128;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::common::{Address, ChainId, DateTimeUtc};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Header {
    pub chain_id: ChainId,
    pub expiration: Option<DateTimeUtc>,
    pub timestamp: DateTimeUtc,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Transaction {
    pub header: Header,
    pub chain_id: ChainId,
    pub from: Address,
    pub tx_payload: TxPayload,
    pub nonce: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum TxPayload {
    CreateAccount,
    Transfer { to: Address, amount: NonZeroU128 },
}
