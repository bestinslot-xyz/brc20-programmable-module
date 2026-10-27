# Testing

CI builds the crate with all features and with no default features, then runs:

```
cargo test --all-features
```

Unit tests live next to the code in `src/`. Integration tests in `tests/` start a server on a free local port with a temporary database through `test_utils::spawn_test_server`, and talk to it with `Brc20ProgApiClient`. Shared helpers and input data live in the `test_utils` crate (`test_utils/src/`, `test_utils/data/`).

Several tests pin their output with [insta](https://insta.rs) snapshots. To accept an intended change, run the test with `INSTA_UPDATE=always` and review the `.snap` diff before committing it.

## Consensus fixtures

The consensus fixtures pin everything that feeds consensus: gas accounting, call traces, receipts, logs, transaction hashing, signature handling and block trace hashes.

- `tests/consensus_fixture_mainnet.rs` runs on mainnet below the PRAGUE and RLP tx hash activation heights: CANCUN rules and signing-hash transaction hashes. Chain id `0x4252433230`.
- `tests/consensus_fixture_signet.rs` runs on signet: CANCUN blocks, the PRAGUE activation at height 275,000, PRAGUE blocks, and RLP transaction hashes. Chain id `0x425243323073`.

Both call `run_consensus_fixture` in `test_utils/src/consensus_fixture.rs`. It starts a fresh server, calls `brc20_initialise`, and runs a fixed workload for each era of the network:

1. A block that deploys the `Fixture` contract and calls storage, transient storage, memory, log, nested call, call kind and creation operations, plus a `brc20_deposit`.
2. A block of failure and limit cases: large runtime and initcode, self-destruct, reverts and invalid opcodes, logs in failing frames, math and environment opcodes, Ethereum and BRC2.0 precompiles, precompile call kinds, an opcode probe for opcodes from later forks, one transaction per precompile edge case (`precompileEdge:*`), calls straight to a precompile and to an empty account, and two `brc20_withdraw` calls, one above the balance.
3. A block of pre-signed `brc20_transact` transactions: a nonce gap that is queued and filled, a high-s signature, a wrong chain id and malformed data.

It then renders one text report: per block the trace hash, the block header, the raw header, the sorted raw receipts and any rejected submissions, per transaction the receipt and the `callTracer` trace, and the final `brc20_balance`. The report is the insta snapshot in `tests/snapshots/consensus_fixture_{mainnet,signet}__*.snap`.

**A snapshot diff in a consensus fixture means a consensus change.** Blocks already on chain would execute differently, so the change needs an activation height (see [Hardforks and activation heights](differences-from-ethereum.md#hardforks-and-activation-heights)) or is a bug.

### Fixture inputs

The inputs in `test_utils/data/consensus_fixture/` are pre-encoded, so the workload does not depend on the crates under test:

- `Fixture.sol`: the contract the fixture deploys and calls.
- `fixture_deploy_tx_data`: the creation bytecode of `Fixture`.
- `fixture_calls`: one `label calldata` pair per line.
- `fixture_txs_mainnet`, `fixture_txs_testnets`: signed legacy transactions for `brc20_transact`, for chain ids "BRC20" and "BRC20s".
- `malleate.py`: rewrites a signed legacy transaction into its high-s twin for the `highSCallKinds` transaction.

`build.sh` regenerates all of these from `Fixture.sol`. It requires [Foundry](https://getfoundry.sh) (`forge`, `cast`) and `python3`, and compiles with solc 0.8.28, EVM version cancun and 200 optimizer runs. Changing these inputs changes every consensus fixture snapshot, so regenerate them only together with an intentional snapshot update.

```
test_utils/data/consensus_fixture/build.sh
```

## Inscription decoding corpus

Inscription data decides transaction calldata, so decoding it is consensus. `test_inscription_data_corpus` in `src/api/types.rs` (built with the `server` feature) decodes every payload in `src/api/testdata/inscription_corpus.txt` and pins each outcome (decoded length and sha256, `rejected`, or `panic`) in `src/api/snapshots/brc20_prog__api__types__tests__inscription_data_corpus.snap`. A changed outcome, for example from a base64 or zstd upgrade, is a consensus change.

The corpus covers the base64, nada and zstd edge cases of `decode_bytes_from_inscription_data`. `src/api/testdata/gen_inscription_corpus.py` regenerates it and requires the `zstd` CLI:

```
cd src/api/testdata
python3 -I gen_inscription_corpus.py > inscription_corpus.txt
```

## Public API snapshot

`tests/public_api.rs` builds rustdoc JSON with all features and pins the crate's public API in `tests/snapshots/public_api__public_api.snap`. A diff means the public Rust API changed.

rustdoc JSON needs a nightly toolchain. The test pins it in `RUSTDOC_NIGHTLY` (`nightly-2026-10-01`), a nightly whose JSON format the `public-api` crate reads, and installs it through rustup when it is missing. Update the snapshot with:

```
INSTA_UPDATE=always cargo test --all-features --test public_api
```

## Bitcoin RPC tests and benchmarks

The `BTC_Transaction` (`0xfd`) and `BTC_LastSatLoc` (`0xfc`) precompiles read from a Bitcoin node, so their tests need a reachable node:

- `tests/btc_rpc_mainnet.rs` reads the node settings from `.env.mainnet`.
- `tests/btc_rpc_signet.rs` reads the node settings from `.env.signet`.

Each env file sets `BITCOIN_RPC_URL`, `BITCOIN_RPC_USER`, `BITCOIN_RPC_PASSWORD` and `BITCOIN_RPC_NETWORK` (see [Configuration](configuration.md)). Both tests return early when `CI=true`. In a local run they need the env file and the node; set `CI=true` to skip them.

The RPC settings are loaded into process-wide configuration, so two networks never share a process: each network's test is its own test binary, with shared helpers in `test_utils/src/btc_rpc.rs`. Run one network at a time with:

```
cargo test --all-features --test btc_rpc_signet
cargo test --all-features --test btc_rpc_mainnet
```

`tests/btc_rpc_local.rs` is an opt-in end-to-end check of the `0xfd` precompile against a live signet node. It reads `.env.signet` and runs only with `BRC20_RUN_BTC_RPC_TESTS=1` (in the shell or in `.env.signet`):

```
BRC20_RUN_BTC_RPC_TESTS=1 cargo test --all-features --test btc_rpc_local
```

The benchmarks follow the same split. `benches/precompiles.rs` needs no node. `benches/btc_precompiles_mainnet.rs` reads `.env.mainnet` and `benches/btc_precompiles_signet.rs` reads `.env.signet`, one network per binary:

```
cargo bench --bench precompiles
cargo bench --bench btc_precompiles_signet
cargo bench --bench btc_precompiles_mainnet
```

`.env*` files are ignored by git.
