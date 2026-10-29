# Embedding in an indexer

An indexer written in Rust can run the module in its own process instead of calling it over JSON-RPC. `Brc20Prog::open` opens the database, and every JSON-RPC method is an async method of the `Brc20ProgApiServer` trait, with the same names and parameters as `Brc20ProgApiClient`. `start_public_rpc_server` serves the same instance to other clients, and rejects the indexer methods for every caller.

```rust
use brc20_prog::{Brc20Prog, Brc20ProgApiServer, Brc20ProgConfig};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let prog = Brc20Prog::open(Brc20ProgConfig::from_env())?;
    let rpc = prog.start_public_rpc_server().await?;

    // The indexer calls the module directly, e.g. once per Bitcoin block:
    prog.brc20_mine(1, 0).await?;
    prog.brc20_commit_to_database().await?;

    rpc.stop()?;
    Ok(())
}
```

The configuration is process-wide, so open one instance per process.

The first open after upgrading from v0.16.6 or earlier migrates the data dir into the engine store inside `Brc20Prog::open` (see [Storage](configuration.md#storage)). The open blocks until the migration has finished: 11 seconds for the 760 MB signet data dir, and minutes for a large mainnet one. Call it where blocking is acceptable, before the indexer starts its own deadlines or health checks, and not from an async task that must stay responsive.

## Supplying Bitcoin data

The Bitcoin precompiles look up transactions and block heights through a Bitcoin Core node at `BITCOIN_RPC_URL`. An indexer that already has this data can implement the `BitcoinRpc` trait and open the module with `Brc20Prog::open_with_bitcoin_rpc`, which serves the lookups in process. `BITCOIN_RPC_NETWORK` still selects the network, and `brc20_initialise` checks it against `BitcoinRpc::network`.

```rust
use std::sync::Arc;

use brc20_prog::{BitcoinRpc, Brc20Prog, Brc20ProgConfig};

fn open(source: Arc<dyn BitcoinRpc>) -> Result<Brc20Prog, Box<dyn std::error::Error>> {
    Brc20Prog::open_with_bitcoin_rpc(Brc20ProgConfig::from_env(), source)
}
```

Answers feed consensus, so they must be the same on every node for a given chain state. Return `Ok(None)` for a transaction that does not exist: the precompile call fails. Return `Err` for a lookup that cannot be answered right now. What the module does with it depends on where the lookup runs:

- During block execution (`brc20_call`, `brc20_transact` and the other methods that add a transaction to a block), the module retries it five times, one second apart, and then panics, so an outage stops the indexer rather than producing a different result.
- During a public call (`eth_call`, `eth_callMany`, `eth_estimateGas`, `eth_estimateGasMany`, `brc20_balance`), the module retries it once with no pause, and then the call fails with `Call failed: Bitcoin data unavailable`. `brc20_balance` reaches no Bitcoin precompile, so it hits this only if its contract ever calls one. A source that holds only part of the chain can return `Err` for the rest, and a public request for it neither stalls nor crashes the node.

The methods run synchronously inside EVM execution, on the thread that called the `Brc20ProgApiServer` method. An implementation backed by async code can bridge with `tokio::task::block_in_place` on a multi-threaded runtime, and must not wait on a lock that the indexer holds across that call.

`Brc20ProgConfig::from_env` reads the environment variables listed in [Configuration](configuration.md). The methods are documented in [Indexer JSON-RPC methods](indexer-api.md) and [Ethereum JSON-RPC methods](eth-api.md), and the order to call them in is described in [Indexer integration](indexer-integration.md).
