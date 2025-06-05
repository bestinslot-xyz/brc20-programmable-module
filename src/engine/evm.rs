use alloy_primitives::{B256, U256};
use revm::context::{BlockEnv, CfgEnv, Evm, TxEnv};
use revm::context_interface::block::BlobExcessGasAndPrice;
use revm::handler::instructions::EthInstructions;
use revm::interpreter::interpreter::EthInterpreter;
use revm::primitives::hardfork::SpecId;
use revm::{Context, Journal, JournalEntry};
use revm_inspectors::tracing::{TracingInspector, TracingInspectorConfig};

use crate::db::Brc20ProgDatabase;
use crate::engine::precompiles::BRC20Precompiles;
use crate::global::CHAIN_ID;

const CURRENT_SPEC: SpecId = SpecId::CANCUN;

pub fn get_evm(
    block_number: u64,
    block_hash: B256,
    timestamp: u64,
    db: Brc20ProgDatabase,
    gas_limit: Option<u64>,
) -> Evm<
    Context<BlockEnv, TxEnv, CfgEnv, Brc20ProgDatabase>,
    TracingInspector,
    EthInstructions<EthInterpreter, Context<BlockEnv, TxEnv, CfgEnv, Brc20ProgDatabase>>,
    BRC20Precompiles,
> {
    let mut ctx: Context<BlockEnv, TxEnv, CfgEnv, Brc20ProgDatabase, Journal<Brc20ProgDatabase, JournalEntry>> =
        Context::new(db, CURRENT_SPEC);

    ctx.cfg.chain_id = *CHAIN_ID;
    ctx.cfg.spec = CURRENT_SPEC;
    ctx.cfg.limit_contract_code_size = Some(usize::MAX);

    ctx.block.number = block_number;
    ctx.block.gas_limit = gas_limit.unwrap_or(u64::MAX);
    ctx.block.timestamp = timestamp;
    ctx.block.basefee = 0;
    ctx.block.difficulty = U256::ZERO;
    ctx.block.prevrandao = Some(block_hash);
    ctx.block.blob_excess_gas_and_price = Some(BlobExcessGasAndPrice::new(0, false));

    ctx.tx.chain_id = Some(*CHAIN_ID);
    ctx.tx.gas_limit = u64::MAX;
    ctx.tx.gas_price = 0;
    ctx.tx.value = U256::ZERO;

    Evm::new_with_inspector(
        ctx,
        TracingInspector::new(TracingInspectorConfig::none()),
        EthInstructions::new_mainnet(),
        BRC20Precompiles::new(),
    )
}
