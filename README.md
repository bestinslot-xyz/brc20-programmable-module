<h1 align="center">BRC20 Programmable Module</h1>
<p align="center">Smart contract execution engine compatible with BRC20 standard.</p>
<div align="center">

[![BRC2.0](https://github.com/bestinslot-xyz/brc20-programmable-module/actions/workflows/rust.yml/badge.svg)](https://github.com/bestinslot-xyz/brc20-programmable-module/actions)
[![Discord](https://dcbadge.vercel.app/api/server/6G8yPAcP3Z?style=flat)](https://discord.com/invite/6G8yPAcP3Z)

</div>

BRC2.0 provides smart contract execution capabilities for BRC20 indexers.

See our proposal at [bestinslot-xyz/brc20-prog-module-proposal](https://github.com/bestinslot-xyz/brc20-prog-module-proposal) for detailed information about how the BRC2.0 module works alongside the indexers.

For questions, comments and requests, use [the issues section](https://github.com/bestinslot-xyz/brc20-programmable-module/issues) or [Best in Slot discord server](https://discord.com/invite/6G8yPAcP3Z).

## Usage

**Build and run brc20_prog:**

```
cargo run --release
```

Bitcoin and BRC20 related precompiled contracts require environment variables to work properly, see the [Precompiles](#precompiles) section to learn how to set them up, otherwise precompiled contracts will fail.

## Supported Methods

BRC2.0 provides a JSON-RPC 2.0 server to interact with the indexers, and chain explorers. `eth_*` methods are supported to provide information on blocks and transactions, while `brc20_*` methods are used for adding new transactions and blocks to run in the execution engine.

### eth_* methods

BRC2.0 implements the [Ethereum JSON-RPC API](https://ethereum.org/en/developers/docs/apis/json-rpc/).

JSON-RPC methods work the same way as the official implementation, e.g. `eth_blockNumber` will return the latest indexed block height, `eth_getBlockByNumber` or `eth_getBlockByHash` will return an indexed block and all the indexed transactions, and `eth_getTransactionReceipt` will return the transaction receipt for given transaction, including logs and status.

`eth_call` can be used to interact with the contracts.

> [!WARNING]
> Filter methods such as `eth_newFilter`, `eth_getFilterChanges` are not supported yet, but they are planned for after release.

### brc20_* methods (for indexers)

BRC2.0 implements following `brc20_*` JSON-RPC methods intended for indexer usage

**Method**: `brc20_mine`

**Description**: Inserts empty blocks with unknown/unimportant hashes, this method can be used to speed up the initialisation process by skipping unnecessary blocks and moving the block height to given point for indexing purposes.

**Parameters**:

- block_count (`int`): Number of empty blocks to insert
- timestamp (`int`): Timestamp for the empty blocks

<hr>

**Method**: `brc20_initialise`

**Description**: Initialises the execution engine with a known block height and hash, deploys the `BRC20_Controller` contract at address `0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb`. This method can be called before or after `brc20_mine`, but subsequent calls to it must have the same parameters, otherwise it will fail.

**Parameters**:

- genesis_hash (`string`): Block hash
- genesis_timestamp (`int`): Timestamp
- genesis_height (`int`): Block height

<hr>

**Method**: `brc20_addTxToBlock`

**Description**: Used to deploy or call a contract, this adds a transaction to current block.

**Parameters**:

- from_pkscript (`string`): Bitcoin pkscript that created the deploy/call inscription
- to (Optional `string`): Contract address, if this is a call inscription, and can be skipped if this is a deploy inscription
- data (`string`): Call or deploy data for EVM
- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash
- tx_idx (`int`): Transaction index, starts from 0 every block, and needs to be incremented for every transaction

<hr>

**Method**: `brc20_finaliseBlock`

**Description**: Finalises a block, this should be called after all the transactions in the block are added via `brc20_addTxToBlock`.

**Parameters**:

- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash
- block_tx_count (`int`): Number of transactions added to this block

<hr>

**Method**: `brc20_commitToDatabase`

**Description**: Writes pending changes to disk.

**Parameters**:

- None

<hr>

**Method**: `brc20_clearCaches`

**Description**: Removes pending changes. Can be used to clear recently added transactions and revert to last saved state.

**Parameters**:

- None

<hr>

**Method**: `brc20_reorg`

**Description**: Reverts to a previous state at the given block. Should be used when a reorg is detected.

**Parameters**:

- latest_valid_block_number (`int`): Block height to revert the state to

> [!NOTE]
> Not all of the history is stored, and reorg is only supported up to 10 blocks earlier (this can be modified in code if needed, but will result in increased storage), otherwise this method will fail and return an error.

<hr>

**Method**: `brc20_deposit`

**Description**: Deposits (mints) BRC20 tokens to given bitcoin pkscript. This is a convenience method to replace `brc20_addTxToBlock` calls for BRC20 transactions, and used to transfer BRC20 tokens into BRC2.0 module.

**Parameters**:

- to_pkscript (`string`): Bitcoin pkscript to receive BRC20 tokens
- ticker (`string`): Ticker for the BRC20 token
- amount (`string`): Amount of BRC20 tokens
- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash
- tx_idx (`int`): Transaction index

<hr>

**Method**: `brc20_withdraw`

**Description**: Withdraws (burns) BRC20 tokens from given bitcoin pkscript. Method returns an error if the address doesn't have enough tokens. This is a convenience method to replace `brc20_addTxToBlock` calls for BRC20 transactions, and used to transfer BRC20 tokens out of BRC2.0 module.

**Parameters**:

- from_pkscript (`string`): Bitcoin pkscript to burn BRC20 tokens
- ticker (`string`): Ticker for the BRC20 token
- amount (`string`): Amount of BRC20 tokens
- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash
- tx_idx (`int`): Transaction index

<hr>

**Method**: `brc20_balance`

**Description**: Returns a transaction receipt for retrieving current BRC20 balance for the given pkscript and ticker.

**Parameters**:

- address_pkscript (`string`): Bitcoin pkscript
- ticker (`string`): BRC20 ticker

## Precompiles

Execution engine has precompiled contracts deployed at given addresses to make it easier to work with bitcoin transactions.

| Precompile | Address |
| --- | --- |
| BRC20_Balance      | 0x00000000000000000000000000000000000000ff |
| BIP322_Verifier    | 0x00000000000000000000000000000000000000fe |
| BTC_Transaction    | 0x00000000000000000000000000000000000000fd |
| BTC_LastSatLoc     | 0x00000000000000000000000000000000000000fc |
| BTC_LockedPkScript | 0x00000000000000000000000000000000000000fb |

### BRC20 Balance Contract

`BRC20_Balance` contract can be used to retrieve non-module BRC20 balance for a given pkscript. BRC2.0 makes an RPC call to the server at `BRC20_PROG_BALANCE_SERVER_URL` environment variable.

```
> curl "http://localhost:18546/?address=1234567890&ticker=blah"
86
```

Indexers should expose a server and set the environment variable accordingly.

**Contract interface**:

```solidity
/**
 * @dev Get non-module BRC-20 balance of a given Bitcoin wallet script and BRC-20 ticker.
 */
interface IBRC20_Balance {
    function balanceOf(
        string calldata ticker,
        string calldata address_pkscript
    ) external view returns (uint256);
}
```

### BIP322 Verifier Contract

`BIP322_Verifier` contract can be used to verify a BIP322 signature.

**Contract interface**:

```solidity
/**
 * @dev BIP322 verification method
 */
interface IBIP322_Verifier {
    function verify(
        string calldata addr,
        string calldata message_base64,
        string calldata signature_base64
    ) external returns (bool);
}
```

### Bitcoin Contracts

#### Transaction details

`BTC_Transaction` contract can be used to retrieve details for a bitcoin transaction. Returns block height, and `vin`, `vout` txids, scriptPubKeys and values as arrays.

**Contract interface**:

```solidity
/**
 * Get Bitcoin transaction details using tx ids.
 */
interface IBTC_Transaction {
    function getTxDetails(
        string calldata txid
    )
        external
        view
        returns (
            uint256 block_height,
            string[] memory vin_txids,
            uint256[] memory vin_vouts,
            string[] memory vin_scriptPubKey_hexes,
            uint256[] memory vin_values,
            string[] memory vout_scriptPubKey_hexes,
            uint256[] memory vout_values
        );
}
```

#### Last sat location

`BTC_LastSatLoc` contract can be used to retrieve previous location of a satoshi at given `txid`, `vout` and `sat` number.

**Contract interface**:

```solidity
/**
 * @dev Get last satoshi location of a given sat location in a transaction.
 */
interface IBTC_LastSatLoc {
    function getLastSatLocation(
        string calldata txid,
        uint256 vout,
        uint256 sat
    ) external view returns (string memory last_txid, uint256 last_vout, uint256 last_sat, string memory old_pkscript, string memory new_pkscript);
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
        string calldata address_pkscript,
        uint256 lock_block_count
    ) external view returns (string memory locked_pkscript);
}
```

## Indexer Integration Guide

### Deploy/Call inscriptions

### Deposit/Withdrawal inscriptions

### Initialisation and empty blocks

### Loop for adding transactions and finalising blocks

### BRC20 Balance Server
