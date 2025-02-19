use std::collections::HashMap;

use crate::common::{Address, ChainId};

use super::Account;

struct MerkleTree<'a, S> {
    accounts: jmt::Sha256Jmt<'a, S>,
}

#[derive(Debug)]
pub struct State {
    pub chain_id: ChainId,
    // TODO: switch to merkle
    //state: cnidarium::Storage,
    pub accounts: HashMap<Address, Account>,
    pub current_height: i64,
}
