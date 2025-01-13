use revm::primitives::alloy_primitives::Bytes;
use revm::primitives::env::{BlobExcessGasAndPrice, BlockEnv, Env, TransactTo};
use revm::primitives::specification::SpecId;
use revm::primitives::{Address, B256, U256};
use revm::Evm;

use std::sync::Arc;

use db::DB;

mod precompiles;
use precompiles::load_precompiles;

pub fn get_evm(block_info: &BlockEnv, db: DB) -> Evm<(), DB> {
    let mut env = Env::default();
    env.cfg.chain_id = 331337;
    env.cfg.limit_contract_code_size = Some(usize::MAX);

    env.block.number = block_info.number;
    env.block.coinbase = block_info.coinbase;
    env.block.timestamp = block_info.timestamp;
    env.block.gas_limit = U256::MAX;
    env.block.basefee = U256::ZERO;
    env.block.difficulty = U256::ZERO;
    env.block.prevrandao = Some(B256::ZERO);
    env.block.blob_excess_gas_and_price = Some(BlobExcessGasAndPrice::new(0));

    env.tx.gas_limit = u64::MAX;
    env.tx.gas_price = U256::ZERO;
    env.tx.value = U256::ZERO;

    let mut evm = Evm::builder()
        .with_db(db)
        .with_env(Box::new(env))
        .with_spec_id(SpecId::SHANGHAI) // NOTE: also change load_precompiles while changing this
        .build();

    evm.handler.pre_execution.load_precompiles = Arc::new(load_precompiles);

    evm
}

pub fn modify_evm_with_tx_env(
    evm: Evm<(), DB>,
    caller: Address,
    transact_to: TransactTo,
    data: Bytes,
) -> Evm<(), DB> {
    evm.modify()
        .modify_tx_env(|tx_env| {
            tx_env.caller = caller;
            tx_env.transact_to = transact_to;
            tx_env.data = data;
        })
        .build()
}
