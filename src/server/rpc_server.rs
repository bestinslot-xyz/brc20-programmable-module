use std::cmp::min;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;

use alloy::primitives::Bytes;
use hyper::Method;
use jsonrpsee::core::middleware::RpcServiceBuilder;
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::server::{BatchRequestConfig, Server, ServerConfigBuilder, ServerHandle};
use revm::primitives::TxKind;
use revm::state::Bytecode;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing::{debug, info, instrument, warn};

use crate::api::types::{select_bytes, EthCall, GetLogsFilter};
use crate::api::{Brc20ProgApiServer, INDEXER_METHODS};
use crate::brc20_controller::{
    decode_brc20_balance_result, load_brc20_balance_tx, load_brc20_burn_tx, load_brc20_mint_tx,
};
use crate::db::types::{
    AddressED, BlockResponseED, BytecodeED, LogED, TraceED, TxED, TxReceiptED, B256ED, U256ED,
};
use crate::engine::{get_evm_address_from_pkscript, BRC20ProgEngine, TxInfo};
use crate::global::{CONFIG, GAS_PER_BYTE, INVALID_ADDRESS};
use crate::server::auth::{HttpNonBlockingAuth, RpcAuthMiddleware};
use crate::server::error::{
    wrap_rpc_error, wrap_rpc_error_string, wrap_rpc_error_string_with_data,
};
use crate::types::{Base64Bytes, PrecompileData, RawBytes};
use crate::Brc20ProgConfig;

struct RpcServer {
    engine: BRC20ProgEngine,
}

impl RpcServer {
    fn parse_block_number(&self, number: &str) -> Result<u64, Box<dyn Error>> {
        if number == "latest" || number == "safe" || number == "finalized" {
            self.engine.get_latest_block_height()
        } else if number == "pending" {
            self.engine.get_next_block_height()
        } else if number == "earliest" {
            Ok(0)
        } else if number.starts_with("0x") {
            u64::from_str_radix(&number[2..], 16).map_err(|_| "Invalid block number".into())
        } else {
            number.parse().map_err(|_| "Invalid block number".into())
        }
    }

    async fn resolve_block_hash_or_number(
        &self,
        hash_or_number: &str,
    ) -> Result<u64, Box<dyn Error>> {
        match self.parse_block_number(hash_or_number) {
            Ok(block_number) => Ok(block_number),
            Err(_) => {
                let hash = serde_json::from_str::<B256ED>(hash_or_number)
                    .map_err(|_| "Invalid block hash or number")?;
                if let Ok(Some(block)) = self.engine.get_block_by_hash(hash.bytes, false) {
                    Ok(block.number.into())
                } else {
                    Err("Block not found".into())
                }
            }
        }
    }
}

fn log_call() {
    info!("rpc.request");
}

#[async_trait]
impl Brc20ProgApiServer for RpcServer {
    #[instrument(skip(self), level = "error")]
    async fn brc20_mine(&self, block_count: u64, timestamp: u64) -> RpcResult<()> {
        log_call();
        self.engine
            .mine_blocks(block_count, timestamp)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_deposit(
        &self,
        to_pkscript: String,
        ticker: String,
        amount: U256ED,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
    ) -> RpcResult<TxReceiptED> {
        log_call();

        let to_address = get_evm_address_from_pkscript(&to_pkscript).map_err(wrap_rpc_error)?;
        let block_height = self
            .engine
            .get_next_block_height()
            .map_err(wrap_rpc_error)?;

        self.engine
            .add_tx_to_block(
                timestamp,
                &load_brc20_mint_tx(ticker_as_bytes(&ticker), to_address, amount.uint),
                tx_idx,
                block_height,
                hash.bytes,
                inscription_id,
                u64::MAX,
                [0u8; 32].into(), // Dummy op_return_tx_id
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_withdraw(
        &self,
        from_pkscript: String,
        ticker: String,
        amount: U256ED,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
    ) -> RpcResult<TxReceiptED> {
        log_call();

        self.engine
            .add_tx_to_block(
                timestamp,
                &load_brc20_burn_tx(
                    ticker_as_bytes(&ticker),
                    get_evm_address_from_pkscript(&from_pkscript).map_err(wrap_rpc_error)?,
                    amount.uint,
                ),
                tx_idx,
                self.engine
                    .get_next_block_height()
                    .map_err(wrap_rpc_error)?,
                hash.bytes,
                inscription_id,
                u64::MAX,
                [0u8; 32].into(), // Dummy op_return_tx_id
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_balance(&self, pkscript: String, ticker: String) -> RpcResult<String> {
        log_call();

        self.engine
            .read_contract(
                &load_brc20_balance_tx(
                    ticker_as_bytes(&ticker),
                    get_evm_address_from_pkscript(&pkscript).map_err(wrap_rpc_error)?,
                ),
                None,
                None,
            )
            .map(|receipt| {
                format!(
                    "0x{:x}",
                    decode_brc20_balance_result(receipt.output.as_ref())
                )
            })
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_initialise(
        &self,
        genesis_hash: B256ED,
        genesis_timestamp: u64,
        genesis_height: u64,
    ) -> RpcResult<()> {
        log_call();
        self.engine
            .initialise(genesis_hash.bytes, genesis_timestamp, genesis_height)
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "brc20_getTxReceiptByInscriptionId",
        skip(self),
        level = "error"
    )]
    async fn brc20_get_tx_receipt_by_inscription_id(
        &self,
        inscription_id: String,
    ) -> RpcResult<Option<TxReceiptED>> {
        log_call();
        self.engine
            .get_transaction_receipt_by_inscription_id(inscription_id)
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "brc20_getInscriptionIdByContractAddress",
        skip(self),
        level = "error"
    )]
    async fn brc20_get_inscription_id_by_contract_address(
        &self,
        contract_address: AddressED,
    ) -> RpcResult<Option<String>> {
        log_call();
        self.engine
            .get_inscription_id_by_contract_address(contract_address.address)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_get_inscription_id_by_tx_hash(
        &self,
        transaction: B256ED,
    ) -> RpcResult<Option<String>> {
        log_call();
        self.engine
            .get_transaction_by_hash(transaction.bytes)
            .map(|tx| tx.and_then(|tx| tx.inscription_id))
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self, data), level = "error")]
    async fn brc20_deploy(
        &self,
        from_pkscript: String,
        data: Option<RawBytes>,
        base64_data: Option<Base64Bytes>,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
        inscription_byte_len: u64,
        op_return_tx_id: B256ED,
    ) -> RpcResult<TxReceiptED> {
        log_call();

        let block_height = self
            .engine
            .get_next_block_height()
            .map_err(wrap_rpc_error)?;

        let data = select_bytes(&data, &base64_data)
            .map_err(wrap_rpc_error)?
            .unwrap_or_default();

        self.engine
            .add_tx_to_block(
                timestamp,
                &TxInfo::from_inscription(
                    get_evm_address_from_pkscript(&from_pkscript).map_err(wrap_rpc_error)?,
                    if data.is_empty() {
                        TxKind::Call(*INVALID_ADDRESS)
                    } else {
                        TxKind::Create
                    },
                    data,
                ),
                tx_idx,
                block_height,
                hash.bytes,
                inscription_id,
                inscription_byte_len,
                op_return_tx_id.bytes,
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self, data), level = "error")]
    async fn brc20_call(
        &self,
        from_pkscript: String,
        contract_address: Option<AddressED>,
        contract_inscription_id: Option<String>,
        data: Option<RawBytes>,
        base64_data: Option<Base64Bytes>,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
        inscription_byte_len: u64,
        op_return_tx_id: B256ED,
    ) -> RpcResult<Option<TxReceiptED>> {
        log_call();

        let block_height = self
            .engine
            .get_next_block_height()
            .map_err(wrap_rpc_error)?;

        let data = select_bytes(&data, &base64_data).map_err(wrap_rpc_error)?;

        let derived_contract_address = if data.is_none() {
            *INVALID_ADDRESS
        } else if let Some(contract_inscription_id) = contract_inscription_id {
            self.engine
                .get_contract_address_by_inscription_id(contract_inscription_id)
                .map_err(wrap_rpc_error)?
                .unwrap_or(*INVALID_ADDRESS)
        } else {
            contract_address
                .map(|x| x.address)
                .unwrap_or(*INVALID_ADDRESS)
        };

        let from_address = get_evm_address_from_pkscript(&from_pkscript).map_err(wrap_rpc_error)?;

        self.engine
            .add_tx_to_block(
                timestamp,
                &TxInfo::from_inscription(
                    from_address,
                    derived_contract_address.into(),
                    data.unwrap_or_default(),
                ),
                tx_idx,
                block_height,
                hash.bytes,
                inscription_id,
                inscription_byte_len,
                op_return_tx_id.bytes,
            )
            .map(|receipt| Some(receipt))
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_transact(
        &self,
        raw_tx_data: Option<RawBytes>,
        base64_raw_tx_data: Option<Base64Bytes>,
        timestamp: u64,
        hash: B256ED,
        tx_idx: u64,
        inscription_id: String,
        inscription_byte_len: u64,
        op_return_tx_id: B256ED,
    ) -> RpcResult<Vec<TxReceiptED>> {
        log_call();

        let block_height = self
            .engine
            .get_next_block_height()
            .map_err(wrap_rpc_error)?;

        self.engine
            .add_raw_tx_to_block(
                timestamp,
                select_bytes(&raw_tx_data, &base64_raw_tx_data)
                    .map_err(wrap_rpc_error)?
                    .map(|x| x.to_vec())
                    .unwrap_or_default(),
                tx_idx,
                block_height,
                hash.bytes,
                inscription_id,
                inscription_byte_len,
                op_return_tx_id.bytes,
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_finalise_block(
        &self,
        timestamp: u64,
        hash: B256ED,
        block_tx_count: u64,
    ) -> RpcResult<()> {
        log_call();
        let block_height = self
            .engine
            .get_next_block_height()
            .map_err(wrap_rpc_error)?;
        self.engine
            .finalise_block(timestamp, block_height, hash.bytes, block_tx_count)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_reorg(&self, latest_valid_block_number: u64) -> RpcResult<()> {
        warn!("Reorg!");
        self.engine
            .reorg(latest_valid_block_number)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_commit_to_database(&self) -> RpcResult<()> {
        log_call();
        self.engine.commit_to_db().map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn brc20_clear_caches(&self) -> RpcResult<()> {
        log_call();
        self.engine.clear_caches().map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_block_number(&self) -> RpcResult<String> {
        // Skip logs since this is a common call
        // log_call();
        Ok(format!(
            "0x{:x}",
            self.engine
                .get_latest_block_height()
                .map_err(wrap_rpc_error)?
        ))
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_get_block_by_number(
        &self,
        block: String,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED> {
        // Skip logs since this is a common call
        // log_call();
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        if let Some(block) = self
            .engine
            .get_block_by_number(block_number, is_full.unwrap_or(false))
            .map_err(wrap_rpc_error)?
        {
            Ok(block)
        } else {
            Err(wrap_rpc_error_string("Block not found"))
        }
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_get_block_by_hash(
        &self,
        block: B256ED,
        is_full: Option<bool>,
    ) -> RpcResult<BlockResponseED> {
        log_call();
        if let Some(block) = self
            .engine
            .get_block_by_hash(block.bytes, is_full.unwrap_or(false))
            .map_err(wrap_rpc_error)?
        {
            Ok(block)
        } else {
            Err(wrap_rpc_error_string("Block not found"))
        }
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_get_transaction_count(
        &self,
        account: AddressED,
        block: String,
    ) -> RpcResult<String> {
        log_call();
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        self.engine
            .get_transaction_count(account.address, block_number)
            .map(|count| format!("0x{:x}", count))
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "eth_getBlockTransactionCountByNumber",
        skip(self),
        level = "error"
    )]
    async fn eth_get_block_transaction_count_by_number(&self, block: String) -> RpcResult<String> {
        log_call();
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        self.engine
            .get_block_transaction_count_by_number(block_number)
            .map(|count| format!("0x{:x}", count))
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "eth_getBlockTransactionCountByHash",
        skip(self),
        level = "error"
    )]
    async fn eth_get_block_transaction_count_by_hash(&self, block: B256ED) -> RpcResult<String> {
        log_call();
        self.engine
            .get_block_transaction_count_by_hash(block.bytes)
            .map(|count| format!("0x{:x}", count))
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_get_logs(&self, filter: GetLogsFilter) -> RpcResult<Vec<LogED>> {
        log_call();
        let from_block = filter
            .from_block
            .clone()
            .and_then(|from| self.parse_block_number(&from).ok());
        let to_block = filter
            .to_block
            .clone()
            .and_then(|to| self.parse_block_number(&to).ok());
        Ok(self
            .engine
            .get_logs(
                from_block,
                to_block,
                filter.address.clone().map(|x| x.address),
                filter.topics_as_b256(),
            )
            .map_err(wrap_rpc_error)?)
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_call(&self, call: EthCall, block_height: Option<String>) -> RpcResult<String> {
        log_call();
        let Some(data) = call.data else {
            return Err(wrap_rpc_error_string("No data or input provided"));
        };
        let block_height = if let Some(block_height) = block_height {
            Some(
                self.parse_block_number(&block_height)
                    .map_err(wrap_rpc_error)?,
            )
        } else {
            None
        };
        let receipt = self.engine.read_contract(
            &TxInfo::from_inscription(
                call.from
                    .as_ref()
                    .map(|x| x.address)
                    .unwrap_or(*INVALID_ADDRESS),
                call.to.as_ref().map(|x| x.address).into(),
                data.value().unwrap_or_default().clone(),
            ),
            block_height,
            None,
        );
        let Ok(result) = receipt else {
            return Err(wrap_rpc_error_string_with_data(
                3,
                "Call failed",
                "0x".into(),
            ));
        };
        let data_string = result.output.unwrap_or(Bytes::new()).to_string();
        if !result.status {
            return Err(wrap_rpc_error_string_with_data(
                3,
                format!("Execution reverted: {}", result.status_string).as_str(),
                data_string,
            ));
        }
        Ok(data_string)
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_call_many(
        &self,
        calls: Vec<EthCall>,
        block_height: Option<String>,
        precompile_data: Option<PrecompileData>,
    ) -> RpcResult<Vec<String>> {
        log_call();
        let block_height = if let Some(block_height) = block_height {
            Some(
                self.parse_block_number(&block_height)
                    .map_err(wrap_rpc_error)?,
            )
        } else {
            None
        };

        let mut txinfos = Vec::with_capacity(calls.len());
        for call in &calls {
            let Some(data) = &call.data else {
                return Err(wrap_rpc_error_string("No data or input provided"));
            };
            txinfos.push(TxInfo::from_inscription(
                call.from
                    .as_ref()
                    .map(|x| x.address)
                    .unwrap_or(*INVALID_ADDRESS),
                call.to.as_ref().map(|x| x.address).into(),
                data.value().unwrap_or_default().clone(),
            ));
        }
        debug!("eth_call_many: prepared {} calls", txinfos.len());

        let receipts =
            self.engine
                .read_contract_multi(&txinfos, block_height, precompile_data, None);
        let Ok(results) = receipts else {
            return Err(wrap_rpc_error_string_with_data(
                3,
                "Call failed",
                "0x".into(),
            ));
        };

        let mut outputs = Vec::with_capacity(results.len());
        let mut result_idx = 0;
        for result in results {
            let data_string = result.output.unwrap_or(Bytes::new()).to_string();

            if !result.status {
                return Err(wrap_rpc_error_string_with_data(
                    3,
                    format!(
                        "Execution with index {} reverted: {}",
                        result_idx, result.status_string
                    )
                    .as_str(),
                    data_string,
                ));
            }
            outputs.push(data_string);
            result_idx += 1;
        }

        Ok(outputs)
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_estimate_gas(
        &self,
        call: EthCall,
        block_height: Option<String>,
    ) -> RpcResult<String> {
        log_call();
        let Some(data) = call.data else {
            return Err(wrap_rpc_error_string("No data or input provided"));
        };
        let block_height = if let Some(block_height) = block_height {
            Some(
                self.parse_block_number(&block_height)
                    .map_err(wrap_rpc_error)?,
            )
        } else {
            None
        };

        let tx_info = TxInfo::from_inscription(
            call.from
                .as_ref()
                .map(|x| x.address)
                .unwrap_or(*INVALID_ADDRESS),
            call.to.as_ref().map(|x| x.address).into(),
            data.value().unwrap_or_default().clone(),
        );

        let Ok(result) = self.engine.read_contract(&tx_info, block_height, None) else {
            return Err(wrap_rpc_error_string_with_data(
                3,
                "Call failed",
                "0x".into(),
            ));
        };

        if !result.status {
            let data_string = result.output.unwrap_or(Bytes::new()).to_string();
            return Err(wrap_rpc_error_string_with_data(
                3,
                format!("Execution reverted: {}", result.status_string).as_str(),
                data_string,
            ));
        }

        let mut upper_gas_limit = CONFIG.read().evm_call_gas_limit;
        let mut lower_gas_limit = 21_000u64;
        let mut estimated_gas;

        while lower_gas_limit + GAS_PER_BYTE < upper_gas_limit {
            estimated_gas = (lower_gas_limit + upper_gas_limit) / 2;
            let receipt = self
                .engine
                .read_contract(&tx_info, block_height, Some(estimated_gas));
            let Ok(result) = receipt else {
                lower_gas_limit = estimated_gas + 1;
                debug!("eth_estimate_gas: estimated gas too low: {}", estimated_gas);
                continue;
            };
            if result.status {
                upper_gas_limit = estimated_gas;
                debug!(
                    "eth_estimate_gas: estimated gas sufficient: {}, used: {}",
                    estimated_gas, result.gas_used
                );
            } else {
                lower_gas_limit = estimated_gas + 1;
                debug!("eth_estimate_gas: estimated gas too low: {}", estimated_gas);
            }
        }
        estimated_gas = upper_gas_limit;

        let receipt = self.engine.read_contract(
            &tx_info,
            block_height,
            Some(estimated_gas),
        );

        let Ok(result) = receipt else {
            return Err(wrap_rpc_error_string_with_data(
                3,
                "Call failed",
                "0x".into(),
            ));
        };

        let data_string = result.output.unwrap_or(Bytes::new()).to_string();
        if !result.status {
            return Err(wrap_rpc_error_string_with_data(
                3,
                format!("Execution reverted: {}", result.status_string).as_str(),
                data_string,
            ));
        }
        Ok(format!("0x{:x}", estimated_gas))
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_estimate_gas_many(
        &self,
        calls: Vec<EthCall>,
        block_height: Option<String>,
        precompile_data: Option<PrecompileData>,
    ) -> RpcResult<Vec<String>> {
        log_call();
        let block_height = if let Some(block_height) = block_height {
            Some(
                self.parse_block_number(&block_height)
                    .map_err(wrap_rpc_error)?,
            )
        } else {
            None
        };

        let mut txinfos = Vec::with_capacity(calls.len());
        for call in &calls {
            let Some(data) = &call.data else {
                return Err(wrap_rpc_error_string("No data or input provided"));
            };
            txinfos.push(TxInfo::from_inscription(
                call.from
                    .as_ref()
                    .map(|x| x.address)
                    .unwrap_or(*INVALID_ADDRESS),
                call.to.as_ref().map(|x| x.address).into(),
                data.value().unwrap_or_default().clone(),
            ));
        }

        // First run to check if all calls can succeed with the max gas limit, to avoid
        // unnecessary bisection for all calls.
        let Ok(results) =
            self.engine
                .read_contract_multi(&txinfos, block_height, precompile_data.clone(), None)
        else {
            return Err(wrap_rpc_error_string_with_data(
                3,
                "Call failed",
                "0x".into(),
            ));
        };

        for (result_idx, result) in results.iter().enumerate() {
            if !result.status {
                return Err(wrap_rpc_error_string_with_data(
                    3,
                    format!(
                        "Execution with index {} reverted: {}",
                        result_idx, result.status_string
                    )
                    .as_str(),
                    result.output.as_ref().unwrap_or(&Bytes::new()).to_string(),
                ));
            }
        }

        let mut estimated_gases: Vec<u64> = vec![CONFIG.read().evm_call_gas_limit; txinfos.len()];
        for i in 0..txinfos.len() {
            let mut upper_gas_limit = CONFIG.read().evm_call_gas_limit;
            let mut lower_gas_limit = 21_000u64;

            while lower_gas_limit + GAS_PER_BYTE < upper_gas_limit {
                estimated_gases[i] = (lower_gas_limit + upper_gas_limit) / 2;

                let receipts = self.engine.read_contract_multi(
                    &txinfos,
                    block_height,
                    precompile_data.clone(),
                    Some(estimated_gases.as_ref()),
                );
                let Ok(result) = receipts else {
                    lower_gas_limit = estimated_gases[i] + 1;
                    debug!(
                        "eth_estimate_gas_many {}: estimated gas too low: {}",
                        i, estimated_gases[i]
                    );
                    continue;
                };
                if result[i].status {
                    upper_gas_limit = estimated_gases[i];
                    debug!(
                        "eth_estimate_gas_many {}: estimated gas sufficient: {}, used: {}",
                        i, estimated_gases[i], result[i].gas_used
                    );
                } else {
                    lower_gas_limit = estimated_gases[i] + 1;
                    debug!(
                        "eth_estimate_gas_many {}: estimated gas too low: {}",
                        i, estimated_gases[i]
                    );
                }
            }
            estimated_gases[i] = upper_gas_limit;
        }

        let receipts = self.engine.read_contract_multi(
            &txinfos,
            block_height,
            precompile_data,
            Some(estimated_gases.as_ref()),
        );
        let Ok(results) = receipts else {
            return Err(wrap_rpc_error_string_with_data(
                3,
                "Call failed",
                "0x".into(),
            ));
        };

        let mut result_idx = 0;
        for result in results {
            let data_string = result.output.unwrap_or(Bytes::new()).to_string();

            if !result.status {
                return Err(wrap_rpc_error_string_with_data(
                    3,
                    format!(
                        "Execution with index {} reverted: {}",
                        result_idx, result.status_string
                    )
                    .as_str(),
                    data_string,
                ));
            }
            result_idx += 1;
        }

        let mut outputs = Vec::with_capacity(estimated_gases.len());
        for gas in estimated_gases {
            let gas_string = format!("0x{:x}", gas);
            outputs.push(gas_string);
        }

        Ok(outputs)
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_get_storage_at(&self, contract: AddressED, location: U256ED) -> RpcResult<String> {
        log_call();
        Ok(format!(
            "0x{:x}",
            self.engine
                .get_storage_at(contract.address, location.uint)
                .map_err(wrap_rpc_error)?
        ))
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_get_code(&self, contract: AddressED) -> RpcResult<BytecodeED> {
        log_call();
        if let Some(bytecode) = self
            .engine
            .get_contract_bytecode(contract.address)
            .map_err(wrap_rpc_error)?
        {
            Ok(bytecode)
        } else {
            Ok(BytecodeED::new(Bytecode::new()))
        }
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_get_transaction_receipt(
        &self,
        transaction: B256ED,
    ) -> RpcResult<Option<TxReceiptED>> {
        log_call();
        self.engine
            .get_transaction_receipt(transaction.bytes)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn debug_trace_transaction(&self, transaction: B256ED) -> RpcResult<Option<TraceED>> {
        log_call();
        self.engine
            .get_trace(transaction.bytes)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn debug_get_block_trace_string(&self, block: String) -> RpcResult<Option<String>> {
        log_call();
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        self.engine
            .get_block_trace_string(block_number)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn debug_get_block_trace_hash(&self, block: String) -> RpcResult<Option<String>> {
        log_call();
        let block_number = self.parse_block_number(&block).map_err(wrap_rpc_error)?;
        self.engine
            .get_block_trace_hash(block_number)
            .map_err(wrap_rpc_error)
    }

    #[instrument(skip(self), level = "error")]
    async fn eth_get_transaction_by_hash(&self, transaction: B256ED) -> RpcResult<Option<TxED>> {
        log_call();
        self.engine
            .get_transaction_by_hash(transaction.bytes)
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "eth_getTransactionByBlockNumberAndIndex",
        skip(self),
        level = "error"
    )]
    async fn eth_get_transaction_by_block_number_and_index(
        &self,
        block_number: u64,
        tx_idx: Option<u64>,
    ) -> RpcResult<Option<TxED>> {
        log_call();
        self.engine
            .get_transaction_by_block_number_and_index(block_number, tx_idx.unwrap_or(0))
            .map_err(wrap_rpc_error)
    }

    #[instrument(
        name = "eth_getTransactionByBlockHashAndIndex",
        skip(self),
        level = "error"
    )]
    async fn eth_get_transaction_by_block_hash_and_index(
        &self,
        block_hash: B256ED,
        tx_idx: Option<u64>,
    ) -> RpcResult<Option<TxED>> {
        log_call();
        self.engine
            .get_transaction_by_block_hash_and_index(block_hash.bytes, tx_idx.unwrap_or(0))
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "txpool_content", skip(self), level = "error")]
    async fn txpool_content(
        &self,
    ) -> RpcResult<HashMap<String, HashMap<AddressED, HashMap<u64, TxED>>>> {
        log_call();
        let mut result = HashMap::new();
        result.insert(
            "pending".to_string(),
            self.engine
                .get_all_pending_transactions()
                .map_err(wrap_rpc_error)?,
        );
        result.insert(
            "queued".to_string(),
            HashMap::new(), // BRC20Prog does not support queued transactions
        );

        Ok(result)
    }

    #[instrument(name = "txpool_content_from", skip(self), level = "error")]
    async fn txpool_content_from(
        &self,
        from: AddressED,
    ) -> RpcResult<HashMap<String, HashMap<AddressED, HashMap<u64, TxED>>>> {
        log_call();
        let mut result = HashMap::new();
        result.insert(
            "pending".to_string(),
            self.engine
                .get_pending_transactions_from(from.address)
                .map_err(wrap_rpc_error)?,
        );
        result.insert(
            "queued".to_string(),
            HashMap::new(), // BRC20Prog does not support queued transactions
        );

        Ok(result)
    }

    #[instrument(name = "debug_getRawHeader", skip(self), level = "error")]
    async fn debug_get_raw_header(
        &self,
        block_hash_or_number: String,
    ) -> RpcResult<Option<String>> {
        log_call();
        self.engine
            .get_raw_header(
                self.resolve_block_hash_or_number(&block_hash_or_number)
                    .await
                    .map_err(wrap_rpc_error)?,
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "debug_getRawBlock", skip(self), level = "error")]
    async fn debug_get_raw_block(&self, block_hash_or_number: String) -> RpcResult<Option<String>> {
        log_call();
        self.engine
            .get_raw_block(
                self.resolve_block_hash_or_number(&block_hash_or_number)
                    .await
                    .map_err(wrap_rpc_error)?,
            )
            .map_err(wrap_rpc_error)
    }

    #[instrument(name = "debug_getRawReceipts", skip(self), level = "error")]
    async fn debug_get_raw_receipts(
        &self,
        block_hash_or_number: String,
    ) -> RpcResult<Option<Vec<String>>> {
        log_call();
        self.engine
            .get_raw_receipts(
                self.resolve_block_hash_or_number(&block_hash_or_number)
                    .await
                    .map_err(wrap_rpc_error)?,
            )
            .map_err(wrap_rpc_error)
    }
}

pub async fn start_rpc_server(
    engine: BRC20ProgEngine,
    config: Brc20ProgConfig,
) -> Result<ServerHandle, Box<dyn Error>> {
    let cors = CorsLayer::new()
        // Allow `POST` when accessing the resource
        .allow_methods([Method::POST])
        // Allow requests from any origin
        .allow_origin(Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);

    let http_middleware =
        ServiceBuilder::new()
            .layer(cors)
            .layer(ValidateRequestHeaderLayer::custom(
                if !config.brc20_prog_rpc_server_enable_auth {
                    HttpNonBlockingAuth::allow()
                } else {
                    let Some(rpc_username) = config.brc20_prog_rpc_server_user else {
                        return Err(
                "RPC username environment variable is required when authentication is enabled"
                    .into(),
            );
                    };
                    let Some(rpc_password) = config.brc20_prog_rpc_server_password else {
                        return Err(
                "RPC password environment variable is required when authentication is enabled"
                    .into(),
            );
                    };
                    HttpNonBlockingAuth::new(&rpc_username, &rpc_password)
                },
            ));
    let rpc_middleware = RpcServiceBuilder::new()
        .rpc_logger(1024)
        .layer_fn(|service| RpcAuthMiddleware::new(service, &*INDEXER_METHODS));
    let module = RpcServer { engine }.into_rpc();

    let handle = Server::builder()
        .set_config(
            ServerConfigBuilder::default()
                .max_request_body_size(config.max_request_size)
                .max_response_body_size(config.max_response_size)
                .set_batch_request_config(if config.batch_request_limit == 0 {
                    BatchRequestConfig::Unlimited
                } else {
                    BatchRequestConfig::Limit(config.batch_request_limit)
                })
                .build(),
        )
        .set_http_middleware(http_middleware)
        .set_rpc_middleware(rpc_middleware)
        .build(config.brc20_prog_rpc_server_url.parse::<SocketAddr>()?)
        .await?
        .start(module);

    Ok(handle)
}

fn ticker_as_bytes(ticker: &str) -> Bytes {
    let ticker_lowercase = ticker.to_lowercase();
    Bytes::from(ticker_lowercase.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use alloy::primitives::B256;
    use tempfile::TempDir;

    use super::*;
    use crate::db::Brc20ProgDatabase;
    use crate::engine::BRC20ProgEngine;

    fn create_test_server() -> RpcServer {
        let temp_dir = TempDir::new().unwrap();
        let db = Brc20ProgDatabase::new(temp_dir.path()).unwrap();
        RpcServer {
            engine: BRC20ProgEngine::new(db),
        }
    }

    #[test]
    fn test_ticker_as_bytes() {
        assert_eq!(
            ticker_as_bytes("BRC20"),
            Bytes::from(vec![0x62, 0x72, 0x63, 0x32, 0x30])
        );
        assert_eq!(
            ticker_as_bytes("brc20"),
            Bytes::from(vec![0x62, 0x72, 0x63, 0x32, 0x30])
        );
    }

    #[tokio::test]
    async fn test_parse_block_number() {
        let server = create_test_server();

        let _ = server.brc20_initialise([1; 32].into(), 20, 0).await;

        assert_eq!(server.parse_block_number("latest").unwrap(), 0);
        assert_eq!(server.parse_block_number("safe").unwrap(), 0);
        assert_eq!(server.parse_block_number("finalized").unwrap(), 0);
        assert_eq!(server.parse_block_number("pending").unwrap(), 1);
        assert_eq!(server.parse_block_number("earliest").unwrap(), 0);
        assert_eq!(server.parse_block_number("0x1").unwrap(), 1);
        assert_eq!(server.parse_block_number("1").unwrap(), 1);
    }

    #[test]
    fn test_parse_block_number_invalid() {
        let server = create_test_server();
        assert!(server.parse_block_number("invalid").is_err());
        assert!(server.parse_block_number("0xinvalid").is_err());
    }

    #[tokio::test]
    async fn test_initialise() {
        let server = create_test_server();
        let _ = server.brc20_initialise([1; 32].into(), 20, 0).await;

        assert_eq!(server.engine.get_latest_block_height().unwrap(), 0);
        assert_eq!(server.engine.get_next_block_height().unwrap(), 1);
        assert_eq!(
            server
                .engine
                .get_block_by_number(0, false)
                .unwrap()
                .unwrap()
                .number,
            0u64.into()
        );
        assert_eq!(server.engine.get_block_by_number(1, true).unwrap(), None);
        assert_eq!(
            server
                .engine
                .get_block_by_hash(B256::from_slice(&[1; 32]), true)
                .unwrap()
                .unwrap()
                .number,
            0u64.into()
        )
    }

    #[tokio::test]
    async fn test_mine() {
        let server = create_test_server();
        let _ = server.brc20_initialise([1; 32].into(), 20, 0).await;

        assert_eq!(server.engine.get_latest_block_height().unwrap(), 0);
        assert_eq!(server.engine.get_next_block_height().unwrap(), 1);

        assert!(server.brc20_mine(1, 20).await.is_ok());
        assert_eq!(server.engine.get_latest_block_height().unwrap(), 1);
        assert_eq!(server.engine.get_next_block_height().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_invalid_tx() {
        let server = create_test_server();

        let result = server
            .brc20_call(
                "deadbeef".to_string(),
                None,
                None,
                None,
                Base64Bytes::empty().into(),
                20,
                [1; 32].into(),
                0,
                "inscription_id".to_string(),
                1000,
                [2; 32].into(),
            )
            .await
            .unwrap();

        assert_eq!(result.unwrap().to.unwrap().address, *INVALID_ADDRESS);
    }
}
