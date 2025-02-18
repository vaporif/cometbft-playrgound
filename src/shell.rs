use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use thiserror::Error;

use crate::{
    common::{Address, ChainId, DateTimeUtc},
    transaction::{Transaction, TxPayload},
};

pub mod abci;
struct MerkleTree<'a, S> {
    accounts: jmt::Sha256Jmt<'a, S>,
}

#[derive(Clone, Debug)]
pub struct Shell {
    pub chain_id: ChainId,
    // TODO: arc rwlocks should be removed in favor of proper storage
    //state: cnidarium::Storage,
    accounts: Arc<RwLock<HashMap<Address, Account>>>,
    height: Arc<RwLock<i64>>,
}

#[derive(Debug)]
pub struct Account {
    balance: u128,
    nonce: u64,
}

#[derive(Error, Debug)]
#[error("{log}")]
struct ValidationError {
    code: u32,
    log: String,
}

impl Shell {
    // TODO: read config
    pub fn new() -> Self {
        Self {
            chain_id: ChainId("test".to_string()),
            height: Arc::new(RwLock::new(0)),
            accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn validate_tx(&self, tx: &Transaction) -> Result<(), ValidationError> {
        let accounts = self.accounts.read();
        if let Some(expiration) = tx.header.expiration {
            // TODO: Should be block time
            if DateTimeUtc::now().0 > expiration.0 {
                return Err(ValidationError {
                    code: 3,
                    log: "transaction expired".to_string(),
                });
            }
        }

        let curr_nonce = accounts.get(&tx.from).map(|f| f.nonce).unwrap_or(0);
        if curr_nonce + 1 != tx.nonce {
            return Err(ValidationError {
                code: 7,
                log: "incorrect nonce".to_string(),
            });
        }

        match &tx.tx_payload {
            TxPayload::CreateAccount => {
                if accounts.contains_key(&tx.from) {
                    return Err(ValidationError {
                        code: 7,
                        log: format!("account {} already exists", tx.from).to_string(),
                    });
                }
            }
            TxPayload::Transfer { to, amount } => {
                if !accounts.contains_key(to) {
                    return Err(ValidationError {
                        code: 7,
                        log: format!("account {} does not exist", tx.from).to_string(),
                    });
                }

                let balance = accounts
                    .get(&tx.from)
                    .ok_or_else(|| ValidationError {
                        code: 5,
                        log: format!("account {} not found", tx.from),
                    })?
                    .balance;

                if balance < amount.get() {
                    return Err(ValidationError {
                        code: 5,
                        log: "insufficient funds".to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}
