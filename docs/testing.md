# Testing

CI (`.github/workflows/rust.yml`) runs three parallel jobs, each with its own build cache:

```
cargo check --no-default-features
cargo test --all-features -- --skip public_api
cargo test --all-features --test public_api
```

The public API snapshot has its own job because it builds the crate a second time with a nightly toolchain.

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

It then renders one text report: per block the trace hash, the block header, the raw header, the raw block, the raw receipts in transaction order and any rejected submissions, per transaction the receipt and the `callTracer` trace, and the final `brc20_balance`. The report is the insta snapshot in `tests/snapshots/consensus_fixture_{mainnet,signet}__*.snap`.

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

## Storage tests

The storage layout and its migration are described in [Storage](configuration.md#storage).

- `src/db/atomic_commit_tests.rs` fails a commit before and after its write batch is written, and a reorg the same way, through fail points in `src/db/store.rs` that only test builds arm. After a reopen the store is at the previous block with every column family unchanged, or at the new block with the same contents as an uninterrupted run, and re-executing the lost block gives the same trace hash and receipts.
- `src/db/store.rs` tests migrate a copy of `tests/fixtures/v0_16_6_mainnet_db` and compare every column family with its source. They interrupt the migration after each copied column family, before `storage_version` is written, before and after the rename and midway through the cleanup, and check that the next open completes it. An interrupted migration must leave the per-database directories byte-identical. They also cover a cleanup that deletes only the dirs listed in `migrated_from`, the empty `config` database an earlier release leaves next to `engine`, a leftover `engine.migrating`, an open while another holds `engine.lock`, too little free space (through a test hook for the free-space query), a newer `storage_version` (the store must stay byte-identical) and a store without one.
- `src/global/database.rs` tests start on a copy of the fixture with the wrong network, and without a `config` database, and check that the start fails with the config error and leaves every directory byte-identical, with no `engine` or `engine.migrating`. They also check that a new store holds its configuration before it is renamed to `engine`, and that a start while another holds `engine.lock` fails without writing anything.
- `tests/storage_migration.rs` opens a server on a copy of the fixture, so the migration runs at startup, and checks that the consensus fixture report rendered from the migrated data dir equals both a fresh run and the mainnet snapshot, and that every column family equals its v0.16.6 database.

`tests/fixtures/v0_16_6_mainnet_db` is a data dir written by brc20-prog v0.16.6: the mainnet consensus fixture workload, committed with `brc20_commitToDatabase`, with the RocksDB `LOG` and `LOCK` files left out. `tests/fixtures/regen_v0_16_6_mainnet_db.sh` regenerates it: it builds the `v0.16.6` tag in a temporary git worktree, points the test server at a fixed data dir, commits after the mainnet consensus fixture workload and copies the result without those files. The block databases record wall-clock processing time, so a regenerated fixture differs from the committed one in those values only.

```
tests/fixtures/regen_v0_16_6_mainnet_db.sh
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
