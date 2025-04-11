use alloy_primitives::{B256, U256};
use revm::context::{BlockEnv, CfgEnv, Evm, TxEnv};
use revm::context_interface::block::BlobExcessGasAndPrice;
use revm::handler::instructions::EthInstructions;
use revm::inspector::NoOpInspector;
use revm::interpreter::interpreter::EthInterpreter;
use revm::primitives::hardfork::SpecId;
use revm::{Context, Journal, JournalEntry};

use crate::db::DB;
use crate::evm::precompiles::BRC20Precompiles;

const CURRENT_SPEC: SpecId = SpecId::CANCUN;

pub fn get_evm(
    block_info: BlockEnv,
    db: DB,
    gas_limit: Option<u64>,
) -> Evm<
    Context<BlockEnv, TxEnv, CfgEnv, DB>,
    NoOpInspector,
    EthInstructions<EthInterpreter, Context<BlockEnv, TxEnv, CfgEnv, DB>>,
    BRC20Precompiles,
> {
    let mut ctx: Context<BlockEnv, TxEnv, CfgEnv, DB, Journal<DB, JournalEntry>> =
        Context::new(db, CURRENT_SPEC);

    ctx.cfg.chain_id = 331337;
    ctx.cfg.limit_contract_code_size = Some(usize::MAX);

    ctx.block.number = block_info.number;
    ctx.block.timestamp = block_info.timestamp;
    ctx.block.gas_limit = gas_limit.unwrap_or(u64::MAX);
    ctx.block.basefee = 0;
    ctx.block.difficulty = U256::ZERO;
    ctx.block.prevrandao = Some(B256::ZERO);
    ctx.block.blob_excess_gas_and_price = Some(BlobExcessGasAndPrice::new(0, false));

    ctx.tx.gas_limit = u64::MAX;
    ctx.tx.gas_price = 0;
    ctx.tx.value = U256::ZERO;

    Evm::new_with_inspector(
        ctx,
        NoOpInspector,
        EthInstructions::new_mainnet(),
        BRC20Precompiles::default(),
    )
}
