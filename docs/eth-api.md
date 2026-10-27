# Ethereum JSON-RPC methods

BRC2.0 provides a JSON-RPC 2.0 server to interact with the indexers, and chain explorers at `localhost:18545`. `eth_*` methods are supported to provide information on blocks and transactions, while `brc20_*` methods are used for adding new transactions and blocks to run in the execution engine. The `brc20_*` methods are documented in [Indexer JSON-RPC methods](indexer-api.md).

## eth_* methods

BRC2.0 implements the [Ethereum JSON-RPC API](https://ethereum.org/en/developers/docs/apis/json-rpc/).

JSON-RPC methods work the same way as the official implementation, e.g. `eth_blockNumber` will return the latest indexed block height, `eth_getBlockByNumber` or `eth_getBlockByHash` will return an indexed block and all the indexed transactions, and `eth_getTransactionReceipt` will return the transaction receipt for given transaction, including logs and status.

`eth_call` can be used to interact with the contracts.

> [!WARNING]
> Filter methods such as `eth_newFilter`, `eth_getFilterChanges` are not supported yet, but they are planned for after release.

Execution follows Ethereum semantics except for the points listed in [Differences from Ethereum](differences-from-ethereum.md).

## debug_* methods

BRC2.0 can record traces of transactions and serve a [callTracer](https://geth.ethereum.org/docs/developers/evm-tracing/built-in-tracers#call-tracer) result via [`debug_traceTransaction`](https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-debug#debugtracetransaction) method similar to Geth.

This needs to be enabled by setting `EVM_RECORD_TRACES` environment variable to `true`.

> [!NOTE]
> Currently, only `debug_traceTransaction` method with a `callTracer` is supported.

## txpool_content method

BRC2.0 maintains a pending transaction pool for transactions that are sent out of order, and these can be retrieved using `txpool_content` method similar to Geth.
