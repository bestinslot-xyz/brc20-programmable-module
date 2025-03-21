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

#### Mine empty blocks

**Method**: `brc20_mine`

**Description**: Inserts empty blocks with unknown/unimportant hashes, this method can be used to speed up the initialisation process by skipping unnecessary blocks and moving the block height to given point for indexing purposes.

**Parameters**:

- block_count (`int`): Number of empty blocks to insert
- timestamp (`int`): Timestamp for the empty blocks

<hr>

#### Initialise and deploy BRC20_Controller contract

**Method**: `brc20_initialise`

**Description**: Initialises the execution engine with a known block height and hash, deploys the `BRC20_Controller` contract at address `0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb`. This method can be called before or after `brc20_mine`, but subsequent calls to it must have the same parameters, otherwise it will fail.

**Parameters**:

- genesis_hash (`string`): Block hash
- genesis_timestamp (`int`): Timestamp
- genesis_height (`int`): Block height

<hr>

#### Add Transaction to Block

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

#### Finalise Block

**Method**: `brc20_finaliseBlock`

**Description**: Finalises a block, this should be called after all the transactions in the block are added via `brc20_addTxToBlock`.

**Parameters**:

- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash
- block_tx_count (`int`): Number of transactions added to this block

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

**Description**: Deposits (mints) BRC20 tokens to given bitcoin pkscript. This is a convenience method to replace `brc20_addTxToBlock` calls for BRC20 transactions, and used to transfer BRC20 tokens into BRC2.0 module.

**Parameters**:

- to_pkscript (`string`): Bitcoin pkscript to receive BRC20 tokens
- ticker (`string`): Ticker for the BRC20 token
- amount (`string`): Amount of BRC20 tokens
- timestamp (`int`): Current block timestamp
- hash (`string`): Current block hash
- tx_idx (`int`): Transaction index

<hr>

#### BRC20 Withdraw

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

#### BRC20 Balance

**Method**: `brc20_balance`

**Description**: Returns a transaction receipt for retrieving current BRC20 balance (in-module) for the given pkscript and ticker.

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

`BRC20_Balance` contract can be used to retrieve non-module BRC20 balance for a given pkscript. BRC2.0 makes an HTTP call to the server at `BRC20_PROG_BALANCE_SERVER_URL` environment variable.

```
> curl "http://localhost:18546/?address=1234567890&ticker=blah"
86
```

BRC20 indexers should expose this HTTP server and set the environment variable accordingly.

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

> [!WARNING]
> `BRC20_PROG_BALANCE_SERVER_URL` must be set for this precompile to work.

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

BRC2.0 has a set of precompiles that make it easier to work with bitcoin transactions within a smart contract. These can be used to retrieve transaction details, track satoshis across transactions and calculate locked pkscripts. These allow BRC2.0 smart contracts to be aware of the transactions, ordinals and ordinal lockers that happen outside the execution engine.

> [!WARNING]
> `BTC_Transaction` and `BTC_LastSatLoc` precompiles use Bitcoin JSON-RPC calls to calculate results, so an RPC server needs to be specified in the environment variables.
> 
> Associated environment variables are `BITCOIN_RPC_URL`, `BITCOIN_RPC_USER`, `BITCOIN_RPC_PASSWORD` and `BITCOIN_NETWORK`. See [env.sample](env.sample) for a sample environment.

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

BRC2.0 execution engine is designed to work together with a BRC20 indexer, and the indexer should recognise inscriptions that are intended for BRC2.0 and execute transactions, deposit and withdraw BRC20 tokens.

### Deploy/Call inscriptions

Defined in the [proposal](https://github.com/bestinslot-xyz/brc20-prog-module-proposal), deploy inscriptions have the following structure:

```json
{
    "p": "brc20-prog",
    "op": "deploy",
    "d": "<bytecode + constructor_args in hex>"
}
```

Whenever an indexer encounters a deploy inscription, it should inform the programmable module via the `brc20_addTxToBlock` JSON-RPC method, leaving `to` parameter empty, this will allow the EVM to deploy a new smart contract.

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

Similar to a deploy inscription, call inscriptions should also be added as transactions to the EVM using `brc20_addTxToBlock` JSON-RPC method. For call transaction, `to` field should be set to the contract address to determine which contract should be called.

If the `c` field is set, then it can be used directly as the `to` value. If the `i` field is set instead, then the deployed contract address from corresponding deploy inscription should be used.

> [!NOTE]
> This requires maintaining a map of deploy inscription id to contract address. `brc20_addTxToBlock` returns a standard transaction receipt that contains contract addresses for deploy transactions.

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
> Tokens should be withdrawn from the sender's address, but deposited to the receiver's address for a withdraw inscription. A withdraw inscription can be sent to the same address, or a different address.

### Initialisation and empty blocks

Execution engine deploys a `BRC20_Controller` contract for BRC20 deposits, transfers and withdrawals. This deployment should be triggered by an indexer via `brc20_initialise` method at any point, before any of the inscriptions take place. This will add a block with a single transaction that is the `BRC20_Controller` deployment transaction.

In order to skip initial blocks, i.e. empty blocks, miners can call `brc20_mine` to add empty blocks to the system. If the first inscription is at block height 100, then initialisation might look like:

```
brc20_mine { block_count: 100, timestamp: 0 }
brc20_initialise { genesis_hash: "100TH_BLOCK_HASH", genesis_timestamp: "100TH_BLOCK_TIMESTAMP", genesis_height: 100 }
```

If an indexer wants earlier block hashes and timestamps to be correct, they can also initialise empty blocks using `brc20_finaliseBlock`, and pass the correct hashes and timestamps.

```
brc20_initialise { genesis_hash: "GENESIS_HASH", genesis_timestamp: "GENESIS_TIMESTAMP", genesis_height: 0 }
for all initial blocks:
  brc20_finaliseBlock { hash: "KNOWN_HASH", timestamp: "KNOWN_TIMESTAMP", block_tx_count: 0 }
```
### Loop for adding transactions and finalising blocks

When a new block arrives, all its deploy/call/deposit/withdraw transactions should be sent to the execution engine in order, with the correct transaction index using the relevant methods such as `brc20_addTxToBlock`, `brc20_deposit`, and `brc20_withdraw`. Once all inscriptions in the block are processed, block should be finalised using the `brc20_finaliseBlock` JSON-RPC method.

Indexing for a single block in pseudo code would look like the following (field validation is omitted for simplicity):

```
# contract_address_map is a Map<InscriptionID, ContractAddress>

block = await_new_block()
current_tx_idx = 0
for (inscription, transfer) in block:
    inscription_id = transfer.inscription_id
    sender = transfer.sender
    receiver = transfer.receiver

    if inscription.op is 'deploy' and receiver.pkscript is OP_RETURN "BRC20PROG":
        # Deploy transactions are added with `to` set to None
        result = brc20_addTxToBlock(
            from_pkscript: sender.pkscript,
            to: None,
            data: inscription.d,
            hash: block.hash,
            timestamp: block.timestamp,
            tx_idx: current_tx_idx++)
        if result.status is '0x1':
            # Contract address is saved for later use
            contract_address_map[inscription_id] = result.contractAddress

    if inscription.op is 'call' and receiver.pkscript is OP_RETURN "BRC20PROG":
        # Call transactions are added with `to` set to contract address
        brc20_addTxToBlock(
            from_pkscript: sender.pkscript,
            to: inscription.c OR
                contract_address_map[inscription.i],
            data: inscription.d,
            hash: block.hash,
            timestamp: block.timestamp,
            tx_idx: current_tx_idx++)

    if inscription.op is 'transfer' and receiver.pkscript is OP_RETURN "BRC20PROG":
        if sender.balance[inscription.tick] > inscription.amt:
            sender.balance[inscription.tick] -= inscription.amt;
            brc20_deposit(
                to_pkscript: sender.pkscript,
                ticker: inscription.tick,
                amount: inscription.amt (padded to 18 decimals),
                hash: block.hash,
                timestamp: block.timestamp,
                tx_idx: current_tx_idx++)

    if inscription.op is 'withdraw' and
       inscription.p is 'brc20-module' and
       inscription.module is 'BRC20PROG':
        # Withdrawals are done from sender's address
        result = brc20_withdraw(
            from_pkscript: sender.pkscript,
            ticker: inscription.tick,
            amount: inscription.amt (padded to 18 decimals),
            hash: block.hash,
            timestamp: block.timestamp,
            tx_idx: current_tx_idx++)
        # Withdrawals fail if there is not enough funds
        if result.status = '0x1':
            # Note that withdrawals are sent to receiver's address
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

Indexers should expose a balance server that returns current overall balance for an address and a ticker, and set the `BRC20_BALANCE_SERVER_URL` environment variable to make sure the `BRC20_Balance` precompiled contract knows where to send these requests to.

```
> curl "http://localhost:18546/?address=1234567890&ticker=blah"
86
```

### Indexer Checklist

- [ ] Set environment variables, check [env.sample](env.sample) for a list
- [ ] Start a [BRC20 balance server](#brc20-balance-server) for [BRC20_Balance Contract](#brc20-balance-contract)
- [ ] Mine [`brc20_mine`](#mine-empty-blocks) or finalise empty blocks [`brc20_finaliseBlock`](#finalise-block) to fill the database before the first inscription height
- [ ] Deploy the `BRC20_Controller` contract by calling [`brc20_initialise`](#initialise-and-deploy-brc20_controller-contract)
- [ ] Index every block for BRC2.0 transactions
  - [ ] [Add deploy/call inscriptions](#deploycall-inscriptions) via [`brc20_addTxToBlock`](#add-transaction-to-block)
  - [ ] [Deposit/Withdraw BRC20 tokens](#depositwithdrawal-inscriptions) via [`brc20_deposit`](#brc20-deposit) and [`brc20_withdraw`](#brc20-withdraw)
  - [ ] Finalise every block via [`brc20_finaliseBlock`](#finalise-block)
  - [ ] Commit changes to database via [`brc20_commitToDatabase`](#commit-to-database)
- [ ] Call [`brc20_reorg`](#reorg) when a reorg is detected
