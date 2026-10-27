# Differences from Ethereum

BRC2.0 executes transactions with [`revm`](https://github.com/bluealloy/revm) under Ethereum rules (CANCUN or PRAGUE, see [Hardforks and activation heights](#hardforks-and-activation-heights)), with the four differences below. All four are consensus behaviour on mainnet and are pinned by the consensus fixtures (see [Testing](testing.md#consensus-fixtures)). Changing any of them requires an activation height.

## A failing Ethereum precompile aborts the whole transaction

When an Ethereum precompile (`0x01` to `0x0a`, plus `0x0b` to `0x11` from PRAGUE) rejects its input or runs out of gas, `BRC20Precompiles::run` in `src/engine/precompiles/precompiles.rs` returns an error instead of a failed call result. Examples are an invalid KZG proof, malformed bn254 or BLS12-381 input, a blake2f input of the wrong length, and too little gas for ecrecover or sha256.

The error ends the whole transaction: the receipt has status `0x0` and `gasUsed` `0x0`, there are no logs, and the trace stops at the failing precompile call. On Ethereum the precompile call fails and the calling contract continues with `success = false`.

The BRC2.0 precompiles (`0xfa` to `0xfe`, see [Precompiles](precompiles.md)) return rejected input and out of gas as a failed call result, so the caller continues as on Ethereum.

Fixture cases: `precompileEdge:*` (one transaction per rejected input or out-of-gas call) and `topLevelPointEvaluationZeros` (a transaction sent straight to the point evaluation precompile `0x0a`).

## A contract created and self-destructed in the same transaction stays alive

`DatabaseCommit::commit` in `src/db/brc20_prog_database.rs` writes the balance, nonce, code hash, code and changed storage of every touched account, and does not act on the self-destruct flag. A contract that is created and self-destructed in the same transaction keeps its code and storage, and later transactions can call it.

On Ethereum, [EIP-6780](https://eips.ethereum.org/EIPS/eip-6780) deletes such a contract at the end of the transaction.

Fixture cases: `createAndDestroy` creates and destroys a child contract, and `probeDestroyedChild` reads its code size, code hash and `value()` in a later transaction.

## DELEGATECALL and CALLCODE to a precompile run the empty code at its address

`BRC20Precompiles::run` looks up precompiles by the call's `target_address`. For CALL and STATICCALL that is the precompile address. For DELEGATECALL and CALLCODE the target is the calling contract, so no precompile matches and the call runs the code at the precompile address, which is empty: the call succeeds with empty output.

On Ethereum, precompiles are resolved by the code address, so DELEGATECALL and CALLCODE to a precompile execute it.

This applies to the Ethereum precompiles and the BRC2.0 precompiles alike. Fixture case: `precompileCallKinds` (CALL, DELEGATECALL and CALLCODE to sha256 `0x02`, and DELEGATECALL to `0xfe` and `0xfa`).

## The current txid precompile `0xfa` is not pre-warmed

Precompile addresses are warm from the start of every transaction ([EIP-2929](https://eips.ethereum.org/EIPS/eip-2929)). `BRC20Precompiles::new` builds that set (`all_addresses`) from the Ethereum precompiles and `0xfb` to `0xfe`. The current txid precompile `0xfa` is not in it, so the first access to `0xfa` in a transaction pays the cold account access cost, and later accesses in the same transaction pay the warm cost.

## Hardforks and activation heights

Consensus changes are introduced behind per-network activation heights in `src/engine/hardforks.rs`. Blocks below the height keep the old rules, so a node replays existing blocks unchanged. The network comes from `BITCOIN_RPC_NETWORK` (see [Configuration](configuration.md)).

| Change | Mainnet | Signet | Other networks |
| ------ | ------- | ------ | -------------- |
| EVM spec PRAGUE (CANCUN below the height) | 923,369 | 275,000 | PRAGUE at every height |
| Transaction hash is the RLP hash of the signed transaction (signing hash below the height) | 929,000 | 0 | RLP hash at every height |
| BIP322 verification with bip322 v0.0.11 rules (v0.0.10 rules below the height) | 960,847 | 328,057 | v0.0.11 rules at every height |

- `get_evm_spec` selects the EVM spec. The current txid precompile `0xfa` is available from PRAGUE, so it activates at the same heights.
- `use_rlp_hash_for_tx_hash` selects the transaction hash. The mainnet activation needs no reindex. On signet the RLP hash applies from height 0, which requires a reindex.
- `use_bip322_v0_0_11` selects the BIP322 rules. bip322 v0.0.11 accepts short DER signatures, P2PKH and multisig proofs that v0.0.10 rejects.

A change to any of the four differences above, a dependency upgrade that changes execution (for example revm, bip322, base64 or zstd), or any other change that alters a consensus fixture snapshot follows the same pattern: a new activation height per network in `src/engine/hardforks.rs`, with the old behaviour kept for blocks below it.
