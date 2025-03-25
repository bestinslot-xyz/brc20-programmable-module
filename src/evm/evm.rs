use revm::context::{BlockEnv, CfgEnv, ContextSetters, ContextTr, Evm, EvmData, TxEnv};
use revm::context_interface::block::BlobExcessGasAndPrice;
use revm::handler::instructions::{EthInstructions, InstructionProvider};
use revm::handler::EvmTr;
use revm::inspector::{inspect_instructions, InspectorEvmTr, JournalExt, NoOpInspector};
use revm::interpreter::interpreter::EthInterpreter;
use revm::interpreter::{Interpreter, InterpreterTypes};
use revm::primitives::hardfork::SpecId;
use revm::primitives::{B256, U256};
use revm::{Context, Inspector, Journal, JournalEntry};

use crate::db::DB;
use crate::evm::precompiles::BRC20Precompiles;

const CURRENT_SPEC: SpecId = SpecId::CANCUN;

pub fn get_evm(
    block_info: BlockEnv,
    db: DB,
    gas_limit: Option<u64>,
) -> BRC20Evm<Context<BlockEnv, TxEnv, CfgEnv, DB>, NoOpInspector> {
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

    BRC20Evm::new(ctx, NoOpInspector)
}

/// MyEvm variant of the EVM.
pub struct BRC20Evm<CTX, INSP>(
    pub Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, BRC20Precompiles>,
);

impl<CTX: ContextTr, INSP> BRC20Evm<CTX, INSP> {
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            data: EvmData { ctx, inspector },
            instruction: EthInstructions::new_mainnet(),
            precompiles: BRC20Precompiles::default(),
        })
    }
}

impl<CTX: ContextTr, INSP> EvmTr for BRC20Evm<CTX, INSP>
where
    CTX: ContextTr,
{
    type Context = CTX;
    type Instructions = EthInstructions<EthInterpreter, CTX>;
    type Precompiles = BRC20Precompiles;

    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.0.data.ctx
    }

    fn ctx_ref(&self) -> &Self::Context {
        self.0.ctx_ref()
    }

    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        self.0.ctx_instructions()
    }

    fn run_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output
    {
        self.0.run_interpreter(interpreter)
    }

    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        self.0.ctx_precompiles()
    }
}

impl<CTX: ContextTr, INSP> InspectorEvmTr for BRC20Evm<CTX, INSP>
where
    CTX: ContextSetters<Journal: JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Inspector = INSP;

    fn inspector(&mut self) -> &mut Self::Inspector {
        self.0.inspector()
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        self.0.ctx_inspector()
    }

    fn run_inspect_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output
    {
        let context = &mut self.0.data.ctx;
        let instructions = &mut self.0.instruction;
        let inspector = &mut self.0.data.inspector;

        inspect_instructions(
            context,
            interpreter,
            inspector,
            instructions.instruction_table(),
        )
    }
}
