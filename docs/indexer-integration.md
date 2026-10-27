# Indexer Integration Guide

BRC2.0 execution engine is designed to work together with a BRC20 indexer, and the indexer should recognise inscriptions that are intended for BRC2.0 and execute transactions, deposit and withdraw BRC20 tokens.

The indexer talks to the module over JSON-RPC (see [Indexer JSON-RPC methods](indexer-api.md)), or runs it in its own process (see [Embedding in an indexer](embedding.md)).

> [!NOTE]
> BRC20 indexer in [OPI/experimental-signet-brc20-prog](https://github.com/bestinslot-xyz/OPI/tree/experimental-signet-brc20-prog) branch already has the brc20-prog integration in place.

## Deploy/Call inscriptions

Defined in the [proposal](https://github.com/bestinslot-xyz/brc20-prog-module-proposal), deploy inscriptions have the following structure:

```json
{
    "p": "brc20-prog",
    "op": "deploy (or d)",
    "d": "<bytecode + constructor_args in hex>",
    "b": "<base64 encoded bytecode + constructor_args with the compression prefix>"
}
```

Whenever an indexer encounters a deploy inscription, it should inform the programmable module via the `brc20_deploy` JSON-RPC method, this will allow the EVM to deploy a new smart contract.

Once an inscription is deployed as a smart contract, then methods can be called via call inscriptions with the following structure:

```json
{
    "p": "brc20-prog",
    "op": "call (or c)",
    "c": "<contract_addr>",
    "i": "<inscription_id>",
    "d": "<call data>",
    "b": "<base64 encoded call data with the compression prefix>"
}
```

Call inscriptions should be added as transactions to the EVM using `brc20_call` JSON-RPC method. BRC2.0 maintains a map of contract addresses and deploy inscriptions, so at least one of the `"c"` or `"i"` fields should be set to call the contract `"c"`, or a contract deployed by the inscription `"i"`.

## Raw Signed Transaction Inscriptions

Raw signed transaction inscriptions are used to send pre-signed transactions to the execution engine. These inscriptions have the following structure:

```json
{
    "p": "brc20-prog",
    "op": "transact (or t)",
    "d": "<raw signed transaction data in hex>",
    "b": "<base64 encoded raw signed transaction data with the compression prefix>"
}
```

When an indexer encounters this inscription, it should call `brc20_transact` JSON-RPC method to send the raw signed transaction to the execution engine. This allows users to send pre-signed transactions using their EVM compatible wallets, and have them executed in the BRC2.0 module.

## Deposit/Withdrawal inscriptions

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

## Initialisation and empty blocks

Execution engine deploys a `BRC20_Controller` contract for BRC20 deposits, transfers and withdrawals. This deployment should be triggered by an indexer via `brc20_initialise` method first, before any blocks are added. This will add a block with a single transaction that is the `BRC20_Controller` deployment transaction.

In order to skip initial empty blocks, indexers need to call `brc20_mine` to add empty blocks to the system. If the first inscription is at block height 100, then initialisation might look like:

```
brc20_initialise {
    genesis_hash: "0x0000...0000",
    genesis_timestamp: "0",
    genesis_height: 0
}
brc20_mine {
    block_count: 99,
    timestamp: 0
}
```

This will create a block with the `BRC20_Controller` contract deployed at address `0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb`, and 99 more empty blocks, so the indexer can start processing inscriptions from block height 100.

This also makes sure every block hash before the first inscription is 0, and contracts can only access block hashes after the first BRC2.0 inscription.

## Loop for adding transactions and finalising blocks

When a new block arrives, all its deploy/call/deposit/withdraw transactions should be sent to the execution engine in order, with the correct transaction index using the relevant methods such as `brc20_deploy`, `brc20_call`, `brc20_transact`, `brc20_deposit`, and `brc20_withdraw`. Once all inscriptions in the block are processed, block should be finalised using the `brc20_finaliseBlock` JSON-RPC method.

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
            base64_data: inscription.b,
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
            base64_data: inscription.b,
            hash: block.hash,
            timestamp: block.timestamp,
            tx_idx: current_tx_idx++,
            inscription_id: current_inscription_id,
            inscription_byte_len: inscription.content.length)

    if inscription.op in ['transact', 't'] and
       receiver.pkscript is OP_RETURN "BRC20PROG":
        receipts = brc20_transact(
            raw_tx_data: inscription.d,
            base64_raw_tx_data: inscription.b,
            hash: block.hash,
            timestamp: block.timestamp,
            tx_idx: current_tx_idx,
            inscription_id: current_inscription_id,
            inscription_byte_len: inscription.content.length)
        current_tx_idx += receipts.length

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

## Securing the indexer methods

The indexer methods change chain state, so restrict who can call them. See [Authorization](configuration.md#authorization) for HTTP basic auth, and [Admin RPC server](configuration.md#admin-rpc-server) for serving the indexer methods on a separate listener.

## Indexer Checklist

- [ ] Set environment variables, check [env.sample](../env.sample) and [Configuration](configuration.md) for a list
- [ ] Mine [`brc20_mine`](indexer-api.md#mine-empty-blocks) or finalise empty blocks [`brc20_finaliseBlock`](indexer-api.md#finalise-block) to fill the database before the first inscription height
- [ ] Deploy the `BRC20_Controller` contract by calling [`brc20_initialise`](indexer-api.md#initialise-and-deploy-brc20_controller-contract)
- [ ] Index every block for BRC2.0 transactions
  - [ ] [Add deploy/call inscriptions](#deploycall-inscriptions) via [`brc20_deploy`](indexer-api.md#deploy-contract) or [`brc20_call`](indexer-api.md#call-contract)
  - [ ] [Deposit/Withdraw BRC20 tokens](#depositwithdrawal-inscriptions) via [`brc20_deposit`](indexer-api.md#brc20-deposit) and [`brc20_withdraw`](indexer-api.md#brc20-withdraw)
  - [ ] Finalise every block via [`brc20_finaliseBlock`](indexer-api.md#finalise-block)
  - [ ] Commit changes to database via [`brc20_commitToDatabase`](indexer-api.md#commit-to-database)
- [ ] Call [`brc20_reorg`](indexer-api.md#reorg) when a reorg is detected
