use std::{collections::HashMap, env, num::NonZeroU128, sync::Arc};

use borsh::{BorshDeserialize, BorshSerialize};
use parking_lot::RwLock;
use tendermint_abci::Application;
use tendermint_proto::abci::{ExecTxResult, ResponseCheckTx, ResponseQuery};
use thiserror::Error;

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Hash, Clone)]
struct Address([u8; tendermint::account::LENGTH]);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
struct Transaction {
    from: Address,
    tx_payload: TxPayload,
    nonce: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
enum TxPayload {
    CreateAccount,
    Transfer { to: Address, amount: NonZeroU128 },
}

#[derive(Clone, Debug)]
struct AppChain {
    // TODO: arc rwlocks should be removed in favor of proper storage
    //state: cnidarium::Storage,
    balances: Arc<RwLock<HashMap<Address, u128>>>,
    nonces: Arc<RwLock<HashMap<Address, u64>>>,
    height: Arc<RwLock<i64>>,
}

#[derive(Error, Debug)]
#[error("{log}")]
struct ValidationError {
    code: u32,
    log: String,
}

impl AppChain {
    fn new() -> Self {
        Self {
            height: Arc::new(RwLock::new(0)),
            balances: Arc::new(RwLock::new(HashMap::new())),
            nonces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn validate_tx(&self, tx: &Transaction) -> Result<(), ValidationError> {
        let nonces = self.nonces.read();
        let balances = self.balances.read();
        let curr_nonce = nonces.get(&tx.from).unwrap_or(&0);
        if curr_nonce < &tx.nonce {
            return Err(ValidationError {
                code: 7,
                log: "incorrect nonce".to_string(),
            });
        }

        match &tx.tx_payload {
            TxPayload::CreateAccount => {
                if balances.contains_key(&tx.from) {
                    return Err(ValidationError {
                        code: 7,
                        log: format!("account {} already exists", tx.from).to_string(),
                    });
                }
            }
            TxPayload::Transfer { to, amount } => {
                if !balances.contains_key(to) {
                    return Err(ValidationError {
                        code: 7,
                        log: format!("account {} does not exist", tx.from).to_string(),
                    });
                }

                let balance = balances.get(&tx.from).ok_or_else(|| ValidationError {
                    code: 5,
                    log: format!("account {} not found", tx.from),
                })?;

                if *balance < amount.get() {
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

impl Application for AppChain {
    fn info(
        &self,
        request: tendermint_proto::abci::RequestInfo,
    ) -> tendermint_proto::abci::ResponseInfo {
        tendermint_proto::abci::ResponseInfo {
            data: "ex".to_string(),
            version: request.version,
            app_version: 1,
            last_block_height: 0,
            last_block_app_hash: vec![].into(),
        }
    }

    fn query(&self, request: tendermint_proto::abci::RequestQuery) -> ResponseQuery {
        todo!()
    }

    fn check_tx(&self, request: tendermint_proto::abci::RequestCheckTx) -> ResponseCheckTx {
        match Transaction::try_from_slice(&request.tx) {
            Ok(tx) => {
                if let Err(val_error) = self.validate_tx(&tx) {
                    return ResponseCheckTx {
                        code: val_error.code,
                        log: val_error.log,
                        ..Default::default()
                    };
                }

                ResponseCheckTx::default()
            }
            Err(e) => ResponseCheckTx {
                code: 1,
                log: format!("failed to parse transaction {e:?}"),
                ..Default::default()
            },
        }
    }

    fn finalize_block(
        &mut self,
        request: tendermint_proto::abci::RequestFinalizeBlock,
    ) -> tendermint_proto::abci::ResponseFinalizeBlock {
        let mut height = self.height.write();
        let mut balances = self.balances.write();
        let mut nonces = self.nonces.write();
        *height = request.height;

        let mut tx_results = Vec::new();

        for tx_bytes in request.txs {
            match Transaction::try_from_slice(&tx_bytes) {
                Ok(tx) => {
                    if let Err(val_error) = self.validate_tx(&tx) {
                        tx_results.push(ExecTxResult {
                            code: val_error.code,
                            log: val_error.log,
                            ..Default::default()
                        });

                        continue;
                    }
                    match tx.tx_payload {
                        TxPayload::CreateAccount => {
                            balances.insert(tx.from.clone(), 1_000_000);

                            tx_results.push(ExecTxResult {
                                code: 0,
                                log: format!("Account {} created successfully", &tx.from),
                                ..Default::default()
                            });
                        }
                        TxPayload::Transfer { to, amount } => {
                            *balances.get_mut(&tx.from).unwrap() -= amount.get();
                            *balances.get_mut(&to).unwrap() += amount.get();

                            tx_results.push(ExecTxResult {
                                code: 0,
                                log: format!("Transaction complete"),
                                ..Default::default()
                            });
                        }
                    }
                }
                Err(err) => {
                    tx_results.push(ExecTxResult {
                        code: 1,
                        log: format!("failed to parse tx {}", err),
                        ..Default::default()
                    });
                }
            }
        }

        tendermint_proto::abci::ResponseFinalizeBlock {
            events: vec![],
            tx_results,
            validator_updates: vec![],
            consensus_param_updates: None,
            app_hash: vec![0u8].into(),
        }
    }
}

fn main() -> eyre::Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    env::set_var("RUST_LOG", "trace");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    tracing::info!("starting");

    let app = AppChain::new();

    let app = tendermint_abci::ServerBuilder::default().bind("127.0.0.1:26657", app)?;
    tracing::info!("listenening for abci events");
    _ = app.listen();
    Ok(())
}
