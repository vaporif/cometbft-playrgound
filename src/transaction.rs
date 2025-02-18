use std::num::NonZeroU128;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::common::{Address, ChainId};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Transaction {
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
