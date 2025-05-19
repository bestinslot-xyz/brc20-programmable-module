<h1 align="center">BRC2.0 - Programmable Module</h1>
<p align="center">Smart contract execution engine compatible with BRC20 standard.</p>
<div align="center">

[![BRC2.0](https://github.com/bestinslot-xyz/brc20-programmable-module/actions/workflows/rust.yml/badge.svg)](https://github.com/bestinslot-xyz/brc20-programmable-module/actions)
[![Discord](https://dcbadge.vercel.app/api/server/6G8yPAcP3Z?style=flat)](https://discord.com/invite/6G8yPAcP3Z)

</div>

BRC2.0 programmable module provides smart contract execution capabilities for BRC20 indexers.

This module allows users to inscribe smart contracts and function calls on Bitcoin blockchain to implement decentralised applications.

BRC2.0 runs on a custom `EVM` execution engine using [`revm`](https://github.com/bluealloy/revm). Our main reasons for choosing `EVM` are listed below:

- Rich open-source ecosystem for tooling, including several different execution engines
- Heavily tested open-source smart contract libraries that are readily available for various financial applications
- Large and active developer community - many smart contract developers are already familiar with `EVM` and `Solidity`
- `EVM` is deterministic and Turing complete.

See our proposal at [bestinslot-xyz/brc20-prog-module-proposal](https://github.com/bestinslot-xyz/brc20-prog-module-proposal) for detailed information about how the BRC2.0 programmable module works.

See [Indexer Integration Guide](#indexer-integration-guide) on how to integrate the programmable module your BRC20 indexer.

For questions, comments and requests, use [the issues section](https://github.com/bestinslot-xyz/brc20-programmable-module/issues) or [Best in Slot discord server](https://discord.com/invite/6G8yPAcP3Z).

> [!WARNING]
> This module is not currently enabled on Bitcoin mainnet.

## Usage

BRC2.0 Programmable Module is written in [Rust](https://www.rust-lang.org/), so you need Cargo installed in order to build and run the server.

Precompiled contracts require environment variables to work properly, see the [Precompiles](#precompiles) section and [Indexer Integration Guide](#indexer-integration-guide) to learn how to set them up, otherwise precompiled contracts will fail.

**Build and run brc20_prog:**

```
cargo run --release
```

> [!NOTE]
> You must use clang as CC. Try installing clang `sudo apt install clang` before running `brc20_prog`.
> 
> Eg. `CC=/usr/bin/clang CXX=/usr/bin/clang++`. Clang llvm version must be the same as the one used by rust compiler. On the rust side you should use `RUSTFLAGS="-Clinker-plugin-lto -Clinker=clang -Clink-arg=-fuse-ld=lld"`.

## Supported JSON-RPC methods

BRC2.0 provides a JSON-RPC 2.0 server to interact with the indexers, and chain explorers at `localhost:18545`. `eth_*` methods are supported to provide information on blocks and transactions, while `brc20_*` methods are used for adding new transactions and blocks to run in the execution engine.

### eth_* methods

BRC2.0 implements the [Ethereum JSON-RPC API](https://ethereum.org/en/developers/docs/apis/json-rpc/).

JSON-RPC methods work the same way as the official implementation, e.g. `eth_blockNumber` will return the latest indexed block height, `eth_getBlockByNumber` or `eth_getBlockByHash` will return an indexed block and all the indexed transactions, and `eth_getTransactionReceipt` will return the transaction receipt for given transaction, including logs and status.

`eth_call` can be used to interact with the contracts.

> [!WARNING]
> Filter methods such as `eth_newFilter`, `eth_getFilterChanges` are not supported yet, but they are planned for after release.

### debug_* methods

BRC2.0 can record traces of transactions and serve a [callTracer](https://geth.ethereum.org/docs/developers/evm-tracing/built-in-tracers#call-tracer) result via [`debug_traceTransaction`](https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-debug#debugtracetransaction) method similar to Geth.

This needs to be enabled by setting `EVM_RECORD_TRACES` environment variable to `true`.

> [!NOTE]
> Currently, only `debug_traceTransaction` method with a `callTracer` is supported.

### brc20_* methods (for indexers)

BRC2.0 implements following `brc20_*` JSON-RPC methods intended for indexer usage

#### Mine empty blocks

**Method**: `brc20_mine`

**Description**: Inserts empty blocks with unknown/unimportant hashes, this method can be used to speed up the initialisation process by skipping unnecessary blocks and moving the block height to given point for indexing purposes.

**Parameters**:

- block_count (`int`): Number of empty blocks to insert
- timestamp (`int`): Timestamp for the empty blocks

<hr>

#### Initialise and deploy BRC20_Controller contract

**Method**: `brc20_initialise`

**Description**: Initialises the execution engine with a known block height and hash, deploys the `BRC20_Controller` contract at address `0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb`. This method can be called before or after `brc20_mine`, but subsequent calls to it must have the same genesis parameters, otherwise it will fail.

**Parameters**:

- genesis_hash (`string`): Block hash
- genesis_timestamp (`int`): Timestamp
- genesis_height (`int`): Block height

**Returns**:

- Error if block info doesn't match a previous `brc20_initialise` call.

<hr>

#### Deploy contract

**Method**: `brc20_deploy`

**Description**: Used to deploy a contract, this adds a transaction to current block.

**Parameters**:

- from_pkscript (`string`): Bitcoin pkscript that created the deploy/call inscription
- data (`string`): Call or deploy data for EVM
- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash
- tx_idx (`int`): Transaction index, starts from 0 every block, and needs to be incremented for every transaction
- inscription_id (Optional `string`): Source inscription ID that triggered this transaction, will be recorded for easier contract address retrieval
- inscription_byte_len (Optional `number`): Length of the insription content, used to determine the gas limit for this transaction

**Returns**:

- Receipt for the executed transaction, see [eth_getTransactionReceipt](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt) for details.

<hr>

#### Call contract

**Method**: `brc20_call`

**Description**: Used to call a contract, this adds a transaction to current block.

**Parameters**:

- from_pkscript (`string`): Bitcoin pkscript that created the deploy/call inscription
- contract_address (`string`): Address of the contract to call, corresponds to the "c" (Contract Address) field of a call inscription
- contract_inscription_id (`string`): Contract deployed by the inscription ID to call, corresponds to the "i" (Inscription ID) field of a call inscription
- data (`string`): Call or deploy data for EVM, corresponds to the "d" (Data) field of a call inscription
- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash
- tx_idx (`int`): Transaction index, starts from 0 every block, and needs to be incremented for every transaction
- inscription_id (Optional `string`): Inscription ID that triggered this transaction, will be recorded for easier transaction receipt retrieval
- inscription_byte_len (Optional `number`): Length of the insription content, used to determine the gas limit for this transaction

**Returns**:

- Receipt for the executed transaction, see [eth_getTransactionReceipt](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt) for details.

> [!NOTE]
> `inscription_byte_len` parameter is used to determine the gas limit for `brc20_deploy` and `brc20_call` transactions, currently BRC2.0 sets an allowance of 12000 gas per byte (object to change, but generously set). In case of calling expensive methods and contracts, inscriptions should be padded to increase the gas allowance. Minimum gas limit is set to 32 bytes per transaction. `eth_estimateGas` JSON-RPC method can be used to estimate how much gas this transaction might consume.

<hr>

#### Get Transaction Receipt by Inscription ID

**Method**: `brc20_getTxReceiptByInscriptionId`

**Description**: Returns the transaction receipt for given inscription ID, previously sent via `brc20_deploy` or `brc20_call`. This makes it easier to work with inscriptions rather than transactions in BRC2.0 applications.

**Parameters**:

- inscription_id (`string`): Inscription ID previously added via `brc20_deploy`, `brc20_call`, `brc20_deposit`, or `brc20_withdraw`.

**Returns**:

- Transaction receipt, following `eth_getTransactionReceipt` structure.
- None if the inscription isn't added yet, i.e. it doesn't match previous calls.

<hr>

#### Get Inscription ID by Transaction Hash

**Method**: `brc20_getInscriptionIdByTxHash`

**Description**: Returns the inscription ID for given transaction, previously sent via `brc20_deploy` or `brc20_call`. This makes it easier to work with inscriptions rather than transactions in BRC2.0 applications.

**Parameters**:

- tx_hash (`string`): Transaction hash previously added via `brc20_deploy`, `brc20_call`, `brc20_deposit`, or `brc20_withdraw`.

**Returns**:

- Inscription ID, as string
- None if the transaction doesn't have an inscription

<hr>

#### Finalise Block

**Method**: `brc20_finaliseBlock`

**Description**: Finalises a block, this should be called after all the transactions in the block are added via `brc20_deploy`, `brc20_call`, `brc20_deposit`, or `brc20_withdraw`.

**Parameters**:

- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash
- block_tx_count (`int`): Number of transactions added to this block

**Returns**:

- Error if any of the `timestamp` or `hash` parameters don't match previous calls.
- Error if `block_tx_count` doesn't match transaction count for this block.

<hr>

#### Commit to Database

**Method**: `brc20_commitToDatabase`

**Description**: Writes pending changes to disk.

**Parameters**:

- None

<hr>

#### Clear Caches

**Method**: `brc20_clearCaches`

**Description**: Removes pending changes. Can be used to clear recently added transactions and revert to last saved state.

**Parameters**:

- None

<hr>

#### Reorg

**Method**: `brc20_reorg`

**Description**: Reverts to a previous state at the given block. Should be used when a reorg is detected.

**Parameters**:

- latest_valid_block_number (`int`): Block height to revert the state to

> [!NOTE]
> Not all of the history is stored, and reorg is only supported up to 10 blocks earlier (this can be modified in code if needed, but will result in increased storage), otherwise this method will fail and return an error.

<hr>

#### BRC20 Deposit

**Method**: `brc20_deposit`

**Description**: Deposits (mints) BRC20 tokens to given bitcoin pkscript. This is a convenience method to replace `brc20_call` calls for BRC20 transactions, and used to transfer BRC20 tokens into BRC2.0 module.

**Parameters**:

- to_pkscript (`string`): Bitcoin pkscript to receive BRC20 tokens
- ticker (`string`): Ticker for the BRC20 token
- amount (`string`): Amount of BRC20 tokens
- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash (starting with 0x)
- tx_idx (`int`): Transaction index
- inscription_id (`string`): Inscription ID that triggered this transaction

**Returns**:

- Receipt for the executed transaction, see [eth_getTransactionReceipt](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt) for details.

<hr>

#### BRC20 Withdraw

**Method**: `brc20_withdraw`

**Description**: Withdraws (burns) BRC20 tokens from given bitcoin pkscript. Method returns an error if the given pkscript doesn't have enough tokens. This is a convenience method to replace `brc20_call` calls for BRC20 transactions, and used to transfer BRC20 tokens out of BRC2.0 module.

**Parameters**:

- from_pkscript (`string`): Bitcoin pkscript to burn BRC20 tokens
- ticker (`string`): Ticker for the BRC20 token
- amount (`string`): Amount of BRC20 tokens
- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash (starting with 0x)
- tx_idx (`int`): Transaction index
- inscription_id (`string`): Inscription ID that triggered this transaction

**Returns**:

- Receipt for the executed transaction, see [eth_getTransactionReceipt](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt) for details.

<hr>

#### BRC20 Balance

**Method**: `brc20_balance`

**Description**: Returns a transaction receipt for retrieving current BRC20 balance (in-module) for the given pkscript and ticker.

**Parameters**:

- pkscript (`string`): Bitcoin pkscript
- ticker (`string`): BRC20 ticker

**Returns**:

- (string) BRC20 balance of the bitcoin pkscript for the given ticker

## Precompiles

Execution engine has precompiled contracts deployed at given addresses to make it easier to work with bitcoin transactions.

| Precompile         | Address                                    |
| ------------------ | ------------------------------------------ |
| BRC20_Balance      | 0x00000000000000000000000000000000000000ff |
| BIP322_Verifier    | 0x00000000000000000000000000000000000000fe |
| BTC_Transaction    | 0x00000000000000000000000000000000000000fd |
| BTC_LastSatLoc     | 0x00000000000000000000000000000000000000fc |
| BTC_LockedPkScript | 0x00000000000000000000000000000000000000fb |

### BRC20 Balance Contract

`BRC20_Balance` contract can be used to retrieve non-module BRC20 balance for a given pkscript. BRC2.0 makes an HTTP call to the server at `BRC20_PROG_BALANCE_SERVER_URL` environment variable.

```
> curl "http://localhost:18546/?pkscript=1234567890ABCDEF&ticker=0x12345678"
86
```

> [!NOTE]
> `ticker` parameter is hex encoded to avoid passing invalid URL strings.

BRC20 indexers should expose this HTTP server and set the environment variable accordingly.

> [!WARNING]
> BRC20 Balance Server exposed by the indexer should return BRC20 balance at the time of current transaction after processing all the BRC20 events up until this point, and NOT the BRC20 balance at the start of the block.

**Contract interface**:

```solidity
/**
 * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
 */
interface IBRC20_Balance {
    function balanceOf(
        bytes calldata ticker,
        bytes calldata pkscript
    ) external view returns (uint256);
}
```

> [!WARNING]
> `BRC20_PROG_BALANCE_SERVER_URL` must be set for this precompile to work.

### BIP322 Verifier Contract

`BIP322_Verifier` contract can be used to verify a BIP322 signature. This precompile uses the [rust-bitcoin/bip322](https://github.com/rust-bitcoin/bip322) library.

**Contract interface**:

```solidity
/**
 * @dev BIP322 verification method
 */
interface IBIP322_Verifier {
    function verify(
        bytes calldata pkscript,
        bytes calldata message,
        bytes calldata signature
    ) external returns (bool success);
}
```

> [!WARNING]
> Currently [rust-bitcoin/bip322](https://github.com/rust-bitcoin/bip322) and this precompile only supports `P2TR`, `P2WPKH` and `P2SH-P2WPKH` single-sig addresses.

### Bitcoin Contracts

BRC2.0 has a set of precompiles that make it easier to work with bitcoin transactions within a smart contract. These can be used to retrieve transaction details, track satoshis across transactions and calculate locked pkscripts. These allow BRC2.0 smart contracts to be aware of the transactions, ordinals and ordinal lockers that happen outside the execution engine.

> [!WARNING]
> `BTC_Transaction` and `BTC_LastSatLoc` precompiles use Bitcoin JSON-RPC calls to calculate results, so an RPC server needs to be specified in the environment variables.
> 
> Associated environment variables are `BITCOIN_RPC_URL`, `BITCOIN_RPC_USER`, `BITCOIN_RPC_PASSWORD` and `BITCOIN_RPC_NETWORK`. See [env.sample](env.sample) for a sample environment for Signet.

#### Transaction details

`BTC_Transaction` contract can be used to retrieve details for a bitcoin transaction. Returns block height, and `vin`, `vout` txids, scriptPubKeys and values as arrays.

**Contract interface**:

```solidity
/**
 * Get Bitcoin transaction details using tx ids.
 */
interface IBTC_Transaction {
    function getTxDetails(
        bytes32 txid
    )
        external
        view
        returns (
            uint256 block_height,
            bytes32[] memory vin_txids,
            uint256[] memory vin_vouts,
            bytes[] memory vin_scriptPubKeys,
            uint256[] memory vin_values,
            bytes[] memory vout_scriptPubKeys,
            uint256[] memory vout_values
        );
}
```

#### Last sat location

`BTC_LastSatLoc` contract can be used to retrieve previous location of a satoshi at given `txid`, `vout` and `sat` number using the rules detailed at [ordinals/ord/blob/master/bip.mediawiki](https://github.com/ordinals/ord/blob/master/bip.mediawiki).

**Contract interface**:

```solidity
/**
 * @dev Get last satoshi location of a given sat location in a transaction.
 */
interface IBTC_LastSatLoc {
    function getLastSatLocation(
        bytes32 txid,
        uint256 vout,
        uint256 sat
    ) external view returns (
        bytes32 last_txid,
        uint256 last_vout,
        uint256 last_sat,
        bytes memory old_pkscript,
        bytes memory new_pkscript
    );
}
```

#### Get locked pkscript

`BTC_LockedPkScript` contract can be used to calculate lock pkscripts for given pkscript and block count.

**Contract interface**:

```solidity
/**
 * @dev Get locked pkscript of a given Bitcoin wallet script.
 */
interface IBTC_LockedPkscript {
    function getLockedPkscript(
        bytes calldata pkscript,
        uint256 lock_block_count
    ) external view returns (bytes memory locked_pkscript);
}
```

## Indexer Integration Guide

BRC2.0 execution engine is designed to work together with a BRC20 indexer, and the indexer should recognise inscriptions that are intended for BRC2.0 and execute transactions, deposit and withdraw BRC20 tokens.

> [!NOTE]
> BRC20 indexer in [OPI/experimental-signet-brc20-prog](https://github.com/bestinslot-xyz/OPI/tree/experimental-signet-brc20-prog) branch already has the brc20-prog integration in place.

### Deploy/Call inscriptions

Defined in the [proposal](https://github.com/bestinslot-xyz/brc20-prog-module-proposal), deploy inscriptions have the following structure:

```json
{
    "p": "brc20-prog",
    "op": "deploy",
    "d": "<bytecode + constructor_args in hex>"
}
```

Whenever an indexer encounters a deploy inscription, it should inform the programmable module via the `brc20_deploy` JSON-RPC method, this will allow the EVM to deploy a new smart contract.

Once an inscription is deployed as a smart contract, then methods can be called via call inscriptions with the following structure:

```json
{
    "p": "brc20-prog",
    "op": "call",
    "c": "<contract_addr>",
    "i": "<inscription_id>",
    "d": "<call data>"
}
```

Call inscriptions should be added as transactions to the EVM using `brc20_call` JSON-RPC method. BRC2.0 maintains a map of contract addresses and deploy inscriptions, so at least one of the `"c"` or `"i"` fields should be set to call the contract `"c"`, or a contract deployed by the inscription `"i"`.

### Deposit/Withdrawal inscriptions

Deposit inscriptions are standard BRC20 transfer inscriptions that are sent to `OP_RETURN "BRC20PROG"`:

```json
{
  "p": "brc-20",
  "op": "transfer",
  "tick": "ordi",
  "amt": "10"
}
```

When an indexer encounters this, it should call `brc20_deposit` JSON-RPC method to create the same amount of BRC20 tokens in the execution engine. These BRC20 tokens then can be transferred and manipulated using BRC2.0 call inscriptions.

Withdraw inscriptions have the following structure:

```json
{
  "p": "brc20-module",
  "op": "withdraw",
  "tick": "ordi",
  "amt": "10",
  "module": "BRC20PROG"
}
```

When encountered, an indexer can call `brc20_withdraw` JSON-RPC method, and verify the result, as this can fail in case there isn't enough funds to withdraw, and increase BRC20 balance for the pkscript this inscription was sent to.

> [!WARNING]
> Tokens should be withdrawn from the sender's pkscript, but deposited to the receiver's pkscript for a withdraw inscription. A withdraw inscription can be sent to the same pkscript, or a different pkscript.

### Initialisation and empty blocks

Execution engine deploys a `BRC20_Controller` contract for BRC20 deposits, transfers and withdrawals. This deployment should be triggered by an indexer via `brc20_initialise` method at any point, before any of the inscriptions take place. This will add a block with a single transaction that is the `BRC20_Controller` deployment transaction.

In order to skip initial blocks, i.e. empty blocks, miners can call `brc20_mine` to add empty blocks to the system. If the first inscription is at block height 100, then initialisation might look like:

```
brc20_mine {
    block_count: 100,
    timestamp: 0
}
brc20_initialise {
    genesis_hash: "100TH_BLOCK_HASH",
    genesis_timestamp: "100TH_BLOCK_TIMESTAMP",
    genesis_height: 100
}
```

If an indexer wants earlier block hashes and timestamps to be correct, they can also initialise empty blocks using `brc20_finaliseBlock`, and pass the correct hashes and timestamps.

```
brc20_initialise {
    genesis_hash: "GENESIS_HASH",
    genesis_timestamp: "GENESIS_TIMESTAMP",
    genesis_height: 0
}
for all initial blocks:
    brc20_finaliseBlock {
        hash: "KNOWN_HASH",
        timestamp: "KNOWN_TIMESTAMP", block_tx_count: 0
    }
```
### Loop for adding transactions and finalising blocks

When a new block arrives, all its deploy/call/deposit/withdraw transactions should be sent to the execution engine in order, with the correct transaction index using the relevant methods such as `brc20_deploy`, `brc20_call`, `brc20_deposit`, and `brc20_withdraw`. Once all inscriptions in the block are processed, block should be finalised using the `brc20_finaliseBlock` JSON-RPC method.

Indexing for a single block in pseudo code would look like the following (field validation is omitted for simplicity):

```
block = await_new_block()
current_tx_idx = 0
for (inscription, transfer) in block:
    current_inscription_id = transfer.inscription_id
    sender = transfer.sender
    receiver = transfer.receiver

    if inscription.op in ['deploy', 'd'] and
       receiver.pkscript is OP_RETURN "BRC20PROG":
        brc20_deploy(
            from_pkscript: sender.pkscript,
            data: inscription.d,
            hash: block.hash,
            timestamp: block.timestamp,
            tx_idx: current_tx_idx++,
            inscription_id: current_inscription_id,
            inscription_byte_len: inscription.content.length)

    if inscription.op in ['call', 'c'] and
       receiver.pkscript is OP_RETURN "BRC20PROG":
        brc20_call(
            from_pkscript: sender.pkscript,
            contract_address: inscription.c
            contract_inscription_id: inscription.i,
            data: inscription.d,
            hash: block.hash,
            timestamp: block.timestamp,
            tx_idx: current_tx_idx++,
            inscription_id: current_inscription_id,
            inscription_byte_len: inscription.content.length)

    if inscription.op is 'transfer' and
       receiver.pkscript is OP_RETURN "BRC20PROG":
        if sender.balance[inscription.tick] > inscription.amt:
            sender.balance[inscription.tick] -= inscription.amt;
            brc20_deposit(
                to_pkscript: sender.pkscript,
                ticker: inscription.tick,
                amount: inscription.amt (padded to 18 decimals),
                hash: block.hash,
                timestamp: block.timestamp,
                tx_idx: current_tx_idx++,
                inscription_id: current_inscription_id)

    if inscription.op is 'withdraw' and
       inscription.p is 'brc20-module' and
       inscription.module is 'BRC20PROG':
        # Withdrawals are done from sender's pkscript
        result = brc20_withdraw(
            from_pkscript: sender.pkscript,
            ticker: inscription.tick,
            amount: inscription.amt (padded to 18 decimals),
            hash: block.hash,
            timestamp: block.timestamp,
            tx_idx: current_tx_idx++,
            inscription_id: current_inscription_id)
        # Withdrawals fail if there is not enough funds
        if result.status = '0x1':
            # Note that withdrawals are sent to receiver's wallet
            receiver.balance[inscription.tick] += inscription.amt

# Finalise block at the end
brc20_finaliseBlock(
    hash: block.hash,
    timestamp: block.timestamp,
    block_tx_count: current_tx_idx)

# Committing to database, can be done at any point to write changes to disk
brc20_commitToDatabase()
```

When a reorg is detected, `brc20_reorg` should be called to revert the EVM to a previous state.

### BRC20 Balance Server

Indexers should expose a balance server that returns current overall balance for a pkscript and a ticker, and set the `BRC20_BALANCE_SERVER_URL` environment variable to make sure the `BRC20_Balance` precompiled contract knows where to send these requests to.

```
> curl "http://localhost:18546/?pkscript=1234567890ABCDEF&ticker=0x123456789"
86
```

> [!WARNING]
> BRC20 Balance Server exposed by the indexer should return BRC20 balance at the time of current transaction after processing all the BRC20 events up until this point, and NOT the BRC20 balance at the start of the block.

### Authorization

`brc20_prog` module supports basic username/password HTTP auth. It's turned off by default, but can be enabled using the following environment variables:

```bash
BRC20_PROG_RPC_SERVER_ENABLE_AUTH=true
BRC20_PROG_RPC_SERVER_USER="<USER>"
BRC20_PROG_RPC_SERVER_PASSWORD="<PASSWORD>"
```

### Indexer Checklist

- [ ] Set environment variables, check [env.sample](env.sample) for a list
- [ ] Start a [BRC20 balance server](#brc20-balance-server) for [BRC20_Balance Contract](#brc20-balance-contract)
- [ ] Mine [`brc20_mine`](#mine-empty-blocks) or finalise empty blocks [`brc20_finaliseBlock`](#finalise-block) to fill the database before the first inscription height
- [ ] Deploy the `BRC20_Controller` contract by calling [`brc20_initialise`](#initialise-and-deploy-brc20_controller-contract)
- [ ] Index every block for BRC2.0 transactions
  - [ ] [Add deploy/call inscriptions](#deploycall-inscriptions) via [`brc20_deploy`](#deploy-contract) or [`brc20_call`](#call-contract)
  - [ ] [Deposit/Withdraw BRC20 tokens](#depositwithdrawal-inscriptions) via [`brc20_deposit`](#brc20-deposit) and [`brc20_withdraw`](#brc20-withdraw)
  - [ ] Finalise every block via [`brc20_finaliseBlock`](#finalise-block)
  - [ ] Commit changes to database via [`brc20_commitToDatabase`](#commit-to-database)
- [ ] Call [`brc20_reorg`](#reorg) when a reorg is detected
