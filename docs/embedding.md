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

`Brc20ProgConfig::from_env` reads the environment variables listed in [Configuration](configuration.md). The methods are documented in [Indexer JSON-RPC methods](indexer-api.md) and [Ethereum JSON-RPC methods](eth-api.md), and the order to call them in is described in [Indexer integration](indexer-integration.md).
