use borsh::BorshDeserialize;
use tendermint_abci::Application;
use tendermint_proto::abci::{ExecTxResult, ResponseCheckTx, ResponseQuery};

use crate::{
    common::ChainId,
    transaction::{Transaction, TxPayload},
};

use super::{Account, App};

impl Application for App {
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

    fn init_chain(
        &self,
        request: tendermint_proto::abci::RequestInitChain,
    ) -> tendermint_proto::abci::ResponseInitChain {
        let mut state = self.state.write();
        state.chain_id = ChainId(request.chain_id);

        tendermint_proto::abci::ResponseInitChain::default()
    }

    fn query(&self, _request: tendermint_proto::abci::RequestQuery) -> ResponseQuery {
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
        &self,
        request: tendermint_proto::abci::RequestFinalizeBlock,
    ) -> tendermint_proto::abci::ResponseFinalizeBlock {
        let mut state = self.state.write();
        state.current_height = request.height;

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
                            state.accounts.insert(
                                tx.from.clone(),
                                Account {
                                    balance: 1_000_000,
                                    nonce: 0,
                                },
                            );

                            tx_results.push(ExecTxResult {
                                code: 0,
                                log: format!("Account {} created successfully", &tx.from),
                                ..Default::default()
                            });
                        }
                        TxPayload::Transfer { to, amount } => {
                            let from = state.accounts.get_mut(&tx.from).unwrap();
                            from.balance -= amount.get();
                            from.nonce += 1;
                            state.accounts.get_mut(&to).unwrap().balance += amount.get();

                            tx_results.push(ExecTxResult {
                                code: 0,
                                log: "Transaction complete".to_string(),
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
