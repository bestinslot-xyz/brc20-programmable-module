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

## Pending raw transaction pool

EVM wallets can hand a signed transaction to the RPC server, and a Bitcoin payer can then inscribe it as a `transact` operation. The server keeps the signed transactions in a pool until they are inscribed or expire. Any BRC2.0 RPC operator can implement `eth_sendRawTransaction`, `eth_getRawTransactionByHash` and `brc20_unsentRawTransactions` the same way, so that wallets and payers work across operators.

The pool and everything in this section require `BRC20_PROG_RPC_TX_POOL=true`. When it is `false`, the default, the three pool methods return JSON-RPC error `-32601` (method not found), `eth_getTransactionCount(address, "pending")` returns the on-chain count. See [Configuration](configuration.md).

### eth_sendRawTransaction

Adds a signed raw transaction to the pool and returns its hash, `keccak256(raw)`.

```json
{"jsonrpc": "2.0", "id": 1, "method": "eth_sendRawTransaction", "params": ["0xf86c0a8082520894..."]}
```

```json
{"jsonrpc": "2.0", "id": 1, "result": "0x2f3a...c9d1"}
```

The server accepts a transaction when all of these hold:

- It is a signed legacy (type 0) transaction with the network's EIP-155 chain ID.
- Its size is at most `CALLDATA_LIMIT` (1 MiB).
- Its value is 0, because BRC2.0 has no native value transfer.
- Its nonce is in `[account nonce, account nonce + 10)`.
- The pool holds at most 4 transactions for the same sender and nonce.

An invalid transaction returns error `-32602`:

```json
{"jsonrpc": "2.0", "id": 1, "error": {"code": -32602, "message": "nonce 3 is outside the accepted range [7, 17)"}}
```

A call over the rate limit returns error `-32005`.

Sending the same raw transaction again returns the same hash and does not extend its time to live.

### eth_getRawTransactionByHash

Returns the raw hex of a pooled transaction, or `null` when the pool does not hold it.

```json
{"jsonrpc": "2.0", "id": 1, "method": "eth_getRawTransactionByHash", "params": ["0x2f3a...c9d1"]}
```

```json
{"jsonrpc": "2.0", "id": 1, "result": "0xf86c0a8082520894..."}
```

### brc20_unsentRawTransactions

Lists the pooled transactions, oldest first. It exists for sponsors that inscribe transactions for others. `ageSecs` is the age of the entry in seconds. One response carries at most 16 MiB of raw transaction bytes.

```json
{"jsonrpc": "2.0", "id": 1, "method": "brc20_unsentRawTransactions", "params": []}
```

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": [
    {
      "hash": "0x2f3a...c9d1",
      "raw": "0xf86c0a8082520894...",
      "from": "0x1111111111111111111111111111111111111111",
      "nonce": "0xa",
      "ageSecs": 12
    }
  ]
}
```

### Pool behaviour

- The pool lives in memory, and a restart empties it.
- An entry expires 60 seconds after the server receives it.
- The pool holds at most 10,000 entries and 64 MiB of raw bytes, and evicts the oldest entries first when a cap is exceeded.
- A transaction leaves the pool once `brc20_transact` indexes it.

### Pending nonce

`eth_getTransactionCount(address, "pending")` returns the larger of the on-chain count and the highest pooled nonce of the address plus one, so a wallet can queue several transactions. Other block tags return the on-chain count.

```json
{"jsonrpc": "2.0", "id": 1, "method": "eth_getTransactionCount", "params": ["0x1111111111111111111111111111111111111111", "pending"]}
```

```json
{"jsonrpc": "2.0", "id": 1, "result": "0xb"}
```

### Rate limits

Each client may make 30 calls per 60 seconds to `eth_sendRawTransaction`, counted separately from 30 calls per 60 seconds to `brc20_unsentRawTransactions`. The admin listener has no rate limit. Sponsors and other readers of `brc20_unsentRawTransactions` should use the admin listener, because with the default settings every caller of the public listener shares one window. See [Rate limits](configuration.md#rate-limits) and [Admin RPC server](configuration.md#admin-rpc-server).

### Blocks without baseFeePerGas

Blocks returned by `eth_getBlockByNumber` and `eth_getBlockByHash` have no `baseFeePerGas` field. Wallets such as MetaMask then see no EIP-1559 support and sign legacy transactions, the only kind the engine decodes. Only the JSON response changes; stored data and block hashes are unaffected.
