# Indexer JSON-RPC methods (`brc20_*`)

BRC2.0 implements the following `brc20_*` JSON-RPC methods intended for indexer usage. They add blocks and transactions to the execution engine. See [Indexer integration](indexer-integration.md) for the order an indexer calls them in, and [Ethereum JSON-RPC methods](eth-api.md) for the read methods.

When `BRC20_PROG_ADMIN_RPC_SERVER_URL` is set, the indexer methods are served only on the admin listener. See [Admin RPC server](configuration.md#admin-rpc-server).

## Mine empty blocks

**Method**: `brc20_mine`

**Description**: Inserts empty blocks with unknown/unimportant hashes, this method can be used to speed up the initialisation process by skipping unnecessary blocks and moving the block height to given point for indexing purposes.

**Parameters**:

- block_count (`int`): Number of empty blocks to insert
- timestamp (`int`): Timestamp for the empty blocks

<hr>

## Initialise and deploy BRC20_Controller contract

**Method**: `brc20_initialise`

**Description**: Initialises the execution engine with a known block height and hash, deploys the `BRC20_Controller` contract at address `0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb`. This method can be called before or after `brc20_mine`, but subsequent calls to it must have the same genesis parameters, otherwise it will fail.

**Parameters**:

- genesis_hash (`string`): Block hash
- genesis_timestamp (`int`): Timestamp
- genesis_height (`int`): Block height

**Returns**:

- Error if block info doesn't match a previous `brc20_initialise` call.

<hr>

## Deploy contract

**Method**: `brc20_deploy`

**Description**: Used to deploy a contract, this adds a transaction to current block.

**Parameters**:

- `from_pkscript` (`string`): Bitcoin pkscript that created the deploy/call inscription
- `data` (Optional `string`): Call or deploy data for EVM, corresponds to the "d" (Data) field of a deploy inscription
- `base64_data` (Optional `string`): Call or deploy data for EVM, encoded in base64 with the compression prefix, corresponds to the "b" (Base64 Data) field of a deploy inscription
- `timestamp` (`int`): Current block timestamp
- `hash` (`string`): Current block hash
- `tx_idx` (`int`): Transaction index, starts from 0 every block, and needs to be incremented for every transaction
- `inscription_id` (Optional `string`): Source inscription ID that triggered this transaction, will be recorded for easier contract address retrieval
- `inscription_byte_len` (Optional `number`): Length of the insription content, used to determine the gas limit for this transaction
- `op_return_tx_id` (Optional `string`): Bitcoin transaction ID that sends the inscription to OP_RETURN, used for the precompile

**Returns**:

- Receipt for the executed transaction, see [eth_getTransactionReceipt](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt) for details.

<hr>

## Call contract

**Method**: `brc20_call`

**Description**: Used to call a contract, this adds a transaction to current block.

**Parameters**:

- `from_pkscript` (`string`): Bitcoin pkscript that created the deploy/call inscription
- `contract_address` (Optional `string`): Address of the contract to call, corresponds to the "c" (Contract Address) field of a call inscription
- `contract_inscription_id` (Optional `string`): Contract deployed by the inscription ID to call, corresponds to the "i" (Inscription ID) field of a call inscription
- `data` (Optional `string`): Call or deploy data for EVM, corresponds to the "d" (Data) field of a call inscription
- `base64_data` (Optional `string`): Call or deploy data for EVM, encoded in base64 with the compression prefix, corresponds to the "b" (Base64 Data) field of a call inscription
- `timestamp` (`int`): Current block timestamp
- `hash` (`string`): Current block hash
- `tx_idx` (`int`): Transaction index, starts from 0 every block, and needs to be incremented for every transaction
- `inscription_id` (Optional `string`): Inscription ID that triggered this transaction, will be recorded for easier transaction receipt retrieval
- `inscription_byte_len` (Optional `number`): Length of the insription content, used to determine the gas limit for this transaction
- `op_return_tx_id` (Optional `string`): Bitcoin transaction ID that sends the inscription to OP_RETURN, used for the precompile

**Returns**:

- Receipt for the executed transaction, see [eth_getTransactionReceipt](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt) for details.

> [!NOTE]
> `inscription_byte_len` parameter is used to determine the gas limit for `brc20_deploy` and `brc20_call` transactions, currently BRC2.0 sets an allowance of 12000 gas per byte (object to change, but generously set). In case of calling expensive methods and contracts, inscriptions should be padded to increase the gas allowance. Minimum gas limit is set to 32 bytes per transaction. `eth_estimateGas` JSON-RPC method can be used to estimate how much gas this transaction might consume.

<hr>

## Send raw signed transaction

**Method**: `brc20_transact`

**Description**: Used to send a raw signed transaction, this adds a transaction to current block. This is useful for sending transactions that are pre-signed using ethereum wallets.

**Parameters**:

- `raw_tx_data` (Optional `string`): Raw signed transaction data, encoded in hex format.
- `base64_raw_tx_data` (Optional `string`): Raw signed transaction data, encoded in base64 with the compression prefix.
- `timestamp` (`int`): Current block timestamp.
- `hash` (`string`): Current block hash.
- `tx_idx` (`int`): Transaction index, starts from 0 every block, and needs to be incremented for every transaction.
- `inscription_id` (Optional `string`): Inscription ID that triggered this transaction, will be recorded for easier transaction receipt retrieval.
- `inscription_byte_len` (Optional `number`): Length of the inscription content, used to determine the gas limit for this transaction.
- `op_return_tx_id` (Optional `string`): Bitcoin transaction ID that sends the inscription to OP_RETURN, used for the precompile

**Returns**:

- List of receipts for the executed transactions, see [eth_getTransactionReceipt](https://ethereum.org/en/developers/docs/apis/json-rpc/#eth_gettransactionreceipt) for details.

- If the transaction nonce is not in order, zero receipts will be returned, as transaction will be stored as a pending transaction. In that case, `tx_idx` for the next call shouldn't be incremented in that case.

- Multiple receipts can be returned if the pending transaction pool contains multiple transactions with nonces following the current transaction, as they will be executed together. In that case, `tx_idx` for the next call should be incremented by the number of transactions executed.

<hr>

## Get Transaction Receipt by Inscription ID

**Method**: `brc20_getTxReceiptByInscriptionId`

**Description**: Returns the transaction receipt for given inscription ID, previously sent via `brc20_deploy` or `brc20_call`. This makes it easier to work with inscriptions rather than transactions in BRC2.0 applications.

**Parameters**:

- inscription_id (`string`): Inscription ID previously added via `brc20_deploy`, `brc20_call`, `brc20_deposit`, or `brc20_withdraw`.

**Returns**:

- Transaction receipt, following `eth_getTransactionReceipt` structure.
- None if the inscription isn't added yet, i.e. it doesn't match previous calls.

<hr>

## Get Inscription ID by Transaction Hash

**Method**: `brc20_getInscriptionIdByTxHash`

**Description**: Returns the inscription ID for given transaction, previously sent via `brc20_deploy` or `brc20_call`. This makes it easier to work with inscriptions rather than transactions in BRC2.0 applications.

**Parameters**:

- tx_hash (`string`): Transaction hash previously added via `brc20_deploy`, `brc20_call`, `brc20_deposit`, or `brc20_withdraw`.

**Returns**:

- Inscription ID, as string
- None if the transaction doesn't have an inscription

<hr>

## Finalise Block

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

## Commit to Database

**Method**: `brc20_commitToDatabase`

**Description**: Writes pending changes to disk.

**Parameters**:

- None

> [!NOTE]
> A commit or reorg that stops partway, from a crash, a kill or a power cut, is rolled back when the database next opens, to the last height whose commit finished. The indexer then reads that height with `eth_blockNumber` and replays the blocks after it. A finished commit is synced to disk before it returns.
>
> In the two cases the rollback cannot cover, opening the database wipes it instead and logs a warning naming the directory, the marked height and the reason: the first commit into an empty database was interrupted, or the interrupted commit spans more blocks than the reorg history holds (10). The wipe deletes the module's own databases and keeps every other file in the directory. The module then reports an empty chain, and the indexer replays from its activation height. A wipe that is itself interrupted finishes on the next open.

<hr>

## Clear Caches

**Method**: `brc20_clearCaches`

**Description**: Removes pending changes. Can be used to clear recently added transactions and revert to last saved state.

**Parameters**:

- None

<hr>

## Reorg

**Method**: `brc20_reorg`

**Description**: Reverts to a previous state at the given block. Should be used when a reorg is detected.

**Parameters**:

- latest_valid_block_number (`int`): Block height to revert the state to

> [!NOTE]
> Not all of the history is stored, and reorg is only supported up to 10 blocks earlier (this can be modified in code if needed, but will result in increased storage), otherwise this method will fail and return an error.

<hr>

## BRC20 Deposit

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

## BRC20 Withdraw

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

## BRC20 Balance

**Method**: `brc20_balance`

**Description**: Returns a transaction receipt for retrieving current BRC20 balance (in-module) for the given pkscript and ticker.

**Parameters**:

- pkscript (`string`): Bitcoin pkscript
- ticker (`string`): BRC20 ticker

**Returns**:

- (string) BRC20 balance of the bitcoin pkscript for the given ticker
