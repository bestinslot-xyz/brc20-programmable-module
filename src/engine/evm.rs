use alloy::primitives::{B256, U256};
use revm::context::{BlockEnv, CfgEnv, Evm, TxEnv};
use revm::context_interface::block::BlobExcessGasAndPrice;
use revm::handler::instructions::EthInstructions;
use revm::handler::EthFrame;
use revm::interpreter::interpreter::EthInterpreter;
use revm::{Context, Journal, JournalEntry};
use revm_inspectors::tracing::{TracingInspector, TracingInspectorConfig};

use crate::db::Brc20ProgDatabase;
use crate::engine::hardforks::get_evm_spec;
use crate::engine::precompiles::BRC20Precompiles;
use crate::global::CONFIG;

pub fn get_evm(
    block_number: u64,
    block_hash: B256,
    timestamp: u64,
    db: Brc20ProgDatabase,
    gas_limit: Option<u64>,
    current_op_return_tx_id: B256,
) -> Evm<
    Context<BlockEnv, TxEnv, CfgEnv, Brc20ProgDatabase>,
    TracingInspector,
    EthInstructions<EthInterpreter, Context<BlockEnv, TxEnv, CfgEnv, Brc20ProgDatabase>>,
    BRC20Precompiles,
    EthFrame<EthInterpreter>,
> {
    let evm_spec = get_evm_spec(block_number);
    let mut ctx: Context<
        BlockEnv,
        TxEnv,
        CfgEnv,
        Brc20ProgDatabase,
        Journal<Brc20ProgDatabase, JournalEntry>,
    > = Context::new(db, evm_spec);

    ctx.cfg.chain_id = CONFIG.read().chain_id.into();
    ctx.cfg.spec = evm_spec;
    ctx.cfg.limit_contract_code_size = Some(usize::MAX);

    ctx.block.number = U256::from(block_number);
    ctx.block.gas_limit = gas_limit.unwrap_or(u64::MAX);
    ctx.block.timestamp = U256::from(timestamp);
    ctx.block.basefee = 0;
    ctx.block.difficulty = U256::ZERO;
    ctx.block.prevrandao = Some(block_hash);
    ctx.block.blob_excess_gas_and_price = Some(BlobExcessGasAndPrice::new(0, 1));

    ctx.tx.chain_id = Some(CONFIG.read().chain_id);
    ctx.tx.gas_limit = u64::MAX;
    ctx.tx.gas_price = 0;
    ctx.tx.value = U256::ZERO;

    Evm::new_with_inspector(
        ctx,
        TracingInspector::new(TracingInspectorConfig::none()),
        EthInstructions::new_mainnet(),
        BRC20Precompiles::new(evm_spec.into(), current_op_return_tx_id),
    )
}
