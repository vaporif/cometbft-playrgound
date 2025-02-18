use std::{collections::HashMap, env, num::NonZeroU64};

use borsh::{BorshDeserialize, BorshSerialize};
use tendermint_abci::Application;
use tendermint_proto::abci::{
    RequestCheckTx, RequestInfo, RequestQuery, ResponseCheckTx, ResponseInfo, ResponseQuery,
};
use thiserror::Error;

#[derive(BorshSerialize, BorshDeserialize)]
enum TxPayload {
    CreateAccount,
    Transfer { to: String, amount: NonZeroU64 },
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Hash, Clone)]
struct Address([u8; tendermint::account::LENGTH]);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
struct Transaction {
    from: Address,
    tx_payload: TxPayload,
    nonce: u64,
}

#[derive(Clone, Debug)]
struct AppChain {
    //state: cnidarium::Storage,
    balances: HashMap<Address, u64>,
    nonces: HashMap<Address, u64>,
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
            balances: HashMap::new(),
            nonces: HashMap::new(),
        }
    }

    fn validate_tx(&self, tx: &Transaction) -> Result<(), ValidationError> {
        let curr_nonce = self.nonces.get(&tx.from).unwrap_or(&0);
        if curr_nonce < &tx.nonce {
            return Err(ValidationError {
                code: 7,
                log: "incorrect nonce".to_string(),
            });
        }

        match &tx.tx_payload {
            TxPayload::CreateAccount => {
                if self.balances.contains_key(&tx.from) {
                    return Err(ValidationError {
                        code: 7,
                        log: format!("account {} already exists", tx.from).to_string(),
                    });
                }
            }
            TxPayload::Transfer { to, amount } => {
                if !self.balances.contains_key(to) {
                    return Err(ValidationError {
                        code: 7,
                        log: format!("account {} already exists", tx.from).to_string(),
                    });
                }

                if !self.balances.contains_key(&tx.from) {
                    return Err(ValidationError {
                        code: 7,
                        log: format!("account {} already exists", tx.from).to_string(),
                    });
                }
                let account = self.balances.get(&tx.from).ok_or_else(|| ValidationError {
                    code: 5,
                    log: format!("insufficient funds on {}", tx.from),
                })?;
            }
        }

        Ok(())
    }
}

impl Application for AppChain {
    fn info(&self, request: RequestInfo) -> ResponseInfo {
        ResponseInfo {
            data: "ex".to_string(),
            version: request.version,
            app_version: 1,
            last_block_height: 0,
            last_block_app_hash: vec![].into(),
        }
    }

    fn check_tx(&self, request: RequestCheckTx) -> ResponseCheckTx {
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

    fn query(&self, request: RequestQuery) -> ResponseQuery {
        todo!()
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
