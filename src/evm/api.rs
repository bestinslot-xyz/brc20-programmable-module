use revm::context::result::{HaltReason, InvalidTransaction};
use revm::context::{ContextSetters, JournalOutput};
use revm::context_interface::result::{EVMError, ExecutionResult, ResultAndState};
use revm::context_interface::{ContextTr, Database, JournalTr};
use revm::handler::{EvmTr, Handler};
use revm::inspector::{InspectCommitEvm, InspectEvm, Inspector, InspectorHandler, JournalExt};
use revm::interpreter::interpreter::EthInterpreter;
use revm::{DatabaseCommit, ExecuteCommitEvm, ExecuteEvm};

use crate::evm::evm::BRC20Evm;
use crate::evm::handler::BRC20EvmHandler;
/// Type alias for the error type of the OpEvm.
type MyError<CTX> = EVMError<<<CTX as ContextTr>::Db as Database>::Error, InvalidTransaction>;

// Trait that allows to replay and transact the transaction.
impl<CTX, INSP> ExecuteEvm for BRC20Evm<CTX, INSP>
where
    CTX: ContextSetters<Journal: JournalTr<FinalOutput = JournalOutput>>,
{
    type Output = Result<ResultAndState, MyError<CTX>>;

    type Tx = <CTX as ContextTr>::Tx;

    type Block = <CTX as ContextTr>::Block;

    fn set_tx(&mut self, tx: Self::Tx) {
        self.0.data.ctx.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.0.data.ctx.set_block(block);
    }

    fn replay(&mut self) -> Self::Output {
        BRC20EvmHandler::default().run(self)
    }
}

// Trait allows replay_commit and transact_commit functionality.
impl<CTX, INSP> ExecuteCommitEvm for BRC20Evm<CTX, INSP>
where
    CTX: ContextSetters<Db: DatabaseCommit, Journal: JournalTr<FinalOutput = JournalOutput>>,
{
    type CommitOutput = Result<ExecutionResult<HaltReason>, MyError<CTX>>;

    fn replay_commit(&mut self) -> Self::CommitOutput {
        self.replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

// Inspection trait.
impl<CTX, INSP> InspectEvm for BRC20Evm<CTX, INSP>
where
    CTX: ContextSetters<Journal: JournalTr<FinalOutput = JournalOutput> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.0.data.inspector = inspector;
    }

    fn inspect_replay(&mut self) -> Self::Output {
        BRC20EvmHandler::default().inspect_run(self)
    }
}

// Inspect
impl<CTX, INSP> InspectCommitEvm for BRC20Evm<CTX, INSP>
where
    CTX: ContextSetters<
        Db: DatabaseCommit,
        Journal: JournalTr<FinalOutput = JournalOutput> + JournalExt,
    >,
    INSP: Inspector<CTX, EthInterpreter>,
{
    fn inspect_replay_commit(&mut self) -> Self::CommitOutput {
        self.inspect_replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}
