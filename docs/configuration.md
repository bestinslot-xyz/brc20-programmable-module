# Configuration

`brc20-prog` reads its configuration from environment variables. The binary also loads a `.env` file from the working directory. [`env.sample`](../env.sample) is a sample environment for Signet.

Precompiled contracts require the Bitcoin RPC variables to work properly, otherwise they fail. See [Precompiles](precompiles.md).

## Environment variables

| Variable | Default | Description |
| -------- | ------- | ----------- |
| `BRC20_PROG_DB_PATH` | `target/db` | Path to the database folder. |
| `BRC20_PROG_RPC_SERVER_URL` | `127.0.0.1:18545` | Address of the JSON-RPC server. |
| `BRC20_PROG_ADMIN_RPC_SERVER_URL` | unset | Address of a separate listener for the indexer methods. See [Admin RPC server](#admin-rpc-server). |
| `BRC20_PROG_RPC_SERVER_ENABLE_AUTH` | `false` | Enables HTTP basic auth. See [Authorization](#authorization). |
| `BRC20_PROG_RPC_SERVER_USER` | unset | Username for HTTP basic auth. Required when auth is enabled. |
| `BRC20_PROG_RPC_SERVER_PASSWORD` | unset | Password for HTTP basic auth. Required when auth is enabled. |
| `EVM_RECORD_TRACES` | `false` | Records transaction traces for `debug_traceTransaction`. |
| `EVM_CALL_GAS_LIMIT` | `1000000000` | Gas limit for `eth_call` and `eth_estimateGas`. |
| `BITCOIN_RPC_URL` | `http://localhost:38332` | Bitcoin RPC endpoint used by the Bitcoin precompiles. |
| `BITCOIN_RPC_USER` | empty | Bitcoin RPC username. |
| `BITCOIN_RPC_PASSWORD` | empty | Bitcoin RPC password. |
| `BITCOIN_RPC_NETWORK` | `signet` | Bitcoin network. `bitcoin` or `mainnet` selects chain id `0x4252433230` ("BRC20"); every other value selects `0x425243323073` ("BRC20s"). The network also selects the activation heights in [Differences from Ethereum](differences-from-ethereum.md#hardforks-and-activation-heights). |
| `FAIL_ON_BITCOIN_RPC_ERROR` | `true` | Stops the server when the Bitcoin RPC server is unreachable when needed. |
| `MAX_REQUEST_SIZE` | `10485760` (10 MB) | Maximum JSON-RPC request size in bytes. |
| `MAX_RESPONSE_SIZE` | `104857600` (100 MB) | Maximum JSON-RPC response size in bytes. |
| `BATCH_REQUEST_LIMIT` | `50` | Maximum requests in a JSON-RPC batch, `0` for unlimited. |
| `GET_LOGS_MAX_BLOCKS` | `6` | Maximum number of blocks a single `eth_getLogs` call may span. |

Boolean variables are enabled by the value `true`. A numeric variable that fails to parse falls back to its default.

`Brc20Prog::open`, which the server runs at startup, rejects a configuration where auth is enabled without both a username and a password, where `BRC20_PROG_RPC_SERVER_URL` is empty, or where `BRC20_PROG_ADMIN_RPC_SERVER_URL` is not a socket address or equals `BRC20_PROG_RPC_SERVER_URL`. With `FAIL_ON_BITCOIN_RPC_ERROR=true`, it also rejects an empty `BITCOIN_RPC_URL` or `BITCOIN_RPC_NETWORK`.

## Command-line options

| Option | Description |
| ------ | ----------- |
| `-l`, `--log-level <level>` | Log level (default: `WARN`). |
| `-f`, `--log-file <file>` | Log file. |
| `-h`, `--help` | Prints usage. |

## Authorization

`brc20_prog` module supports basic username/password HTTP auth. It's turned off by default, but can be enabled using the following environment variables:

```bash
BRC20_PROG_RPC_SERVER_ENABLE_AUTH=true
BRC20_PROG_RPC_SERVER_USER="<USER>"
BRC20_PROG_RPC_SERVER_PASSWORD="<PASSWORD>"
```

> [!CAUTION]
> This uses basic auth, so make sure authenticated calls are made through HTTPS or a secure tunnel if the server is exposed to the internet, otherwise credentials can be intercepted.

## Admin RPC server

The indexer methods (`brc20_mine`, `brc20_deploy`, `brc20_call`, `brc20_transact`, `brc20_deposit`, `brc20_withdraw`, `brc20_initialise`, `brc20_finaliseBlock`, `brc20_reorg`, `brc20_commitToDatabase`, `brc20_clearCaches`, `debug_getBlockTraceString`, `debug_getBlockTraceHash`) can be served on a separate listener:

```bash
BRC20_PROG_ADMIN_RPC_SERVER_URL=127.0.0.1:18546
```

When it is set:

- `BRC20_PROG_RPC_SERVER_URL` serves every other method, and rejects the indexer methods for every caller, including callers with valid credentials.
- `BRC20_PROG_ADMIN_RPC_SERVER_URL` serves every method. Authentication settings apply to it as above, as a second layer.

Access to the admin listener is controlled by where it binds and who can reach that address, so keep it off the public internet:

- **Bare metal:** bind it to `127.0.0.1` and block the port in the firewall.
- **Docker, indexer in another container:** bind it to `0.0.0.0:18546` inside the container and don't publish the port. Containers on the same Docker network can reach it.
- **Docker, indexer on the host:** publish it to the host loopback only, e.g. `127.0.0.1:18546:18546`.
- **Reverse proxy:** proxy only the public port, never the admin port.

The server logs a warning at startup when the admin listener binds to a non-loopback address. When `BRC20_PROG_ADMIN_RPC_SERVER_URL` is unset, a single listener serves every method and authentication gates the indexer methods.
