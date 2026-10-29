# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Storage format

 - The engine keeps all its data in one RocksDB at `<data dir>/engine`, with one column family per database, a `<name>_cache` column family per reorg history, and a `meta` column family holding `storage_version = 2`. Each `brc20_commitToDatabase` and `brc20_reorg` writes one batch across all column families, so a crash leaves the engine at the last committed block or at the new one. A power loss can drop the most recent commits but never part of one; the embedding indexer then replays from the engine's latest block.
 - A data dir in the per-database layout migrates automatically the first time it is opened, inside `Brc20Prog::open` and at server startup. The configuration check runs against the old data first, so a mismatched start leaves the data dir untouched. The migration runs once, needs free disk space of about 1.1 times the current data dir, blocks the open until it finishes (11 seconds for a 760 MB signet data dir), verifies every column family against its source, and deletes only the directories it copied, only after the new store is complete. An interrupted migration starts over on the next open. See [Storage](docs/configuration.md#storage).
 - **v0.16.6 and earlier cannot open a migrated data dir.** Keep a copy of the data dir before upgrading if a downgrade may be needed.

## v0.16.6 (2026-10-27)

### New Features

 - <csr-id-c011ca0f1ecb1e99fcfce95c12d60d9afbebec32/> let an embedding indexer supply Bitcoin data via BitcoinRpc

### Other

 - <csr-id-bcfb1dd2dfc2b8eeb8d56879460fe6eb83d30f6f/> run checks in parallel jobs with a build cache
   * ci: run checks in parallel jobs with a build cache
   
   The single job built with all features, built with no default features and
   then ran the tests, all from a cold build. It is now three parallel jobs,
   each with a Swatinem/rust-cache build cache:
   
   - check-no-default-features: cargo check --no-default-features
   - test: every test except public_api
   - public-api: the public API snapshot, which builds the crate a second time
     with a nightly toolchain for rustdoc JSON
   
   The separate cargo build --all-features step is dropped; cargo test compiles
   the same code. Superseded pull request runs are cancelled.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#25](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/25), [#26](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/26)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#25](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/25)**
    - Run checks in parallel jobs with a build cache ([`bcfb1dd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bcfb1dd2dfc2b8eeb8d56879460fe6eb83d30f6f))
 * **[#26](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/26)**
    - Let an embedding indexer supply Bitcoin data via BitcoinRpc ([`c011ca0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c011ca0f1ecb1e99fcfce95c12d60d9afbebec32))
</details>

## v0.16.5 (2026-10-27)

<csr-id-f930fc57c86f3eaa11366f9b496295797f976372/>
<csr-id-b6c8bef08f88fae99fe189bed88ef9e045d22e10/>
<csr-id-91e0519f76dcdf00cad25d7d9fb844157bcf9ff0/>
<csr-id-c4ee12940d1ba6a2022fb36c86188f5f80310dd7/>
<csr-id-b31ec9ccfa6d9c7c8b3f509379177e38f9992e38/>
<csr-id-5ac1b9f6c671a6beb9d826a675d9eb3787d207e7/>
<csr-id-ad6a6caef8a4adccf9ead3e4e0dcc7f8c4b471cf/>
<csr-id-0bef1d7cca5204ac4ba529373c68cccb13e5532a/>
<csr-id-cd21cb389d61fd8fc7fc700d9c741f30cb4bae3a/>
<csr-id-d3f8f5c33a288d7a55e68ce14dd34a82b998ceaf/>
<csr-id-878dba55ba080ecd9d093bc007a666df283789e7/>
<csr-id-ecdeb17fcafbfc10a5594eb6c4eeaef902bbdb09/>
<csr-id-4fb06c727782e47c94f6891a4d4ad9e41de2e836/>

### Chore

 - <csr-id-f930fc57c86f3eaa11366f9b496295797f976372/> bump non-EVM dependencies to their latest majors
   base64 0.23, zstd-safe 8 (libzstd stays 1.5.7), rocksdb 0.25 (RocksDB 11.8),
   rlimit 0.11, tower-http 0.7, public-api 0.52, rustdoc-json 0.9.10, rand 0.10.
   The consensus fixtures and the inscription decoding corpus are unchanged.
   
   public-api 0.52 renders signatures without parameter names and lists the
   jsonrpsee-generated client methods under TypeJsonRpseeInternal; the API itself
   is unchanged.
   
   bip322 stays at 0.0.11: 0.0.12 enforces LOW_S, adds variant prefixes and an
   inconclusive outcome, so adopting it needs its own activation height.
 - <csr-id-b6c8bef08f88fae99fe189bed88ef9e045d22e10/> update dependencies within their semver ranges
   cargo update: alloy 1.1 -> 1.8, alloy-core 1.4 -> 1.7, revm-inspectors
   0.33.0 -> 0.33.2, bitcoin 0.32.7 -> 0.32.102, blst 0.3.17, c-kzg 2.1.8,
   aurora-engine-modexp 1.3, jsonrpsee 0.26.1, serde_json 1.0.154 and
   transitive crates. alloy 1.8 requires rustc 1.91, so rust-version and the
   Docker builder image move to 1.91.
   
   Consensus fixtures and the inscription corpus are unchanged.

### Documentation

 - <csr-id-4c013176eb55996a3ea4227b2084421216005d88/> split the README into docs/ pages, add Differences from Ethereum
   * docs: split the README into focused pages and document differences from Ethereum
- configuration.md: environment variables, CLI options, authorization,
     admin RPC server
- indexer-integration.md: inscriptions, initialisation, indexing loop,
     checklist
- indexer-api.md: brc20_* method reference
- eth-api.md: eth_*, debug_* and txpool_content
- embedding.md: running the module in-process through Brc20Prog
- precompiles.md: BIP322 and Bitcoin precompiles
- testing.md: consensus fixtures, inscription corpus, public API
     snapshot, Bitcoin RPC tests and benches
- differences-from-ethereum.md: the four consensus differences from
     Ethereum and the activation heights in engine/hardforks.rs

### New Features

<csr-id-c550274239bd758e4710d6cd0b480335c0502de5/>

 - <csr-id-c2cdfe98524ef2e88bca21ced4c9e6cd102a46a7/> upgrade to revm 43, revm-inspectors 0.44 and alloy 2
   revm 33 -> 43, revm-inspectors 0.33 -> 0.44, revm-bytecode 7 -> 43,
   alloy 1.8 -> 2.5. alloy 2 requires rustc 1.94.1, so rust-version and the
   Docker builder image move to 1.94.
   
   Every block trace hash, receipt, gas figure, log, bloom and raw header/receipt
   encoding in the consensus fixtures is unchanged. Keeping it that way needed:
   
   - Precompiles: revm 43 reports rejected inputs and out-of-gas as a Halt status
   instead of an error. BRC20Precompiles maps any non-success status back to an
   error, so a failing Ethereum precompile call still aborts the transaction.
- Tracing: revm-inspectors 0.44 only records call input when record_inputs is
     set. The input is part of the trace string, so it is enabled.
- Instructions: the static gas table is fixed when EthInstructions is built.
     It is now built for the block's spec rather than revm's default (OSAKA).
- Logs: Revert and Halt results now carry logs. Receipts and the block log
     index take logs from successful results only, as before.

### Bug Fixes

 - <csr-id-c5242a4bbd2fc83eb65170e85118102a32176748/> return range query results in key order
 - <csr-id-8ce5e4f53986d187b3210d1d1530676b74ebd26a/> reject empty inscription payloads instead of panicking
   decode_bytes_from_inscription_data read the compression byte without a
   length check, so a payload of "" or "=" indexed an empty buffer and
   panicked. With panic = 'abort' in release builds this killed the node.
   An empty decoded buffer returns None, like any other undecodable payload.
   
   The inscription corpus snapshot records the empty and raw_only_padding
   cases as rejected, and a focused unit test covers "" and "=".

### Style

 - <csr-id-91e0519f76dcdf00cad25d7d9fb844157bcf9ff0/> rustfmt the consensus fixture and inscription corpus test

### Test

 - <csr-id-c4ee12940d1ba6a2022fb36c86188f5f80310dd7/> regenerate the public API snapshot on top of Brc20Prog
   public-api 0.52 renders signatures without parameter names and lists
   default trait methods on impls, so the Brc20Prog server methods with
   default bodies now appear. The API is unchanged.
 - <csr-id-b31ec9ccfa6d9c7c8b3f509379177e38f9992e38/> run each Bitcoin RPC network in its own binary, refresh expected gas
   The Bitcoin RPC client is process-wide and only rebuilt when the RPC status
   is validated, so loading .env.signet after .env.mainnet in one process kept
   the mainnet client and the signet precompile calls failed. The mainnet and
   signet tests now live in tests/btc_rpc_{mainnet,signet}.rs with shared
   helpers in test_utils, and the bench is split into
   btc_precompiles_{mainnet,signet}. Moving them out of tests/precompiles.rs
   also stops them reloading CONFIG under the signet precompile tests there.
   
   Expected gas values were stale; the old and new dependency sets both
   estimate 0xc8b36, 0x18c463 and 0x24fd91 for the mainnet calls and 0xc8b36
   for both signet calls.
 - <csr-id-5ac1b9f6c671a6beb9d826a675d9eb3787d207e7/> pin a newer nightly for the public API snapshot
   public-api's MINIMUM_NIGHTLY_RUST_VERSION (nightly-2025-11-22) is older than
   the rustc 1.94.1 that alloy 2 requires. nightly-2026-10-01 builds the crate
   and its rustdoc JSON still parses; the snapshot is unchanged.
 - <csr-id-ad6a6caef8a4adccf9ead3e4e0dcc7f8c4b471cf/> pin inactive opcodes and post-selfdestruct state
   opcodeProbe deploys runtimes using CLZ, DUPN/SWAPN/EXCHANGE and EOF opcodes,
   which must stay invalid under CANCUN and PRAGUE, next to active controls
   (PUSH0, BLOBHASH, BLOBBASEFEE, MCOPY). probeDestroyedChild reads a contract
   created and self-destructed in an earlier transaction: the engine keeps its
   code and storage, and the fixture pins that.
 - <csr-id-0bef1d7cca5204ac4ba529373c68cccb13e5532a/> cover logs emitted before top-level and nested failures
   Receipts of reverted, invalid and out-of-gas transactions carry no logs, and
   a reverted subcall's logs are dropped while the caller's survive. The fixture
   now pins all four.
 - <csr-id-cd21cb389d61fd8fc7fc700d9c741f30cb4bae3a/> cover precompiles reached by DELEGATECALL/CALLCODE and top-level calls
   The engine resolves precompiles by target address, so DELEGATECALL and
   CALLCODE to a precompile run the empty code at that address. The fixture now
   pins that, plus transactions sent straight to sha256 and to the point
   evaluation precompile with a rejected input.
 - <csr-id-d3f8f5c33a288d7a55e68ce14dd34a82b998ceaf/> cover precompile out-of-gas and the EIP-7823 modexp limit
   Adds ecrecover and sha256 calls with too little gas, and a modexp with a
   1025-byte modulus, as separate fixture transactions. Like a rejected input, a
   precompile running out of gas aborts the whole transaction (status 0, no gas
   used). The modexp succeeds: EIP-7823 is an Osaka rule and stays off under
   PRAGUE. The per-transaction precompile cases are now labelled precompileEdge.
 - <csr-id-878dba55ba080ecd9d093bc007a666df283789e7/> pin raw header and receipt RLP in the consensus fixture
   The fixture now also records debug_getRawHeader and the debug_getRawReceipts
   encodings per block. Receipts are sorted because the raw block lists
   transactions in storage iteration order, which varies between runs; the raw
   block itself is left out for the same reason.
 - <csr-id-ecdeb17fcafbfc10a5594eb6c4eeaef902bbdb09/> pin inscription data decoding over a base64/nada/zstd corpus
   Inscription payloads decide transaction calldata, so their decoding is
   consensus. test_inscription_data_corpus snapshots the outcome of
   src/api/testdata/inscription_corpus.txt: canonical and non-canonical base64,
   padding variants, zstd frames with and without content size, checksums,
   skippable and concatenated frames, the 1 MB limit, dictionary and legacy
   headers. The corpus records the current behaviour as-is, including the panic
   on empty payloads.
 - <csr-id-4fb06c727782e47c94f6891a4d4ad9e41de2e836/> consensus fixture pinning gas, traces, receipts and trace hashes
   Adds tests/consensus_fixture_{signet,mainnet}.rs, which drive a fixed workload
   (test_utils/data/consensus_fixture/Fixture.sol plus pre-signed legacy txs) through
   the engine and snapshot every block's trace hash, receipts, logs, blooms and full
   call traces with insta. Signet covers CANCUN, the PRAGUE activation and RLP tx
   hashes; mainnet covers CANCUN with signing-hash tx hashes.
   
   The workload covers SSTORE/transient/memory/log opcodes, nested and failing
   frames, CREATE/CREATE2 collisions, oversized code, SELFDESTRUCT, EIP-7623
   calldata floor, every Ethereum and BRC20 precompile (including rejected inputs,
   which abort the whole transaction), BRC20 deposit/withdraw, and brc20_transact
   with nonce gaps, a high-s signature, a wrong chain id and malformed bytes.
   
   Inputs are pre-encoded by build.sh so the workload does not depend on the
   crates under test. Any snapshot diff is a consensus change.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 19 commits contributed to the release.
 - 18 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 4 unique issues were worked on: [#21](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/21), [#22](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/22), [#23](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/23), [#24](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/24)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#21](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/21)**
    - Embed the module in an indexer's process with Brc20Prog ([`c550274`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c550274239bd758e4710d6cd0b480335c0502de5))
 * **[#22](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/22)**
    - Reject empty inscription payloads instead of panicking ([`8ce5e4f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8ce5e4f53986d187b3210d1d1530676b74ebd26a))
 * **[#23](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/23)**
    - Split the README into docs/ pages, add Differences from Ethereum ([`4c01317`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4c013176eb55996a3ea4227b2084421216005d88))
 * **[#24](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/24)**
    - Return range query results in key order ([`c5242a4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c5242a4bbd2fc83eb65170e85118102a32176748))
 * **Uncategorized**
    - Release brc20-prog v0.16.5 ([`4746dc6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4746dc68b5321a193411b6703210d84019b0d7f5))
    - Regenerate the public API snapshot on top of Brc20Prog ([`c4ee129`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c4ee12940d1ba6a2022fb36c86188f5f80310dd7))
    - Run each Bitcoin RPC network in its own binary, refresh expected gas ([`b31ec9c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b31ec9ccfa6d9c7c8b3f509379177e38f9992e38))
    - Pin a newer nightly for the public API snapshot ([`5ac1b9f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5ac1b9f6c671a6beb9d826a675d9eb3787d207e7))
    - Upgrade to revm 43, revm-inspectors 0.44 and alloy 2 ([`c2cdfe9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c2cdfe98524ef2e88bca21ced4c9e6cd102a46a7))
    - Rustfmt the consensus fixture and inscription corpus test ([`91e0519`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/91e0519f76dcdf00cad25d7d9fb844157bcf9ff0))
    - Pin inactive opcodes and post-selfdestruct state ([`ad6a6ca`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ad6a6caef8a4adccf9ead3e4e0dcc7f8c4b471cf))
    - Cover logs emitted before top-level and nested failures ([`0bef1d7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0bef1d7cca5204ac4ba529373c68cccb13e5532a))
    - Cover precompiles reached by DELEGATECALL/CALLCODE and top-level calls ([`cd21cb3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cd21cb389d61fd8fc7fc700d9c741f30cb4bae3a))
    - Cover precompile out-of-gas and the EIP-7823 modexp limit ([`d3f8f5c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d3f8f5c33a288d7a55e68ce14dd34a82b998ceaf))
    - Pin raw header and receipt RLP in the consensus fixture ([`878dba5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/878dba55ba080ecd9d093bc007a666df283789e7))
    - Bump non-EVM dependencies to their latest majors ([`f930fc5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f930fc57c86f3eaa11366f9b496295797f976372))
    - Update dependencies within their semver ranges ([`b6c8bef`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b6c8bef08f88fae99fe189bed88ef9e045d22e10))
    - Pin inscription data decoding over a base64/nada/zstd corpus ([`ecdeb17`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ecdeb17fcafbfc10a5594eb6c4eeaef902bbdb09))
    - Consensus fixture pinning gas, traces, receipts and trace hashes ([`4fb06c7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4fb06c727782e47c94f6891a4d4ad9e41de2e836))
</details>

<csr-unknown>
The README keeps the description, quick start (build, Docker), aconfiguration pointer, a documentation index and the license. Thedetailed material lives under docs/:The crate docs link to the JSON-RPC references in docs/.New alloy header fields (block_access_list_hash, slot_number) stay None; a rawblock written by alloy 1.x round-trips byte for byte (new unit test).debug_traceTransaction reports some frame errors in revm’s display form(“contract address collision” for CreateCollision, “return data out ofbounds” for OutOfOffset, “invalid opcode” for NotActivated). The error fieldis not part of the trace hash. embed the module in an indexer’s process with Brc20ProgBrc20Prog opens the database and exposes every JSON-RPC method as an async method of Brc20ProgApiServer, so an indexer written in Rust calls the module directly instead of over HTTP and JSON. Clones share one engine; start_public_rpc_server serves the same instance to other clients and rejects the indexer methods for every caller. start() opens a Brc20Prog and serves it as before.<csr-unknown/>

## v0.16.4 (2026-10-26)

### Bug Fixes

 - <csr-id-1b11be6e2a911315059bc9901cd324de683737ee/> gate bip322 v0.0.11 by block height on signet
   Signet blocks were verified with the v0.0.11 rules from height 0, so
   re-indexing signet produced trace hashes that differ from the ones the
   network reported with the v0.0.10 rules (first at block 267703). Signet
   now switches to v0.0.11 at 328057, the block after the current tip, so
   all reported signet history is reproduced unchanged.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#20](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/20)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#20](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/20)**
    - Gate bip322 v0.0.11 by block height on signet ([`1b11be6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1b11be6e2a911315059bc9901cd324de683737ee))
 * **Uncategorized**
    - Release brc20-prog v0.16.4 ([`b12986e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b12986e3fc522868c5df2fa17af76100ec308b3c))
</details>

## v0.16.3 (2026-10-23)

### New Features

 - <csr-id-7dd66d1767e93f5864287249b364209f2ad3714a/> serve indexer methods on a separate admin listener (BRC20_PROG_ADMIN_RPC_SERVER_URL)
   Adds the BRC20_PROG_ADMIN_RPC_SERVER_URL env var and Brc20ProgConfig::brc20_prog_admin_rpc_server_url field. When set, the public listener rejects the indexer methods for every caller, including callers with valid credentials, and the admin listener serves every method behind the configured authentication. When unset, a single listener serves every method as before.
   
   The admin listener disallows cross-origin requests and logs a warning when bound to a non-loopback address. start() returns one ServerHandle that stops both listeners.

### Bug Fixes

 - <csr-id-abf894759f53568f08279e2123f2274ba8246e7b/> bump yanked keccak to 0.1.6 and clear build warnings
   Remove the unused zstd_safe::WriteBuf import whose as_slice collided
   with a future std method, and the deprecated std::usize import.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 3 calendar days.
 - 7 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#14](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/14), [#15](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/15)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#14](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/14)**
    - Bump yanked keccak to 0.1.6 and clear build warnings ([`abf8947`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/abf894759f53568f08279e2123f2274ba8246e7b))
 * **[#15](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/15)**
    - Serve indexer methods on a separate admin listener (BRC20_PROG_ADMIN_RPC_SERVER_URL) ([`7dd66d1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7dd66d1767e93f5864287249b364209f2ad3714a))
 * **Uncategorized**
    - Release brc20-prog v0.16.3 ([`47d4ae4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/47d4ae42fec53d753bc77ed48870be0f01f1cf4c))
</details>

## v0.16.2 (2026-10-15)

### New Features

 - <csr-id-3dada205675cbb5ce521ddf1d6a97c07e1807169/> configurable eth_getLogs block cap (GET_LOGS_MAX_BLOCKS, default 6)
   Adds the GET_LOGS_MAX_BLOCKS env var and Brc20ProgConfig::get_logs_max_blocks
   field: the number of blocks one eth_getLogs call may span, inclusive. The
   default of 6 keeps the previous behavior; 0 or an unparsable value falls back
   to the default.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#13](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/13)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#13](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/13)**
    - Configurable eth_getLogs block cap (GET_LOGS_MAX_BLOCKS, default 6) ([`3dada20`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3dada205675cbb5ce521ddf1d6a97c07e1807169))
 * **Uncategorized**
    - Release brc20-prog v0.16.2 ([`b6af9fa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b6af9fac0717e723f32e0549358d075234626625))
</details>

## v0.16.1 (2026-10-12)

<csr-id-4c9e63e1d1311c9046b168bc860a73c4144ca62f/>

### Bug Fixes

 - <csr-id-46ebe8f39cc04431e9632014bd2b8213e33316a0/> Gate bip322 v0.0.11 verification rules by block height on mainnet
   bip322 v0.0.11 accepts proofs that v0.0.10 rejects (short DER ECDSA
   signatures, P2PKH, P2WSH/P2SH multisig). Mainnet blocks executed before
   the v0.16.0 upgrade were verified with v0.0.10, so a fresh sync with
   v0.16.0 produced different traces, first at block 913183 where a
   P2SH-P2WPKH proof carries a 69-byte signature.
   
   The BIP322 precompile now verifies with the v0.0.10 rules below mainnet
   height 960847 (first block indexed by v0.16.0 on the reference node) and
   with bip322 v0.0.11 from that height on. The v0.0.10 verify path is
   vendored as bip322_legacy since the crate version is yanked.

### Other

 - <csr-id-4c9e63e1d1311c9046b168bc860a73c4144ca62f/> drive latest tag via latest=auto, drop dead is_default_branch line
   The `type=raw,value=latest,enable={{is_default_branch}}` line never fired:
   docker-publish runs only on tag pushes, where {{is_default_branch}} is
   always false. `latest` was actually emitted by metadata-action's default
   `latest=auto` flavor on non-prerelease semver tags, so behavior was correct
   but the line was misleading dead code (flagged in review of #10).
   
   Make it explicit with `flavor: latest=auto`, which also correctly keeps
   `latest` off prerelease tags (e.g. v1.2.3-rc.1).

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 70 calendar days.
 - 70 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#11](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/11), [#12](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/12)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#11](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/11)**
    - Drive latest tag via latest=auto, drop dead is_default_branch line ([`4c9e63e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4c9e63e1d1311c9046b168bc860a73c4144ca62f))
 * **[#12](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/12)**
    - Gate bip322 v0.0.11 verification rules by block height on mainnet ([`46ebe8f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/46ebe8f39cc04431e9632014bd2b8213e33316a0))
 * **Uncategorized**
    - Release brc20-prog v0.16.1 ([`8ae880a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8ae880a8994a76c920c76f9a137529bd4487dedb))
</details>

## v0.16.0 (2026-08-03)

<csr-id-b0d89c889b60ba9d795beb24f71366a874a2c951/>
<csr-id-4b9eaf7329012eae0233b553b85ebcf069d04ae6/>
<csr-id-970a4f5fdfe22a66f70e86cfd7f329cb4a98c1d0/>
<csr-id-feed95e42977b71315d5b4e4789c82074ef3aaef/>
<csr-id-bb3d7cd23dbb679265464b1143fec786332a6aba/>
<csr-id-15b415064506ece80e623ca56723125d16fc29f2/>
<csr-id-fdeb14ed7ae4b25f060ba938b51e6f7323d411b9/>
<csr-id-f5619cb44f0bdf537cdb3ee06934eeefeb92b6d9/>
<csr-id-2e6a84684a38b18f7aca7a3eb7d3780b62ba68a9/>
<csr-id-e2deec2661cfc1c500f43ff356d8d25fca9afc89/>
<csr-id-f9e87704b189773605876ab92b46bd374d474107/>
<csr-id-d5edb5a9cbfeb2d094889ebbc9fe04e5686ee6c3/>
<csr-id-8a163e075c13c78ecab560ef51e2c8dc1e1e3472/>
<csr-id-495186d6cce4bf8ec92f4408d995bdcc1de8dc66/>
<csr-id-4a894c6c04c7f01917fa614f51221ad3848bdb92/>
<csr-id-3de713dc4c78562a7cb290bf292cde66d0642575/>
<csr-id-492065ff749fd418983fb51eb4f1b54a80eb1dc7/>
<csr-id-6fa1bf4e150151996a88af5310695a68fe33c11f/>
<csr-id-0b0ae25ebc1308475b19d0b72967acd5b69b88fa/>
<csr-id-2ffeb5b5f2c5046b85d463fc9eb46ee51cb791d3/>
<csr-id-c38c7b06ad3a2067872e64071f9362d00eefc4be/>
<csr-id-c9178ef3b704b02404b6b30477d4e01f5f54339c/>
<csr-id-f268486b9584f377395fbbc940a12783505af471/>
<csr-id-834e75c56bd35c27fdd439837f3c3392f3f55560/>
<csr-id-d00d3c2771628c06dec713fc6c6cd18c0f326ca9/>
<csr-id-3c088b9a8e328e344c81df24a30f4835ca7ebb08/>
<csr-id-ce01730b9951c1ca3cfe01e331b65a54c2b3af07/>
<csr-id-7347175402e12c2a9413a98a26ab09c466ae86e3/>
<csr-id-5a85fd8596f0513ae78927ec3578375a3ff02edd/>
<csr-id-663c929ae910af58eb32788f8f3620d22c29bc14/>
<csr-id-b1d3f3a2f973e94acad4cc1a5d241fed50f4058f/>
<csr-id-68dfbed0c7dc9e30d4b2dc93d57d176419f12762/>
<csr-id-74341ad293809caae1583bc7835d8c2a7e097b13/>
<csr-id-c57ac662947c524bcda68a1c456b227d6980a431/>
<csr-id-0ffa6304546d3103ef94ac653104847c1273d348/>
<csr-id-32162127551206dd663df412d4fdd5ff8b3a2246/>
<csr-id-0ae78d41a0688a76b363eca883b28b8456c7f6c3/>
<csr-id-c0051151b20ab1002b9e90287b6ac3cd0f7ed431/>
<csr-id-cec0ccb7de02ba0e3ab76c1bcb41daedd8a5a516/>
<csr-id-f537928db53a9581b4321edca7dff67703ffb473/>
<csr-id-4d98b447a20d2cb0060e2da66924f20536bdcac9/>
<csr-id-45b1f50610a1d26f1f035fc0a481542ffea88d8e/>
<csr-id-ceb5be8416a21165ab3df9f1073aa47d5e154fc0/>
<csr-id-90862fbb3ba4e7c5a08b65ac3967e25a71628445/>
<csr-id-6a93ee05efab15719cf7547461be3996834c80e5/>
<csr-id-8c401a9b1c1b2ffce878a04b6ccc06849a189271/>
<csr-id-1b8f1f5a6b63490636fd66c3c47c99137b1b2d67/>
<csr-id-7977ab01c9e77e632b7e706d1e65afbd48783d0c/>
<csr-id-be926645f55fb2f2b53351fc1ee7006ab405ba49/>
<csr-id-0c8fcca035b00524e1a471ce34e6d89a6ba9276f/>
<csr-id-795598b52862a308b83628f10358765b8c55e354/>
<csr-id-4cf9d212069459292af08297aa100fa9c3fc616a/>
<csr-id-d1968eed7ec8dab87f52c590524107e1d1d1906b/>
<csr-id-3163cd4f42e4ba5c5e3e66719101e9ae809b5402/>
<csr-id-3bd0d87ad463857150a4e0a9f1df92f68d8818e3/>
<csr-id-8d5d7b8a280ce7d744a448a332fb3f48cc08a823/>
<csr-id-7e22c4554510615cf46bdaa2fe2830e686eeee48/>
<csr-id-41daefdd27df528f496bc14fbaa6f7470ddfe578/>
<csr-id-42efe3e72360a946974daabf3d46fdb497887b24/>
<csr-id-6bcbe483599ad3a6eb6d459927e5fa9ff2da4309/>
<csr-id-efc0083fca2c77cebb4cbef05ebfe4f1cd61dcb8/>
<csr-id-bf970685dd9d5e0e5698cdd358adb641a74a492f/>
<csr-id-26f7b5f16e23912dd3458c27e4d7aa31e6f1d93c/>
<csr-id-5181b9c28c5fd07033977bbd5da1a3c86401df4a/>
<csr-id-1a5ae117cade1814280651f8b9438215b91b4953/>
<csr-id-843c4646b73ed3e7f66064a06fa816eed401df58/>
<csr-id-75d6a133af28a90fceaf9e9a43a83b463b96c71e/>
<csr-id-f1570b4da72e1bcd8ba00a271b5c56d3e2b16cee/>
<csr-id-4cff9941c50e43fea647630f81834644234a1465/>
<csr-id-add636a06c7accbdecd2c6375a0b5c5bcde01492/>
<csr-id-1ae91676a99b5c5733276dfdd4776e229c3ed34b/>
<csr-id-15e16503608425a7d9af0ba0752a171f9c11e1c0/>
<csr-id-8c19cd08292bf568d3209c72f2fa160262e95063/>
<csr-id-f2f18c4bcd1dd654d8a67d4c0ba9cab587e7f2c6/>
<csr-id-c4efb097b7a55b973632faf1b09c6252f292ccfc/>
<csr-id-912dfefc8d93a0ad39d1d788acc93083a0907f8d/>
<csr-id-fb475d44cbed75d3254061e25cf0ff6fde238e88/>
<csr-id-4045ddbfa1c736529daa6f2fc79ba9482716f0df/>
<csr-id-c02d3714fc22b376d0a1913f7f5806f65618de53/>
<csr-id-67f5b48efbf985abdb2138442c3deae2b71c88da/>
<csr-id-1ca98dcd0c689200218c297c38cec2d4ce29c3c3/>
<csr-id-ac703fd54431b45f481e401129b648a0f3561d0f/>
<csr-id-48a52bd4f4db178f9095958ad6b0576158225709/>
<csr-id-0eae24b7bd1b6c742853822adacd89a6c2793f8e/>
<csr-id-e99469e09cfebb59fb6cbd4b113c147b1eec5c1c/>
<csr-id-234d87431db12d60464dce099fbe22deaf079dec/>
<csr-id-2dc9d3a795e089783949c5943082f67726d88262/>
<csr-id-b6446739a51fa5c440209dd7687299ed939ff911/>
<csr-id-ca651492a512c47ebc8345851cd9118d76465ec1/>
<csr-id-3e9e395c7bac26cb3a747cb2fc35d2bb1586eec8/>
<csr-id-b46ec70af7120595c547a032772c7f656e5594bf/>
<csr-id-4472824b035db9c6822cf06fe4fb2bed824548e3/>
<csr-id-3af32341c61b424b28838c26765c31a1188cb006/>
<csr-id-34b6d4188ad8f9b5581b74ffbb87d563adc3ae35/>
<csr-id-c37b2ef18f4bf9c6263d0a95d086d52cf6a6bdb4/>
<csr-id-fb09f2751ad1c46d89c93545eb2fab83dac72dd1/>
<csr-id-d40c627fe60c2d8da9d57497801dab26977afa91/>
<csr-id-a8ceb606d3456a24221b1345ad329596944888cf/>
<csr-id-2166b8bc70d1c5eeb6b3bdf34d6437fbb16c42f0/>
<csr-id-b4ea3f2f9f7ec6e855fd087da84b2e128142ac62/>
<csr-id-3a82185ae1ee37bbd072db2f2d45d979bf7c8d02/>
<csr-id-1e8c50cdba837afada7c1f2ea7345507686ff2d7/>
<csr-id-541d41c7c3e1c104364c2f24e250f2a8eb6da45f/>

### Chore

 - <csr-id-b0d89c889b60ba9d795beb24f71366a874a2c951/> update dependencies and tests for bip322
 - <csr-id-4b9eaf7329012eae0233b553b85ebcf069d04ae6/> Update API
 - <csr-id-970a4f5fdfe22a66f70e86cfd7f329cb4a98c1d0/> Update dependencies and add new packages
   - Updated `alloy` to version 1.1.2 and related packages to their latest versions.
   - Updated `hyper` to version 1.8.1 and `jsonrpsee` to version 0.26.0.
   - Updated `revm` and related packages to their latest versions.
   - Updated `criterion` and `insta` to their latest versions.
   - Updated `tower-http`, `tracing`, and `tracing-subscriber` to their latest versions.
   - Updated `test-utils` dependencies to match the main project.
 - <csr-id-feed95e42977b71315d5b4e4789c82074ef3aaef/> cargo fmt
 - <csr-id-bb3d7cd23dbb679265464b1143fec786332a6aba/> add release configuration file for versioning and changelog generation
 - <csr-id-15b415064506ece80e623ca56723125d16fc29f2/> Release brc20-prog version 0.11.5
 - <csr-id-fdeb14ed7ae4b25f060ba938b51e6f7323d411b9/> cargo fmt
 - <csr-id-f5619cb44f0bdf537cdb3ee06934eeefeb92b6d9/> Cargo fmt
 - <csr-id-2e6a84684a38b18f7aca7a3eb7d3780b62ba68a9/> Run cargo fmt
 - <csr-id-e2deec2661cfc1c500f43ff356d8d25fca9afc89/> Bump version to 0.6.2
 - <csr-id-f9e87704b189773605876ab92b46bd374d474107/> Update package metadata in Cargo.toml
 - <csr-id-d5edb5a9cbfeb2d094889ebbc9fe04e5686ee6c3/> Move documentation for Brc20ProgApi to top-level module
 - <csr-id-8a163e075c13c78ecab560ef51e2c8dc1e1e3472/> Move super:: imports to crates
 - <csr-id-495186d6cce4bf8ec92f4408d995bdcc1de8dc66/> Run cargo fmt
 - <csr-id-4a894c6c04c7f01917fa614f51221ad3848bdb92/> Update dependencies and update RPC authorization middleware to latest jsonrpsee library
 - <csr-id-3de713dc4c78562a7cb290bf292cde66d0642575/> Bump version of brc20_prog to 0.2.0
 - <csr-id-492065ff749fd418983fb51eb4f1b54a80eb1dc7/> Bump version of brc20_prog to 0.1.1 in Cargo files

### Documentation

 - <csr-id-af79d9c3eeb635e8f6f7d5db667eee0fc3525229/> Add description for txpool_content method in README.md
 - <csr-id-b29317f79b03094c39b0ab8aad5dd6a7f1ca72ca/> Update brc20_deploy, brc20_call, brc20_transact method parameters to include op_return_tx_id field
 - <csr-id-1c437ff3a7ab18c318c386ffa9cffae1d404c2cf/> Simplify cargo run command in README
   Removed the 'server' feature requirement from the run command, as it's now enabled by default
 - <csr-id-86edafc539dc820771c498b1e90ac20816f903e5/> Add warning about ticker case sensitivity and format in README
 - <csr-id-5d83dd1f8037d1a35e4375be25e0f60e8b772fdf/> Add caution note about basic auth and secure connections in README
 - <csr-id-4070f7fe4c6130fc543e4fc932c6c19fddd0ded4/> Remove warning about zstd compression stack size requirement from README, as it now uses Vec instead of a fixed size slice
 - <csr-id-774f3db79454f4780242f0a42bd05c80d0f02a18/> Update README to include note about BRC20 indexer integration and enhance operation checks for deploy/call inscriptions
 - <csr-id-988c23983cb0ba584ba1cbdc2b3783a3901c1aef/> Update module documentation for clarity and completeness
 - <csr-id-25afd999173c616524fca02a8fa41136d911ce4d/> update README with warnings about BRC20 balance server and BIP322_Verifier contract limitations

### New Features

<csr-id-8d7378828b9a897da415825edf5cb0e1d44845ca/>
<csr-id-c99264adc38f835af524bc1653a4e36af9f2ae60/>
<csr-id-a3b5e0b6246b0a02c68befab79a3fb7f40996ca0/>
<csr-id-cd803cceca8cf411838270a72c36a589e66664c5/>
<csr-id-508174591e52e23f746feb13f2622f86b931c5bd/>
<csr-id-831b9b604631f37838206fb1931eed33b8aae0a8/>
<csr-id-9fcaaff1e53ac7575c569ca321447ae5cc1b574c/>
<csr-id-4550e0eff6aa70d3cfa39ba7b378e2eccf5cd7e8/>
<csr-id-e36845347bd58ed8eb5b3448c5d05ce01edabc26/>
<csr-id-ca2b8647be71df9ed2bae0f0318c14c8c1b050bf/>
<csr-id-5a4019ca24d982d12e2c7c96e64d313ea8410b27/>
<csr-id-ad4bb4b6c903c9911dba0e118bb01c0eb8d139e0/>
<csr-id-f2e7d2791fd6acd9b360a2dea3d1c1898c4e5cb9/>
<csr-id-a15371e4737bb0b90fa6e9f8c401233dfe068fcf/>
<csr-id-29fbf41684a93dd86c0e71cf1a63a78e84af6d8d/>
<csr-id-a575638e8fd2e0d510ad2c8854e7b469a5401429/>
<csr-id-af4cea8e78254bb21a6cd28221a231c76db41626/>
<csr-id-443887a631eb1b67eb8c72d189fe63a9323070cd/>
<csr-id-4727a3378ab471e70db5d3ad4af25eee5583233b/>
<csr-id-a6dba4b05c2a3e905462104c8ca40dec9d172178/>
<csr-id-2222ed3240697ed3a40deec004a135a6b6ef071f/>
<csr-id-69af83effd6c6931d1e52eef6c29d297e3c6d026/>
<csr-id-8e835b8be3ffd9407759d1315dbc8bae003f8d08/>
<csr-id-b27ea7c72a864e0febec413b25e03376099f81f6/>
<csr-id-bbab3bfe343afb171cfb3aedae5a9f1f2e88cb57/>
<csr-id-3f7d34653a2719602b2290a710b98125e17ca87c/>
<csr-id-b1753a6a41d82177ba13338a320e6c323a1f3134/>
<csr-id-417fb86602b76e502f0d65cfb4332358e9010be5/>
<csr-id-fe23eb3e01bf6eef40ce253a2861ea3863b31f9d/>
<csr-id-e697503ac501249fac67d6e440d52dfef9f87add/>
<csr-id-7e0fb6fe2327866ff0802ebab058033b84a9e8f0/>
<csr-id-7f38574d482b9e33cd2cd40439b854a4f0e6abc0/>
<csr-id-1408102192951ba007c62352a92fb9ac6ba0b040/>
<csr-id-752f5f04083e163aa047756979c5130528abc14c/>
<csr-id-2b5311f795784b4e0382313e3bcf19a0a3f8d109/>
<csr-id-4b36434cac54d7548fe5e8af8b87ee65b70f8cd7/>
<csr-id-180b24e9ca16500d239f19c9c0749c9eeda0de3a/>
<csr-id-ee9d7b786052253cd9ecfdc413fbec425a06a789/>
<csr-id-73784c2dee9571e1fec735684b77911d59ec3aae/>
<csr-id-401fb70a2d46aaf96f5b52906e466ab16c30cf0c/>
<csr-id-c2f23df1d127f6268d804b778e30a14274b97572/>
<csr-id-654c8e282cd18e7b459e940e879902a10281f3a9/>
<csr-id-1324f623bd9593c9e2b2a8e8a0e6b3ef66b0b484/>
<csr-id-0e96a89cbe90f63fd9fc7b56bb22dba2bfd2ba00/>
<csr-id-97b2d6623c187a2889c06ac9c00a45d49f9a6142/>
<csr-id-ea2bd18608aa9d430996d56aa2ffc803b5f4db51/>
<csr-id-000db4966fec5593798b50b0b19829a665ca2e18/>
<csr-id-d17c4f651213e2c0cb9fac829021352fa4b1f43f/>
<csr-id-d8d93bb628e863e3b4fc629a100b1c900c7dac8d/>
<csr-id-6e58ca17a66ad7dffd8540b3cdfb7bc76391f959/>
<csr-id-b0a754c1f0341e7fc569a942cfb9cd8e56d227fb/>
<csr-id-f7e2a7bdf5054d951d2b08d6cfe97f51ee0dde79/>
<csr-id-eb03c031f83b85693173b789f0e0bfced0c3aaae/>
<csr-id-fd9d875ba40a6038a25d225a2604aeb63b0ad41d/>

 - <csr-id-f29eadb206452bf0ce43ac831a109289e04ed3e1/> eth_call* and eth_estimateGas* methods now wait for block finalisation instead of failing when they are called mid-block
   * feat: Wait for block finalisation on eth_call* methods
* test: Update mid-block read contract test to wait for block finalisation
* fix: Notify waiters when clearing caches to prevent indefinite waiting, and fail gas estimation if block height change during estimation
- Updated dependencies in Cargo.toml to their latest versions, including:
     - alloy to 1.1.0
     - revm to 31.0.0
     - serde to 1.0.228
     - tokio to 1.48.0
     - criterion to 0.7.0
     - tempfile to 3.23.0
   - revm to 31.0.0
   - serde to 1.0.228
   - tokio to 1.48.0
   - criterion to 0.7.0
   - tempfile to 3.23.0
   - revm to 31.0.0
   - serde to 1.0.228
   - tokio to 1.48.0
   - criterion to 0.7.0
   - tempfile to 3.23.0
   - revm to 31.0.0
   - serde to 1.0.228
   - tokio to 1.48.0
   - criterion to 0.7.0
   - tempfile to 3.23.0
   - revm to 31.0.0
   - serde to 1.0.228
   - tokio to 1.48.0
   - criterion to 0.7.0
   - tempfile to 3.23.0
   - revm to 31.0.0
   - serde to 1.0.228
   - tokio to 1.48.0
   - criterion to 0.7.0
   - tempfile to 3.23.0
 - revm to 31.0.0
 - serde to 1.0.228
 - tokio to 1.48.0
 - criterion to 0.7.0
 - tempfile to 3.23.0
 - revm to 31.0.0
 - serde to 1.0.228
 - tokio to 1.48.0
 - criterion to 0.7.0
 - tempfile to 3.23.0
 - revm to 31.0.0
 - serde to 1.0.228
 - tokio to 1.48.0
 - criterion to 0.7.0
 - tempfile to 3.23.0
 - revm to 31.0.0
 - serde to 1.0.228
 - tokio to 1.48.0
 - criterion to 0.7.0
 - tempfile to 3.23.0

### Bug Fixes

<csr-id-fa00152f808aac0efa78ca9b5341772c997cd3a3/>
<csr-id-e0c605adaa08f043375fe769266e865d3d30139b/>
<csr-id-4c6521b6d5d1c4d1121a6b59afde39be4dee0be6/>
<csr-id-ce4d985887280d0b82ce753643b11a870feb6300/>
<csr-id-a502861740265f47c16151455a6c238bec772ec3/>
<csr-id-53c126da6e25e06e11baa9ee56c491243ad47e42/>
<csr-id-b4715160611dcb7d9a30eeb53661f51f3558571a/>
<csr-id-85bc09e2b466b1a703fdaf0fae8c60396915ac7a/>
<csr-id-b0ab81e0976404338a101b9128c2c8dcb9e21061/>
<csr-id-90c5477443dae4cbc5430cbf57b77c08412d07a4/>
<csr-id-4884c89c216c88a2849f72720d44e4b425c17989/>
<csr-id-93026918f09b5f3f5561183bdd231c6f3ea7aad8/>
<csr-id-78aad64571cb7bf8730d283b86530d8139cfc241/>
<csr-id-eab61361dd7cb73aafc36e6498703f2851dcc8aa/>
<csr-id-b8ec391975910cd83d4b1c9d87d4315840b04ec7/>
<csr-id-846099bf4e941abcc37177d9250e04344ecac2c5/>
<csr-id-49b03e4c0564f195d311e439b501077196d72d3c/>
<csr-id-25550c2bcf34f50731f1d16499909b32684ed3ea/>
<csr-id-753be5a770a7f2497a67de3b4da95cb97228dd87/>
<csr-id-1eee9df5682cb19454d45ee1c5ee43fbc5832d1c/>
<csr-id-98bb8ffa89b09929aab08d43ea771523f340f011/>
<csr-id-b5f82f0716d7ee3a3588eef1a132ef36875e28ef/>
<csr-id-4df9d976d42b7894dc52bb56850ae75cf5a0c25e/>
<csr-id-a462db4a4df63f5465a9ed48ad12c357cf0c7027/>
<csr-id-c7f75c87a4370cfb9afdd3da518d5a8f8892539f/>
<csr-id-7cfc17b10315a19f6b09edeed4621423f67162ea/>
<csr-id-391139c6c3ff02efcb029befa82c54a8370f6eb9/>
<csr-id-ae96ee4b209d45ab1b0ddf7422fba081cda81e76/>
<csr-id-a5ef831d20cb19145c9c847f42f930d8eff88e46/>
<csr-id-19b4d147c4143e278744e47681ae6ada486b735e/>
<csr-id-96f0012238e70a6197f77f5a858cd0266a9f4a4b/>
<csr-id-35125ede4e7d1ba716c29dd5d7c18a182566154a/>
<csr-id-829efa531ee062b3149e9a85f3fc4aac34ccd0cb/>
<csr-id-c2317a22aff1f61e3507e48e7f91e90f4547f0bc/>
<csr-id-c2cb4834fa8aeb80ec384573f2f8fe55cab41fbf/>
<csr-id-5e9bf0f04fc3662f914c788cbaa6f4ba5deca2d0/>
<csr-id-66495a5c9cb72db613765e3e8abcefafddd966ae/>
<csr-id-1a56564a185f301dceb4bcb2c8fe82eb5dd9a8b8/>
<csr-id-dd53a43ad2b938ed3d19c3842d9edbcc2f222963/>
<csr-id-657a99b18db4041802e19c76e118270016b18f37/>
<csr-id-bbd9669f3163af5932dccb9dab81e3fc0fd1671a/>
<csr-id-17aa2a93f5059f62b4fc590e11433dde712425ca/>
<csr-id-19032982d0af197ea6aefd64f710c9888696aa81/>
<csr-id-09d4de4b3e5986c7732e60cc4d794cf4eac61344/>
<csr-id-5c0b892a5acc399cdd85ba59e2e2b98f94d5672b/>

 - <csr-id-befe688ca35255a78264e55018a64283144dae2c/> update nonce retrieval to use get_account_nonce for pending transactions
 - <csr-id-8b248c929f36cdfb3ae403e68ae01eb66884c4f2/> Update eth_get_storage_at to return full length hex-encoded storage values
 - <csr-id-0896feb3038bed3f919ad150feb15a9904309328/> Update expected gas response values in test cases
 - <csr-id-228b912f377eb86a6752bdb836a1082e5868be75/> keep midpoint instead of gas_used * 2 as upper bound to cover some edge cases
 - <csr-id-790d806212e4b6067c857d570b2a1d25b68ba31d/> Clean up properly if transact_one(tx) returns an error
 - <csr-id-c22a9454698e55c80c96d68bc93af1c86fa7a9f9/> Clean up properly if transact_one(tx) returns an error
 - <csr-id-a3990352207838aedd0315d8d45007ba250561cb/> Add gas usage assertions in deploy_call, precompiles, and transact tests
 - <csr-id-660e8c009bf291bc87abb0be971f1797237d7c2d/> Handle None case in get_info_from_raw_tx and update related logic, invalid chain ID shouldn't return error, and just ignore the transaction
 - <csr-id-039b21520a786f58039cdfbb8a7d936790045986/> Sort transactions by index in Brc20ProgDatabase
 - <csr-id-8679b42bb119fc0196d90916cab563111198d637/> Update comment to clarify placeholder usage for block hash in get_transaction_and_block_hash_with_overrides
 - <csr-id-9680ce28ccfa7a8b25da93e2760bf431188bc4dc/> Rename getBtcTxId to getTxId for consistency in function naming
 - <csr-id-ca33922f02381a0f3f791f0226aefbccae1181ef/> Update public API (serde -> serde_core)
 - <csr-id-32097286b9fd554d0221054fa8a1c3d3a76e0506/> ensure pending transactions are committed, cleared, and reorganized in Brc20ProgDatabase
 - <csr-id-57a447d39515ec197bf7a865db07aac079d83d6e/> allow "mainnet" and "bitcoin" to be used interchangeably
 - <csr-id-3e44cc0b239ef2c7c0c45bc2cc28a30620204f37/> update generate_block to accept block_hash as a parameter
 - <csr-id-33b4ec2b72315e749f1c2c182df423f21c71231b/> reorder block hash setting to avoid race conditions
 - <csr-id-467fd9ab4851d817c12c927d65487317566e86ec/> revert version bump, this is automated now
 - <csr-id-e6846948651b542114479828e606077ee305eced/> bump version to 0.13.0 in Cargo.toml
 - <csr-id-b4ac41b640fc17c3186cdc551669bde7da64e99c/> serialize transaction type into hex
 - <csr-id-c0b38c05afe2fe20ce10e2de9a3748bf02d09c94/> update homepage URL in Cargo.toml
 - <csr-id-2a3adbde258ad0ea873b20f105c9fc842fb17ee6/> update pre-release hook to correctly format CHANGELOG.md generation
 - <csr-id-746773dfbbfedb3bc7f672476fe9a844ee1993c6/> update pre-release hook to correctly generate and add CHANGELOG.md
 - <csr-id-a238d89cf77893009e8d7abcf8f4d38ddfbd2400/> update read method signature to use lifetime annotation
 - <csr-id-27722d35d3f4e778033fcd560646b0e9aca88c64/> update read method signature to use lifetime annotation
 - <csr-id-4e6f326190cc4c834d0c50a3f9cf352dac544a5a/> remove test data files for BRC20 balance calls
 - <csr-id-72780b60f720eb5edbc1ee43e02d8cb6a3519a23/> bump version to 0.11.4
 - <csr-id-ef1b4cb5c9dfd4b71e4664af9df0d3dd24c3b824/> remove flag from RawBytes#value() function
 - <csr-id-4719fe75f051f7c7e563aae860c9807fe6541056/> add step to build with no default features in CI workflow
 - <csr-id-1eda8aded7a7a9ff96e84c0e534474048446ebed/> bump version to 0.11.3
 - <csr-id-d1b383b3554e713cf3e939ce6a041af7ef5f47fb/> update keccak256 import from revm to alloy
 - <csr-id-0dad5dd1dbc42afd0aaf6a60d8ca348ff9c54cd7/> bump version to 0.11.2
 - <csr-id-dc47f65ed9a16abe05aab7f64b76324583fb14a6/> update public_api snapshot
 - <csr-id-a764b65df6901c0bb735e0091f049e40e622c4c9/> Correct version string format in web3_client_version method
 - <csr-id-b249ec9be690164558d8d85379ab214ca7d190fa/> Improve error handling in contract address resolution
 - <csr-id-29aae3c00da1759ca03c231da0d81272f2dfc7fd/> Refactor Brc20ProgDatabase to remove redundant timestamp and gas methods, they are stored in the block database already
 - <csr-id-c3222bd1d37acc9346a832ed2af9e755f4deb817/> Migrate back the bip322 crate after the fix and remove the fork
 - <csr-id-56b4846140ed2120f90c7ba7ce7376e8d8b3ad3c/> Simplify reorg logic in BlockHistoryCache and UintED serialization/deserialization
 - <csr-id-f3fa71ff498f398a468816bc72ac73179966cc39/> Enhance bytecode handling with error checks during deserialization and decoding
 - <csr-id-ea9a1689aea1b33c7f07cfe67bf1f55290d109dd/> Update inconsistent docs
 - <csr-id-e28f5d5c96e4076a119dd5381549fb8f637fae76/> Fix test_reorg_all_blocks to reorg only 10 blocks
 - <csr-id-a1e1d8d72ad89ab77a81593eb1976c2116d1e30a/> Bump version of brc20-prog to 0.10.4
 - <csr-id-dbf69a74e9ff4b3bb9984092100626c873687c99/> Use a fork of bip322 for a patch where it crashes
 - <csr-id-123ad4e872bb4bf455ecfda84895734a79ff89a0/> Ensure latest value retrieval from the cache is never empty, and reorgs also panic if there's an attempt to empty the cache
 - <csr-id-57bf493852203b6bb6178f510d7c4abb9a0b8b36/> Update BITCOIN_RPC_URL to use address
 - <csr-id-f221717beefa9de50df5eac62ae6c390c0c195e6/> Replace "DB Error" with a static error message and panic instead of returning an error, as it's not recoverable
 - <csr-id-3d156588df34b9b79aaca65414411aeda1acd349/> Couple of bug fixes for reorg handling on database
   - Use get_next_block_height method in DatabaseCommit methods to fix an off-by-one error in reorgs, which might incorrectly keep old values

### Other

 - <csr-id-6fa1bf4e150151996a88af5310695a68fe33c11f/> Add changelog file (generated by cargo smart-release)
 - <csr-id-0b0ae25ebc1308475b19d0b72967acd5b69b88fa/> remove release configuration file and associated pre-release hooks to allow regeneration
 - <csr-id-2ffeb5b5f2c5046b85d463fc9eb46ee51cb791d3/> Add support for raw signed transaction inscriptions and update relevant methods in README

### Refactor

 - <csr-id-c38c7b06ad3a2067872e64071f9362d00eefc4be/> Do a first run on eth_estimateGasMany and eth_estimateGas for an early exit in case calls fail
 - <csr-id-c9178ef3b704b02404b6b30477d4e01f5f54339c/> change inscription_id and inscription_byte_len from Option to required fields in API and update database structures
 - <csr-id-f268486b9584f377395fbbc940a12783505af471/> remove legacy fields from TxReceiptED struct
 - <csr-id-834e75c56bd35c27fdd439837f3c3392f3f55560/> generate receipts for invalid transactions
 - <csr-id-d00d3c2771628c06dec713fc6c6cd18c0f326ca9/> Update public api and fix a visibility error
 - <csr-id-3c088b9a8e328e344c81df24a30f4835ca7ebb08/> Add configurable request/response size limits and batch request settings to RPC server
 - <csr-id-ce01730b9951c1ca3cfe01e331b65a54c2b3af07/> Remove batch request limit from RPC server configuration
 - <csr-id-7347175402e12c2a9413a98a26ab09c466ae86e3/> ditto, improve error message for Bitcoin RPC status check failure
 - <csr-id-5a85fd8596f0513ae78927ec3578375a3ff02edd/> Improve error handling in Bitcoin RPC status check
 - <csr-id-663c929ae910af58eb32788f8f3620d22c29bc14/> Rename DB_PATH to BRC20_PROG_DB_PATH for consistency in env.sample
 - <csr-id-b1d3f3a2f973e94acad4cc1a5d241fed50f4058f/> Enhance RPC server configuration with request/response size limits and batch request settings
 - <csr-id-68dfbed0c7dc9e30d4b2dc93d57d176419f12762/> Print the error message in get_brc20_balance function when it fails
 - <csr-id-74341ad293809caae1583bc7835d8c2a7e097b13/> Bump version to 0.11.1 and update binary name in Cargo.toml
 - <csr-id-c57ac662947c524bcda68a1c456b227d6980a431/> Update eth_getCode to return 0x if there's no code present for compliance
 - <csr-id-0ffa6304546d3103ef94ac653104847c1273d348/> Modify eth_call in RpcServer to return 0x or the revert reason for compliance
 - <csr-id-32162127551206dd663df412d4fdd5ff8b3a2246/> Add legacy fields back in BlockResponseED and TxReceiptED for backwards compatibility
 - <csr-id-0ae78d41a0688a76b363eca883b28b8456c7f6c3/> Add input size limit warning for bip322_verify_precompile to prevent excessive resource usage
 - <csr-id-c0051151b20ab1002b9e90287b6ac3cd0f7ed431/> Enhance ConfigDatabase by adding caching and implementing flush functionality
 - <csr-id-cec0ccb7de02ba0e3ab76c1bcb41daedd8a5a516/> Use catch_unwind for BRC20 balance server checks to handle potential panics gracefully
 - <csr-id-f537928db53a9581b4321edca7dff67703ffb473/> update public api
 - <csr-id-4d98b447a20d2cb0060e2da66924f20536bdcac9/> Simplify balance retrieval by removing error handling from get_brc20_balance function, it now panics when balance server is unreachable or provides invalid data instead of silencing it
 - <csr-id-45b1f50610a1d26f1f035fc0a481542ffea88d8e/> Remove brc20 event simulator test
 - <csr-id-ceb5be8416a21165ab3df9f1073aa47d5e154fc0/> Replace CHAIN_ID with Brc20ProgConfig.chain_id in various modules to enable using different chain IDs for signet and mainnet
 - <csr-id-90862fbb3ba4e7c5a08b65ac3967e25a71628445/> Update raw tx handling to use alloy TxLegacy type
 - <csr-id-6a93ee05efab15719cf7547461be3996834c80e5/> Fix cargo doc
 - <csr-id-8c401a9b1c1b2ffce878a04b6ccc06849a189271/> Move conditional compilation for Brc20ProgDatabase to a separate line
 - <csr-id-1b8f1f5a6b63490636fd66c3c47c99137b1b2d67/> Disable logging for common block retrieval calls to reduce noise
 - <csr-id-7977ab01c9e77e632b7e706d1e65afbd48783d0c/> Update database version from 3 to 4 in configuration, as previous gas values need adjustment
 - <csr-id-be926645f55fb2f2b53351fc1ee7006ab405ba49/> Move gas usage and history size constants to the global package, adjust gas usage for precompiles sending an RPC
 - <csr-id-0c8fcca035b00524e1a471ce34e6d89a6ba9276f/> Update BRC20 types to replace EncodedBytes with InscriptionBytes and EthBytes for improved clarity and functionality
 - <csr-id-795598b52862a308b83628f10358765b8c55e354/> Update decompression buffer to use dynamic Vec instead of fixed array
 - <csr-id-4cf9d212069459292af08297aa100fa9c3fc616a/> Update benchmark functions by removing unused transaction detail cases, add min stack requirement to readme
 - <csr-id-d1968eed7ec8dab87f52c590524107e1d1d1906b/> Add more test cases for bitcoin mainnet rpc tests and run bitcoin rpc tests sequentially
 - <csr-id-3163cd4f42e4ba5c5e3e66719101e9ae809b5402/> Update BTC precompiles to use faster transaction and block height retrieval methods
 - <csr-id-3bd0d87ad463857150a4e0a9f1df92f68d8818e3/> Refactor to use a "Transaction" instance instead of "GetRawTransactionResult" in BTC precompiles
 - <csr-id-8d5d7b8a280ce7d744a448a332fb3f48cc08a823/> Remove redundant names from instrument attributes in Brc20ProgApiServer implementation
 - <csr-id-7e22c4554510615cf46bdaa2fe2830e686eeee48/> Refactor Brc20ProgApiClient methods to include 'brc20_' and 'eth_' prefixes and update related client calls
   - Adjusted public API snapshot to match the updated API method signatures.
 - <csr-id-41daefdd27df528f496bc14fbaa6f7470ddfe578/> Reorganise engine into its own package alongside the evm, and public api types into the API package, add a public_api test to check for changes in the API
 - <csr-id-42efe3e72360a946974daabf3d46fdb497887b24/> Replace Wrapper types with ED types in API and RPC server
 - <csr-id-6bcbe483599ad3a6eb6d459927e5fa9ff2da4309/> Handle JSON-RPC error code -5 without panicking
 - <csr-id-efc0083fca2c77cebb4cbef05ebfe4f1cd61dcb8/> Remove minimum gas limit constant and handle inscription byte length in tests
 - <csr-id-bf970685dd9d5e0e5698cdd358adb641a74a492f/> Move dotenv call to Brc20ProgConfig
 - <csr-id-26f7b5f16e23912dd3458c27e4d7aa31e6f1d93c/> Update BRC20Precompiles initialization to use a constructor
 - <csr-id-5181b9c28c5fd07033977bbd5da1a3c86401df4a/> Simplify revm integration
   - Add a TracingInspector to use later
   - block.prevrandao is now set to current block hash
 - <csr-id-1a5ae117cade1814280651f8b9438215b91b4953/> Simplify EVM setup
 - <csr-id-843c4646b73ed3e7f66064a06fa816eed401df58/> Make tests ignore the result of .initialise(...)
 - <csr-id-75d6a133af28a90fceaf9e9a43a83b463b96c71e/> Remove unused test imports, use alloy_primitives instead of revm re-exports
 - <csr-id-f1570b4da72e1bcd8ba00a271b5c56d3e2b16cee/> Split db/mod.rs into db.rs / mod.rs, rename server_instance.rs to engine.rs
 - <csr-id-4cff9941c50e43fea647630f81834644234a1465/> Use read/write locks for evm database and propagate errors into the RPC layer to wrap and display
   - Renamed ServerInstance to BRC20ProgEngine (will rename the file in a follow-up to keep the diff)
   - Refactored error handling for various operations e.g. transaction validation and block finalization.
   - Created a shared data structure for access to database and block information, RwLocks now allow faster/parallelised reads
 - <csr-id-add636a06c7accbdecd2c6375a0b5c5bcde01492/> Update module visibility across various files
 - <csr-id-1ae91676a99b5c5733276dfdd4776e229c3ed34b/> Rename variables for clarity in various modules
 - <csr-id-15e16503608425a7d9af0ba0752a171f9c11e1c0/> Simplify data decoding in decode_brc20_balance_result function
 - <csr-id-8c19cd08292bf568d3209c72f2fa160262e95063/> Refactor calls to unwrap() for Result/Options to avoid unnecessary panics
   Use rust-bitcoincore-rpc library for precompiles
 - <csr-id-f2f18c4bcd1dd654d8a67d4c0ba9cab587e7f2c6/> remove unused FromHex import in brc20_controller
 - <csr-id-c4efb097b7a55b973632faf1b09c6252f292ccfc/> rename verify_block_does_not_exist to require_block_does_not_exist
 - <csr-id-912dfefc8d93a0ad39d1d788acc93083a0907f8d/> change account parameter type to AddressWrapper in get_transaction_count method
 - <csr-id-fb475d44cbed75d3254061e25cf0ff6fde238e88/> add chain_id and tx_type fields to TxED and update related structures, run cargo fmt
 - <csr-id-4045ddbfa1c736529daa6f2fc79ba9482716f0df/> change transaction index parameter to be optional
 - <csr-id-c02d3714fc22b376d0a1913f7f5806f65618de53/> update EthCall struct to make 'from' field optional and use zero in contract calls if from is empty
 - <csr-id-67f5b48efbf985abdb2138442c3deae2b71c88da/> Use LogResponse for eth_getTransactionReceipt for correct eth JSON-RPC format and add v,r,s fields to eth_getTransaction* methods
 - <csr-id-1ca98dcd0c689200218c297c38cec2d4ce29c3c3/> streamline byte, string and address conversions to use into() and parse() for type flexibility and consistency
 - <csr-id-ac703fd54431b45f481e401129b648a0f3561d0f/> simplify encode method to return Vec<u8> directly instead of Result, as it never fails
 - <csr-id-48a52bd4f4db178f9095958ad6b0576158225709/> update Brc20ProgApi call and estimate_gas methods to use EthCall struct and optional input data
 - <csr-id-0eae24b7bd1b6c742853822adacd89a6c2793f8e/> Rename block count variables for clarity

### Test

 - <csr-id-e99469e09cfebb59fb6cbd4b113c147b1eec5c1c/> opt-in live signet getTxDetails precompile test
   * test(btc): opt-in live signet getTxDetails precompile test
   
   Exercises the getTxDetails Bitcoin-RPC precompile end to end against a
   real signet node (no mocking). Opt-in via BRC20_RUN_BTC_RPC_TESTS=1 plus
   signet RPC env; skips cleanly otherwise so it is safe in CI.
 - <csr-id-234d87431db12d60464dce099fbe22deaf079dec/> cover validate_config and from_env
   * test(config): cover validate_config and from_env logic
   
   Config parsing/validation had zero coverage. Adds tests for
   validate_config (valid config, auth-without-credentials rejection,
   empty RPC URL, Bitcoin-URL requirement gated on fail_on_bitcoin_rpc_error)
   and from_env (mainnet vs testnet chain-id selection, malformed numeric
   env falling back to default).
 - <csr-id-2dc9d3a795e089783949c5943082f67726d88262/> cover controller helpers and deposit/withdraw/balance
   * test(brc20_controller): cover address verification and balance decoding
   
   Unit tests for verify_brc20_contract_address (accepts canonical
   address, rejects other/unparseable) and decode_brc20_balance_result
   (None and malformed data decode to zero, valid ABI word decodes to its
   value).
 - <csr-id-b6446739a51fa5c440209dd7687299ed939ff911/> cover block/tx/code/storage reads and eth_getLogs filtering
   * test(eth): cover block/tx/code/storage read methods
   
   Integration test exercising the eth_* read surface that lacked
   coverage: eth_getBlockByNumber/ByHash, eth_getBlockTransactionCountBy*,
   eth_getTransactionByHash/ByBlockNumberAndIndex/ByBlockHashAndIndex,
   eth_getTransactionCount, eth_getCode and eth_getStorageAt, all
   cross-checked after a deploy + finalise.
 - <csr-id-ca651492a512c47ebc8345851cd9118d76465ec1/> cover reorg, commit_to_db, clear_caches
   * test(engine): cover reorg happy-path, no-op, and rejection cases
   
   reorg() had zero coverage despite being a correctness-critical state
   rollback path. Adds inline tests for: rollback discarding blocks above
   the valid height, reorg-to-tip being a no-op, and rejection of
   future/too-far-behind targets without mutating the chain.
 - <csr-id-3e9e395c7bac26cb3a747cb2fc35d2bb1586eec8/> Update tests
 - <csr-id-b46ec70af7120595c547a032772c7f656e5594bf/> add unit test for Prague spec BLS precompiles inclusion
 - <csr-id-4472824b035db9c6822cf06fe4fb2bed824548e3/> Change out of order test to send 3 transactions
 - <csr-id-3af32341c61b424b28838c26765c31a1188cb006/> Split benchmark for BTC precompiles
 - <csr-id-34b6d4188ad8f9b5581b74ffbb87d563adc3ae35/> Split env setup in tests into .env.mainnet and .env.signet
 - <csr-id-c37b2ef18f4bf9c6263d0a95d086d52cf6a6bdb4/> Add mainnet precompile tests (works with a .env setup)
 - <csr-id-fb09f2751ad1c46d89c93545eb2fab83dac72dd1/> Add microsecond and second tests for gas processing times in opcode generator test
 - <csr-id-d40c627fe60c2d8da9d57497801dab26977afa91/> Migrate precompile benchmarks to Rust
 - <csr-id-a8ceb606d3456a24221b1345ad329596944888cf/> Move test utilities to their own package, add a benchmark for calling the bip322_verify precompile
   Refactored test files to utilize the new test-utils crate for loading files and spawning test servers.
 - <csr-id-2166b8bc70d1c5eeb6b3bdf34d6437fbb16c42f0/> Add BRC20 balance test and a deploy/call test
 - <csr-id-b4ea3f2f9f7ec6e855fd087da84b2e128142ac62/> Update test cases to use a non-zero value for inscription byte length
 - <csr-id-3a82185ae1ee37bbd072db2f2d45d979bf7c8d02/> Add serialization test for BlockResponseED
 - <csr-id-1e8c50cdba837afada7c1f2ea7345507686ff2d7/> Add unit tests for BRC20ProgEngine and RpcServer methods
 - <csr-id-541d41c7c3e1c104364c2f24e250f2a8eb6da45f/> Add unit test for HttpNonBlockingAuth allow_all method

### New Features (BREAKING)

 - <csr-id-c80def906ced3498a1039e0613df9056d8372c65/> Remove BRC20 balance precompile
 - <csr-id-93768685dd365a60051ec23d9b0b9b849eae9df2/> Remove BRC20 balance precompile and update version to 0.12.0
   Associated proposal: https://github.com/bestinslot-xyz/brc20-proposals/blob/main/002-prog-brc20-precompile-removal/index.md

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 462 commits contributed to the release.
 - 260 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 7 unique issues were worked on: [#10](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/10), [#4](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/4), [#5](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/5), [#6](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/6), [#7](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/7), [#8](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/8), [#9](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/9)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#10](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/10)**
    - Add Dockerfile and Docker Hub publish workflow ([`d5ecfbf`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d5ecfbf97089fb7a01c6353d12b59e3c60dd884f))
 * **[#4](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/4)**
    - Eth_call* and eth_estimateGas* methods now wait for block finalisation instead of failing when they are called mid-block ([`f29eadb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f29eadb206452bf0ce43ac831a109289e04ed3e1))
 * **[#5](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/5)**
    - Cover reorg, commit_to_db, clear_caches ([`ca65149`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ca651492a512c47ebc8345851cd9118d76465ec1))
 * **[#6](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/6)**
    - Cover block/tx/code/storage reads and eth_getLogs filtering ([`b644673`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b6446739a51fa5c440209dd7687299ed939ff911))
 * **[#7](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/7)**
    - Cover controller helpers and deposit/withdraw/balance ([`2dc9d3a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2dc9d3a795e089783949c5943082f67726d88262))
 * **[#8](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/8)**
    - Cover validate_config and from_env ([`234d874`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/234d87431db12d60464dce099fbe22deaf079dec))
 * **[#9](https://github.com/bestinslot-xyz/brc20-programmable-module/issues/9)**
    - Opt-in live signet getTxDetails precompile test ([`e99469e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e99469e09cfebb59fb6cbd4b113c147b1eec5c1c))
 * **Uncategorized**
    - Release brc20-prog v0.16.0 ([`8c61541`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8c61541e46620fd00c800e2255a2a662c4d41aee))
    - Update dependencies and tests for bip322 ([`b0d89c8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b0d89c889b60ba9d795beb24f71366a874a2c951))
    - Update warning note for Bitcoin mainnet status ([`1afb8e2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1afb8e29829deaa3bae9f21537efa1a5195680af))
    - Update nonce retrieval to use get_account_nonce for pending transactions ([`befe688`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/befe688ca35255a78264e55018a64283144dae2c))
    - Release brc20-prog v0.15.13 ([`ae07752`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ae07752720650afa664cd0a5ce972f3ee50d669c))
    - Release brc20-prog v0.15.12 ([`ac30d94`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ac30d946ff2257334da483327b6d2912b36d8a77))
    - Update eth_get_storage_at to return full length hex-encoded storage values ([`8b248c9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8b248c929f36cdfb3ae403e68ae01eb66884c4f2))
    - Release brc20-prog v0.15.11 ([`f1bc659`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f1bc65991205d727013edde1ff1e23c39e5cbf18))
    - Merge pull request #3 from bestinslot-xyz/estimateGas-with-binary-search ([`a568d19`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a568d1931e75760c0b138750aee5f3c377373ded))
    - Update expected gas response values in test cases ([`0896feb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0896feb3038bed3f919ad150feb15a9904309328))
    - Do a first run on eth_estimateGasMany and eth_estimateGas for an early exit in case calls fail ([`c38c7b0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c38c7b06ad3a2067872e64071f9362d00eefc4be))
    - Keep midpoint instead of gas_used * 2 as upper bound to cover some edge cases ([`228b912`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/228b912f377eb86a6752bdb836a1082e5868be75))
    - Enhance gas estimation logic with binary search adjustments ([`8d73788`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8d7378828b9a897da415825edf5cb0e1d44845ca))
    - Clean up properly if transact_one(tx) returns an error ([`790d806`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/790d806212e4b6067c857d570b2a1d25b68ba31d))
    - Clean up properly if transact_one(tx) returns an error ([`c22a945`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c22a9454698e55c80c96d68bc93af1c86fa7a9f9))
    - Add gas estimation with binary search for estimateGas and estimateGasMany ([`c99264a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c99264adc38f835af524bc1653a4e36af9f2ae60))
    - Release brc20-prog v0.15.10 ([`6680035`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6680035123e4a97b782e62385a5446b234d84139))
    - Implement eth_callMany with precompile data injection ([`a3b5e0b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a3b5e0b6246b0a02c68befab79a3fb7f40996ca0))
    - Update API ([`4b9eaf7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4b9eaf7329012eae0233b553b85ebcf069d04ae6))
    - Release brc20-prog v0.15.9 ([`d3abded`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d3abded92e5aad1b26f385c965110481cc57be9e))
    - Change tx hash computation to use RLP hash from a set block height ([`cd803cc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cd803cceca8cf411838270a72c36a589e66664c5))
    - Release brc20-prog v0.15.8 ([`c171eca`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c171ecaeac712d03f147f4418679602361de2da1))
    - Sort transactions by index in block processing ([`5081745`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/508174591e52e23f746feb13f2622f86b931c5bd))
    - Release brc20-prog v0.15.7 ([`6ef2f79`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6ef2f79e222c2f9ed453acd27ff21d00cfbb2c76))
    - Update tests ([`3e9e395`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3e9e395c7bac26cb3a747cb2fc35d2bb1586eec8))
    - Add debug methods for block trace string retrieval ([`831b9b6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/831b9b604631f37838206fb1931eed33b8aae0a8))
    - Reapply "chore: Update dependencies and add new packages" ([`d82c0f5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d82c0f59efe1601cff80fba3661d8fad094bcd4a))
    - Revert "chore: Update dependencies and add new packages" ([`5826548`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5826548f2c8878bd07122d30d4f02ab1a615d3db))
    - Release brc20-prog v0.15.6 ([`1468800`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1468800b51daff9edc9a9737b62c725d9edb8a93))
    - Update dependencies and add new packages ([`970a4f5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/970a4f5fdfe22a66f70e86cfd7f329cb4a98c1d0))
    - Add gas usage assertions in deploy_call, precompiles, and transact tests ([`a399035`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a3990352207838aedd0315d8d45007ba250561cb))
    - Release brc20-prog v0.15.5 ([`8036477`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/80364772d032edb82346192a36dac1d504b67a90))
    - Handle None case in get_info_from_raw_tx and update related logic, invalid chain ID shouldn't return error, and just ignore the transaction ([`660e8c0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/660e8c009bf291bc87abb0be971f1797237d7c2d))
    - Release brc20-prog v0.15.4 ([`021c973`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/021c97302d57997146622be790ca055d767fe336))
    - Cargo fmt ([`feed95e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/feed95e42977b71315d5b4e4789c82074ef3aaef))
    - Sort transactions by index in Brc20ProgDatabase ([`039b215`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/039b21520a786f58039cdfbb8a7d936790045986))
    - Update comment to clarify placeholder usage for block hash in get_transaction_and_block_hash_with_overrides ([`8679b42`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8679b42bb119fc0196d90916cab563111198d637))
    - Add PrecompileData struct and update eth_callMany and eth_estimateGasMany methods to include precompile_data parameter ([`9fcaaff`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9fcaaff1e53ac7575c569ca321447ae5cc1b574c))
    - Implemented eth_callMany and eth_estimateGasMany ([`fe608c3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fe608c340ca32242354f3e885afafcb6b3ada34f))
    - Add description for txpool_content method in README.md ([`af79d9c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/af79d9c3eeb635e8f6f7d5db667eee0fc3525229))
    - Update brc20_deploy, brc20_call, brc20_transact method parameters to include op_return_tx_id field ([`b29317f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b29317f79b03094c39b0ab8aad5dd6a7f1ca72ca))
    - Rename getBtcTxId to getTxId for consistency in function naming ([`9680ce2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9680ce28ccfa7a8b25da93e2760bf431188bc4dc))
    - Release brc20-prog v0.15.3 ([`d549690`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d549690f718938928a42add1fde85dd69cffca75))
    - Update public API (serde -> serde_core) ([`ca33922`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ca33922f02381a0f3f791f0226aefbccae1181ef))
    - Release brc20-prog v0.15.2 ([`8ee2ff4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8ee2ff46b77c50b268c12cfac3a20390cfbfe491))
    - Add revm-bytecode dependency to avoid bringing the whole revm for non-server version and update imports ([`4550e0e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4550e0eff6aa70d3cfa39ba7b378e2eccf5cd7e8))
    - Release brc20-prog v0.15.1 ([`6eb813f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6eb813fe2490d24147cf8988e6d6b8d4cb5ef7c3))
    - Update alloy to v1.1.0, revm to v31.0.0, remove revm_state and refactor accordingly ([`e368453`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e36845347bd58ed8eb5b3448c5d05ce01edabc26))
    - Release brc20-prog v0.15.0 ([`6436246`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6436246813bc9912d38b80299e558ad2ce2f6d29))
    - Enhance BRC20 precompiles with op_return_tx_id support (enabled after the prague upgrade) ([`ca2b864`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ca2b8647be71df9ed2bae0f0318c14c8c1b050bf))
    - Release brc20-prog v0.14.0 ([`3fe1e1e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3fe1e1eeffcb138021007f5ded7b852e64df85b0))
    - Change inscription_id and inscription_byte_len from Option to required fields in API and update database structures ([`c9178ef`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c9178ef3b704b02404b6b30477d4e01f5f54339c))
    - Ensure pending transactions are committed, cleared, and reorganized in Brc20ProgDatabase ([`3209728`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/32097286b9fd554d0221054fa8a1c3d3a76e0506))
    - Add unit test for Prague spec BLS precompiles inclusion ([`b46ec70`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b46ec70af7120595c547a032772c7f656e5594bf))
    - Use the block height parameter to eth_call and eth_estimateGas ([`5a4019c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5a4019ca24d982d12e2c7c96e64d313ea8410b27))
    - Allow "mainnet" and "bitcoin" to be used interchangeably ([`57a447d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/57a447d39515ec197bf7a865db07aac079d83d6e))
    - Implement dynamic EVM spec selection based on block number and network ([`ad4bb4b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ad4bb4b6c903c9911dba0e118bb01c0eb8d139e0))
    - Update generate_block to accept block_hash as a parameter ([`3e44cc0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3e44cc0b239ef2c7c0c45bc2cc28a30620204f37))
    - Reorder block hash setting to avoid race conditions ([`33b4ec2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/33b4ec2b72315e749f1c2c182df423f21c71231b))
    - Release brc20-prog v0.13.0 ([`d17add8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d17add84c97b05a5b135d9a95dabec8e0c35d615))
    - Remove BRC20 balance precompile ([`c80def9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c80def906ced3498a1039e0613df9056d8372c65))
    - Revert version bump, this is automated now ([`467fd9a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/467fd9ab4851d817c12c927d65487317566e86ec))
    - Merge branch 'main' into remove-brc20-balance-precompile ([`950de40`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/950de401174da5187e84b47eb3eadb3458df9e7b))
    - Bump version to 0.13.0 in Cargo.toml ([`e684694`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e6846948651b542114479828e606077ee305eced))
    - Release brc20-prog v0.12.0 ([`ab1f806`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ab1f806c4d3b60b3ea0e8d8bb77ac85cb18f53aa))
    - Remove legacy fields from TxReceiptED struct ([`f268486`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f268486b9584f377395fbbc940a12783505af471))
    - Serialize transaction type into hex ([`b4ac41b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b4ac41b640fc17c3186cdc551669bde7da64e99c))
    - Merge branch 'main' into remove-brc20-balance-precompile ([`83b59e5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/83b59e5ea0a3ec2134baa2ee8f49b2f71475a0f6))
    - Update homepage URL in Cargo.toml ([`c0b38c0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c0b38c05afe2fe20ce10e2de9a3748bf02d09c94))
    - Release brc20-prog v0.11.6 ([`0d8a3b1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0d8a3b1e61e273b56fba42877dd2ac265288f1f7))
    - Add changelog file (generated by cargo smart-release) ([`6fa1bf4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6fa1bf4e150151996a88af5310695a68fe33c11f))
    - Remove release configuration file and associated pre-release hooks to allow regeneration ([`0b0ae25`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0b0ae25ebc1308475b19d0b72967acd5b69b88fa))
    - Update pre-release hook to correctly format CHANGELOG.md generation ([`2a3adbd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2a3adbde258ad0ea873b20f105c9fc842fb17ee6))
    - Update pre-release hook to correctly generate and add CHANGELOG.md ([`746773d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/746773dfbbfedb3bc7f672476fe9a844ee1993c6))
    - Release v0.11.6 ([`851059d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/851059de776ff30d9fd0f8d7c34e8724eb43c633))
    - Add release configuration file for versioning and changelog generation ([`bb3d7cd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bb3d7cd23dbb679265464b1143fec786332a6aba))
    - Release brc20-prog version 0.11.5 ([`15b4150`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/15b415064506ece80e623ca56723125d16fc29f2))
    - Generate receipts for invalid transactions ([`834e75c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/834e75c56bd35c27fdd439837f3c3392f3f55560))
    - Enhance error handling for eth_call and eth_estimateGas with status codes and detailed messages in RPC server ([`f2e7d27`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f2e7d2791fd6acd9b360a2dea3d1c1898c4e5cb9))
    - Update read method signature to use lifetime annotation ([`a238d89`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a238d89cf77893009e8d7abcf8f4d38ddfbd2400))
    - Update read method signature to use lifetime annotation ([`27722d3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/27722d35d3f4e778033fcd560646b0e9aca88c64))
    - Remove test data files for BRC20 balance calls ([`4e6f326`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4e6f326190cc4c834d0c50a3f9cf352dac544a5a))
    - Remove BRC20 balance precompile and update version to 0.12.0 ([`9376868`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/93768685dd365a60051ec23d9b0b9b849eae9df2))
    - Bump version to 0.11.4 ([`72780b6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/72780b60f720eb5edbc1ee43e02d8cb6a3519a23))
    - Remove flag from RawBytes#value() function ([`ef1b4cb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ef1b4cb5c9dfd4b71e4664af9df0d3dd24c3b824))
    - Add step to build with no default features in CI workflow ([`4719fe7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4719fe75f051f7c7e563aae860c9807fe6541056))
    - Bump version to 0.11.3 ([`1eda8ad`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1eda8aded7a7a9ff96e84c0e534474048446ebed))
    - Update keccak256 import from revm to alloy ([`d1b383b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d1b383b3554e713cf3e939ce6a041af7ef5f47fb))
    - Bump version to 0.11.2 ([`0dad5dd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0dad5dd1dbc42afd0aaf6a60d8ca348ff9c54cd7))
    - Update public_api snapshot ([`dc47f65`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/dc47f65ed9a16abe05aab7f64b76324583fb14a6))
    - Correct version string format in web3_client_version method ([`a764b65`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a764b65df6901c0bb735e0091f049e40e622c4c9))
    - Implement web3_ api in Brc20ProgApi ([`a15371e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a15371e4737bb0b90fa6e9f8c401233dfe068fcf))
    - Update public api and fix a visibility error ([`d00d3c2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d00d3c2771628c06dec713fc6c6cd18c0f326ca9))
    - Add configurable request/response size limits and batch request settings to RPC server ([`3c088b9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3c088b9a8e328e344c81df24a30f4835ca7ebb08))
    - Remove batch request limit from RPC server configuration ([`ce01730`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ce01730b9951c1ca3cfe01e331b65a54c2b3af07))
    - Ditto, improve error message for Bitcoin RPC status check failure ([`7347175`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7347175402e12c2a9413a98a26ab09c466ae86e3))
    - Improve error handling in Bitcoin RPC status check ([`5a85fd8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5a85fd8596f0513ae78927ec3578375a3ff02edd))
    - Rename DB_PATH to BRC20_PROG_DB_PATH for consistency in env.sample ([`663c929`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/663c929ae910af58eb32788f8f3620d22c29bc14))
    - Enhance RPC server configuration with request/response size limits and batch request settings ([`b1d3f3a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b1d3f3a2f973e94acad4cc1a5d241fed50f4058f))
    - Simplify cargo run command in README ([`1c437ff`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1c437ff3a7ab18c318c386ffa9cffae1d404c2cf))
    - Print the error message in get_brc20_balance function when it fails ([`68dfbed`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/68dfbed0c7dc9e30d4b2dc93d57d176419f12762))
    - Bump version to 0.11.1 and update binary name in Cargo.toml ([`74341ad`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/74341ad293809caae1583bc7835d8c2a7e097b13))
    - Update eth_getCode to return 0x if there's no code present for compliance ([`c57ac66`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c57ac662947c524bcda68a1c456b227d6980a431))
    - Modify eth_call in RpcServer to return 0x or the revert reason for compliance ([`0ffa630`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0ffa6304546d3103ef94ac653104847c1273d348))
    - Add legacy fields back in BlockResponseED and TxReceiptED for backwards compatibility ([`3216212`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/32162127551206dd663df412d4fdd5ff8b3a2246))
    - Add input size limit warning for bip322_verify_precompile to prevent excessive resource usage ([`0ae78d4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0ae78d41a0688a76b363eca883b28b8456c7f6c3))
    - Add warning about ticker case sensitivity and format in README ([`86edafc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/86edafc539dc820771c498b1e90ac20816f903e5))
    - Enhance ConfigDatabase by adding caching and implementing flush functionality ([`c005115`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c0051151b20ab1002b9e90287b6ac3cd0f7ed431))
    - Use catch_unwind for BRC20 balance server checks to handle potential panics gracefully ([`cec0ccb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cec0ccb7de02ba0e3ab76c1bcb41daedd8a5a516))
    - Update public api ([`f537928`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f537928db53a9581b4321edca7dff67703ffb473))
    - Simplify balance retrieval by removing error handling from get_brc20_balance function, it now panics when balance server is unreachable or provides invalid data instead of silencing it ([`4d98b44`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4d98b447a20d2cb0060e2da66924f20536bdcac9))
    - Improve error handling in contract address resolution ([`b249ec9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b249ec9be690164558d8d85379ab214ca7d190fa))
    - Add caution note about basic auth and secure connections in README ([`5d83dd1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5d83dd1f8037d1a35e4375be25e0f60e8b772fdf))
    - Add a global values database to Brc20ProgDatabase and disallow multiple reorgs by storing a global maximum reorg height ([`29fbf41`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/29fbf41684a93dd86c0e71cf1a63a78e84af6d8d))
    - Refactor transaction handling by removing unnecessary fields, and simplify the code ([`a575638`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a575638e8fd2e0d510ad2c8854e7b469a5401429))
    - Cargo fmt ([`fdeb14e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fdeb14ed7ae4b25f060ba938b51e6f7323d411b9))
    - Refactor Brc20ProgDatabase to remove redundant timestamp and gas methods, they are stored in the block database already ([`29aae3c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/29aae3c00da1759ca03c231da0d81272f2dfc7fd))
    - Migrate back the bip322 crate after the fix and remove the fork ([`c3222bd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c3222bd1d37acc9346a832ed2af9e755f4deb817))
    - Simplify reorg logic in BlockHistoryCache and UintED serialization/deserialization ([`56b4846`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/56b4846140ed2120f90c7ba7ce7376e8d8b3ad3c))
    - Enhance bytecode handling with error checks during deserialization and decoding ([`f3fa71f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f3fa71ff498f398a468816bc72ac73179966cc39))
    - Update inconsistent docs ([`ea9a168`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ea9a1689aea1b33c7f07cfe67bf1f55290d109dd))
    - Fix test_reorg_all_blocks to reorg only 10 blocks ([`e28f5d5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e28f5d5c96e4076a119dd5381549fb8f637fae76))
    - Bump version of brc20-prog to 0.10.4 ([`a1e1d8d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a1e1d8d72ad89ab77a81593eb1976c2116d1e30a))
    - Use a fork of bip322 for a patch where it crashes ([`dbf69a7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/dbf69a74e9ff4b3bb9984092100626c873687c99))
    - Ensure latest value retrieval from the cache is never empty, and reorgs also panic if there's an attempt to empty the cache ([`123ad4e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/123ad4e872bb4bf455ecfda84895734a79ff89a0))
    - Update BITCOIN_RPC_URL to use address ([`57bf493`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/57bf493852203b6bb6178f510d7c4abb9a0b8b36))
    - Replace "DB Error" with a static error message and panic instead of returning an error, as it's not recoverable ([`f221717`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f221717beefa9de50df5eac62ae6c390c0c195e6))
    - Couple of bug fixes for reorg handling on database ([`3d15658`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3d156588df34b9b79aaca65414411aeda1acd349))
    - Bump version to 0.10.2 for brc20-prog ([`fa00152`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fa00152f808aac0efa78ca9b5341772c997cd3a3))
    - Refactor Bitcoin RPC validation logic and remove unnecessary checks ([`e0c605a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e0c605adaa08f043375fe769266e865d3d30139b))
    - Clarify initialization process for BRC20_Controller and empty block handling in README ([`4c6521b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4c6521b6d5d1c4d1121a6b59afde39be4dee0be6))
    - Add support for raw signed transaction inscriptions and update relevant methods in README ([`2ffeb5b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2ffeb5b5f2c5046b85d463fc9eb46ee51cb791d3))
    - Add brc20_transact method for sending raw signed transactions and update parameter names ([`af4cea8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/af4cea8e78254bb21a6cd28221a231c76db41626))
    - Bump version to 0.10.1 and ensure raw_block_ed module is conditionally compiled ([`ce4d985`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ce4d985887280d0b82ce753643b11a870feb6300))
    - Update Discord (badge was broken) and add Telegram link to README ([`a502861`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a502861740265f47c16151455a6c238bec772ec3))
    - Prefix hex-encoded strings with "0x" in RawBlock methods ([`53c126d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/53c126da6e25e06e11baa9ee56c491243ad47e42))
    - Cargo fmt ([`f5619cb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f5619cb44f0bdf537cdb3ee06934eeefeb92b6d9))
    - Add debug_getRaw* methods in the API and update version to 0.10.0 ([`443887a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/443887a631eb1b67eb8c72d189fe63a9323070cd))
    - Correct comment formatting for RawBytes struct documentation ([`b471516`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b4715160611dcb7d9a30eeb53661f51f3558571a))
    - Remove brc20 event simulator test ([`45b1f50`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/45b1f50610a1d26f1f035fc0a481542ffea88d8e))
    - Add "brc20_transact" method to indexer methods list ([`4727a33`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4727a3378ab471e70db5d3ad4af25eee5583233b))
    - Store and serve v,r,s fields from raw signed transactions, bump version to 0.9.0 ([`a6dba4b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a6dba4b05c2a3e905462104c8ca40dec9d172178))
    - Replace CHAIN_ID with Brc20ProgConfig.chain_id in various modules to enable using different chain IDs for signet and mainnet ([`ceb5be8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ceb5be8416a21165ab3df9f1073aa47d5e154fc0))
    - Handle extra padding in base64 decoding for inscription data ([`85bc09e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/85bc09e2b466b1a703fdaf0fae8c60396915ac7a))
    - Change out of order test to send 3 transactions ([`4472824`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4472824b035db9c6822cf06fe4fb2bed824548e3))
    - Correctly increment transaction index in pending transaction execution ([`b0ab81e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b0ab81e0976404338a101b9128c2c8dcb9e21061))
    - Update DB_VERSION to 5 in configuration ([`90c5477`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/90c5477443dae4cbc5430cbf57b77c08412d07a4))
    - Enhance BRC2.0 with pending transaction management ([`2222ed3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2222ed3240697ed3a40deec004a135a6b6ef071f))
    - Update raw tx handling to use alloy TxLegacy type ([`90862fb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/90862fbb3ba4e7c5a08b65ac3967e25a71628445))
    - Update ecrecover call to use adjusted v value and remove ignore from tests ([`4884c89`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4884c89c216c88a2849f72720d44e4b425c17989))
    - Ignore raw tx tests as they fail to recover the correct address at the moment ([`9302691`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/93026918f09b5f3f5561183bdd231c6f3ea7aad8))
    - Remove conditional flags for some imports ([`78aad64`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/78aad64571cb7bf8730d283b86530d8139cfc241))
    - Remove optional flag from nada and zstd-safe dependencies, as clients can use these to generate encoded data ([`eab6136`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/eab61361dd7cb73aafc36e6498703f2851dcc8aa))
    - Support using RawBytes and Base64Bytes types together in BRC2.0 API ([`69af83e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/69af83effd6c6931d1e52eef6c29d297e3c6d026))
    - Run cargo fmt ([`2e6a846`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2e6a84684a38b18f7aca7a3eb7d3780b62ba68a9))
    - Add brc20_transact endpoint to handle signed evm transactions, currently disabled via config ([`8e835b8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8e835b8be3ffd9407759d1315dbc8bae003f8d08))
    - Fix cargo doc ([`6a93ee0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6a93ee05efab15719cf7547461be3996834c80e5))
    - Move conditional compilation for Brc20ProgDatabase to a separate line ([`8c401a9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8c401a9b1c1b2ffce878a04b6ccc06849a189271))
    - Rename db.rs > brc20_prog_database.rs ([`b8ec391`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b8ec391975910cd83d4b1c9d87d4315840b04ec7))
    - Gate server-side features under "server" feature, this reduces client compilation time ([`b27ea7c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b27ea7c72a864e0febec413b25e03376099f81f6))
    - Bump version to 0.6.2 ([`e2deec2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e2deec2661cfc1c500f43ff356d8d25fca9afc89))
    - Implement TryFrom trait for AddressED and FixedBytesED, add is_zero method to UintED ([`bbab3bf`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bbab3bfe343afb171cfb3aedae5a9f1f2e88cb57))
    - Disable logging for common block retrieval calls to reduce noise ([`1b8f1f5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1b8f1f5a6b63490636fd66c3c47c99137b1b2d67))
    - Implement TryFrom trait for AddressED and FixedBytesED, add is_zero method to UintED ([`3f7d346`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3f7d34653a2719602b2290a710b98125e17ca87c))
    - Update database version from 3 to 4 in configuration, as previous gas values need adjustment ([`7977ab0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7977ab01c9e77e632b7e706d1e65afbd48783d0c))
    - Move gas usage and history size constants to the global package, adjust gas usage for precompiles sending an RPC ([`be92664`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/be926645f55fb2f2b53351fc1ee7006ab405ba49))
    - Update BRC20 types to replace EncodedBytes with InscriptionBytes and EthBytes for improved clarity and functionality ([`0c8fcca`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0c8fcca035b00524e1a471ce34e6d89a6ba9276f))
    - Remove warning about zstd compression stack size requirement from README, as it now uses Vec instead of a fixed size slice ([`4070f7f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4070f7fe4c6130fc543e4fc932c6c19fddd0ded4))
    - Update decompression buffer to use dynamic Vec instead of fixed array ([`795598b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/795598b52862a308b83628f10358765b8c55e354))
    - Update benchmark functions by removing unused transaction detail cases, add min stack requirement to readme ([`4cf9d21`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4cf9d212069459292af08297aa100fa9c3fc616a))
    - Split benchmark for BTC precompiles ([`3af3234`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3af32341c61b424b28838c26765c31a1188cb006))
    - Add more test cases for bitcoin mainnet rpc tests and run bitcoin rpc tests sequentially ([`d1968ee`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d1968eed7ec8dab87f52c590524107e1d1d1906b))
    - Update BTC precompiles to use faster transaction and block height retrieval methods ([`3163cd4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3163cd4f42e4ba5c5e3e66719101e9ae809b5402))
    - Refactor to use a "Transaction" instance instead of "GetRawTransactionResult" in BTC precompiles ([`3bd0d87`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3bd0d87ad463857150a4e0a9f1df92f68d8818e3))
    - Add EVM call gas limit configuration ([`b1753a6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b1753a6a41d82177ba13338a320e6c323a1f3134))
    - Split env setup in tests into .env.mainnet and .env.signet ([`34b6d41`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/34b6d4188ad8f9b5581b74ffbb87d563adc3ae35))
    - Add mainnet precompile tests (works with a .env setup) ([`c37b2ef`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c37b2ef18f4bf9c6263d0a95d086d52cf6a6bdb4))
    - Update LastBlockInfo to track total processing time ([`846099b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/846099bf4e941abcc37177d9250e04344ecac2c5))
    - Add microsecond and second tests for gas processing times in opcode generator test ([`fb09f27`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fb09f2751ad1c46d89c93545eb2fab83dac72dd1))
    - Reduce gas usage from 100k to 20k for bip322 precompile ([`49b03e4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/49b03e4c0564f195d311e439b501077196d72d3c))
    - Check if the transaction is in the future in last sat location precompile ([`25550c2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/25550c2bcf34f50731f1d16499909b32684ed3ea))
    - Migrate precompile benchmarks to Rust ([`d40c627`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d40c627fe60c2d8da9d57497801dab26977afa91))
    - Move test utilities to their own package, add a benchmark for calling the bip322_verify precompile ([`a8ceb60`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a8ceb606d3456a24221b1345ad329596944888cf))
    - Update README to include note about BRC20 indexer integration and enhance operation checks for deploy/call inscriptions ([`774f3db`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/774f3db79454f4780242f0a42bd05c80d0f02a18))
    - Add BRC20 balance test and a deploy/call test ([`2166b8b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2166b8bc70d1c5eeb6b3bdf34d6437fbb16c42f0))
    - Readjust stack size to 4MB in GitHub Actions workflow (again) ([`753be5a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/753be5a770a7f2497a67de3b4da95cb97228dd87))
    - Remove redundant names from instrument attributes in Brc20ProgApiServer implementation ([`8d5d7b8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8d5d7b8a280ce7d744a448a332fb3f48cc08a823))
    - Refactor Brc20ProgApiClient methods to include 'brc20_' and 'eth_' prefixes and update related client calls ([`7e22c45`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7e22c4554510615cf46bdaa2fe2830e686eeee48))
    - Reduce GitHub Actions stack size to 2MB, move long responses in precompile tests into assets ([`417fb86`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/417fb86602b76e502f0d65cfb4332358e9010be5))
    - Implement some precompile tests in rust ([`fe23eb3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fe23eb3e01bf6eef40ce253a2861ea3863b31f9d))
    - Bump rust stack size in github actions for zstd tests ([`1e56078`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1e56078c0286b78b894fa00989d502cee09b28d4))
    - Reorganise engine into its own package alongside the evm, and public api types into the API package, add a public_api test to check for changes in the API ([`41daefd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/41daefdd27df528f496bc14fbaa6f7470ddfe578))
    - Add zstd compression support with 1MB buffer size, reorganise some internal variables ([`e697503`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e697503ac501249fac67d6e440d52dfef9f87add))
    - Update package metadata in Cargo.toml ([`f9e8770`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f9e87704b189773605876ab92b46bd374d474107))
    - Update module documentation for clarity and completeness ([`988c239`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/988c23983cb0ba584ba1cbdc2b3783a3901c1aef))
    - Move documentation for Brc20ProgApi to top-level module ([`d5edb5a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d5edb5a9cbfeb2d094889ebbc9fe04e5686ee6c3))
    - Made brc20-prog into a library with a binary, bumped version to 0.4.0 ([`b1e0ae4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b1e0ae4b236e2b158e27e5cf53d6190939b741cc))
    - Increase block gas limit to 4MB (block size) * 12K (gas per byte) ([`7e0fb6f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7e0fb6fe2327866ff0802ebab058033b84a9e8f0))
    - Move super:: imports to crates ([`8a163e0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8a163e075c13c78ecab560ef51e2c8dc1e1e3472))
    - Generate std json output for example contracts ([`4cd41de`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4cd41de96ddee1d882c0aeaf3c87a6b441ced3bc))
    - Refactor integration tests: Remove legacy files and update tests to use BRC20ProgClient ([`0398501`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/03985016bce19835d62b34d7f86ffe80355cb579))
    - Replace Wrapper types with ED types in API and RPC server ([`42efe3e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/42efe3e72360a946974daabf3d46fdb497887b24))
    - Add deserialization for various data types ([`5579f73`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5579f739978f6c963138f0356d8aba82674c80d0))
    - Update alloy-sol-types dependency version to 1.1.0 ([`1eee9df`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1eee9df5682cb19454d45ee1c5ee43fbc5832d1c))
    - Add functionality to retrieve inscription ID by contract address, bump db version to 2 ([`7f38574`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7f38574d482b9e33cd2cd40439b854a4f0e6abc0))
    - Update result field name from resultBytes to output in transaction receipt ([`98bb8ff`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/98bb8ffa89b09929aab08d43ea771523f340f011))
    - Remove unused import of E from std::f32::consts ([`b5f82f0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b5f82f0716d7ee3a3588eef1a132ef36875e28ef))
    - Run cargo fmt ([`495186d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/495186d6cce4bf8ec92f4408d995bdcc1de8dc66))
    - Handle JSON-RPC error code -5 without panicking ([`6bcbe48`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6bcbe483599ad3a6eb6d459927e5fa9ff2da4309))
    - Return an error if block range is too large for eth_getLogs ([`4df9d97`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4df9d976d42b7894dc52bb56850ae75cf5a0c25e))
    - Update test cases to use a non-zero value for inscription byte length ([`b4ea3f2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b4ea3f2f9f7ec6e855fd087da84b2e128142ac62))
    - Better eth_getLogs topic and address handling ([`a462db4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a462db4a4df63f5465a9ed48ad12c357cf0c7027))
    - Remove minimum gas limit constant and handle inscription byte length in tests ([`efc0083`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/efc0083fca2c77cebb4cbef05ebfe4f1cd61dcb8))
    - Update dependencies and update RPC authorization middleware to latest jsonrpsee library ([`4a894c6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4a894c6c04c7f01917fa614f51221ad3848bdb92))
    - Handle nada encoding in inscription data (not activated until compression activation height is set) ([`1408102`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1408102192951ba007c62352a92fb9ac6ba0b040))
    - Add PROTOCOL_VERSION to configuration and database validation ([`752f5f0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/752f5f04083e163aa047756979c5130528abc14c))
    - Remove unused rust-embed feature "include-exclude" ([`c7f75c8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c7f75c87a4370cfb9afdd3da518d5a8f8892539f))
    - Use dotenvy instead of unmainained dotenv, avoid embedding the source for brc20_controller contract to reduce binary size ([`7cfc17b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7cfc17b10315a19f6b09edeed4621423f67162ea))
    - Move dotenv call to Brc20ProgConfig ([`bf97068`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bf970685dd9d5e0e5698cdd358adb641a74a492f))
    - Update environment variable name for BITCOIN_RPC_NETWORK, and use a denylist for RPC method auth ([`2b5311f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2b5311f795784b4e0382313e3bcf19a0a3f8d109))
    - Add dotenv support and update dependencies ([`4b36434`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4b36434cac54d7548fe5e8af8b87ee65b70f8cd7))
    - Validate bitcoin network using bitcoin rpc ([`180b24e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/180b24e9ca16500d239f19c9c0749c9eeda0de3a))
    - Bump version of brc20_prog to 0.2.0 ([`3de713d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3de713dc4c78562a7cb290bf292cde66d0642575))
    - Refactor Encode/Decode to use buffers for efficiency and code simplicity ([`64fe806`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/64fe806a968b94ec7649362e01728a384a15481a))
    - Simplify assertions in tests for AccountInfoED conversions ([`391139c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/391139c6c3ff02efcb029befa82c54a8370f6eb9))
    - Refactor data structures and conversions for AddressED, BytecodeED, BytesED, and UintED ([`495a46c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/495a46c94c919ffecfae46c840331054ccc1a278))
    - Add EVM_RECORD_TRACES to env sample ([`ee9d7b7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ee9d7b786052253cd9ecfdc413fbec425a06a789))
    - Add an environment variable to toggle trace recording ([`73784c2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/73784c2dee9571e1fec735684b77911d59ec3aae))
    - Add serialization test for BlockResponseED ([`3a82185`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3a82185ae1ee37bbd072db2f2d45d979bf7c8d02))
    - Serialize block nonce to full hex length ([`ae96ee4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ae96ee4b209d45ab1b0ddf7422fba081cda81e76))
    - Make authorisation logic exclusive to brc20_ methods, and fix TraceED encoding ([`a5ef831`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a5ef831d20cb19145c9c847f42f930d8eff88e46))
    - Record internal transactions and add an API endpoint to return them ([`b831c7d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b831c7dc112b8500a983709658802e3a6664ee61))
    - Update BRC20Precompiles initialization to use a constructor ([`26f7b5f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/26f7b5f16e23912dd3458c27e4d7aa31e6f1d93c))
    - Simplify revm integration ([`5181b9c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5181b9c28c5fd07033977bbd5da1a3c86401df4a))
    - Bump revm, alloy deps and migrate ABI decoding and precompile logic ([`f31be5c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f31be5c72a23832e0256d7e9a72e5dc345aaf728))
    - Simplify EVM setup ([`1a5ae11`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1a5ae117cade1814280651f8b9438215b91b4953))
    - Make tests ignore the result of .initialise(...) ([`843c464`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/843c4646b73ed3e7f66064a06fa816eed401df58))
    - Add unit tests for BRC20ProgEngine and RpcServer methods ([`1e8c50c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1e8c50cdba837afada7c1f2ea7345507686ff2d7))
    - Refactor contract call methods to redirect invalid transactions to 0xdead ([`401fb70`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/401fb70a2d46aaf96f5b52906e466ab16c30cf0c))
    - Add inscription_id field to TxED and implement API method to retrieve it ([`c2f23df`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c2f23df1d127f6268d804b778e30a14274b97572))
    - Remove unused test imports, use alloy_primitives instead of revm re-exports ([`75d6a13`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/75d6a133af28a90fceaf9e9a43a83b463b96c71e))
    - Split db/mod.rs into db.rs / mod.rs, rename server_instance.rs to engine.rs ([`f1570b4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f1570b4da72e1bcd8ba00a271b5c56d3e2b16cee))
    - Use read/write locks for evm database and propagate errors into the RPC layer to wrap and display ([`4cff994`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4cff9941c50e43fea647630f81834644234a1465))
    - Simplify serialization and encoding of BytecodeED by using original_bytes ([`19b4d14`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/19b4d147c4143e278744e47681ae6ada486b735e))
    - Return BytecodeED from eth_getCode, so it gets serialised without the trailing zeroes ([`96f0012`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/96f0012238e70a6197f77f5a858cd0266a9f4a4b))
    - Update module visibility across various files ([`add636a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/add636a06c7accbdecd2c6375a0b5c5bcde01492))
    - Handle missing contract addresses by using Address::ZERO in Brc20ProgApiServer ([`35125ed`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/35125ede4e7d1ba716c29dd5d7c18a182566154a))
    - Enhance contract metadata handling in BRC20 controller ([`41c348b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/41c348b8bd3da36387680475e62019c78d4d048a))
    - Updated BRC20_Controller.sol to deploy Wrapped BRC-20 on new ticker mints. ([`015e0df`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/015e0df01fc2ba9226eec288534fb8a8ab04eb08))
    - Upgrade Solidity compiler to version 0.8.28 and update related dependencies ([`654c8e2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/654c8e282cd18e7b459e940e879902a10281f3a9))
    - Enable optimizer and update output file generation in BRC20_Controller contract ([`1324f62`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1324f623bd9593c9e2b2a8e8a0e6b3ef66b0b484))
    - Add a js script for BRC20_Controller contract to fix solc version ([`0e96a89`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0e96a89cbe90f63fd9fc7b56bb22dba2bfd2ba00))
    - Move Id import to tests ([`829efa5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/829efa531ee062b3149e9a85f3fc4aac34ccd0cb))
    - Return correct ID ffrom RpcAuthAllowEth ([`c2317a2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c2317a22aff1f61e3507e48e7f91e90f4547f0bc))
    - Add unit test for HttpNonBlockingAuth allow_all method ([`541d41c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/541d41c7c3e1c104364c2f24e250f2a8eb6da45f))
    - Fix HttpNonBlockingAuth for allow_all(...) ([`c2cb483`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c2cb4834fa8aeb80ec384573f2f8fe55cab41fbf))
    - Implement HTTP Basic Authentication middleware for RPC server, it allows all methods that start with eth_*, and rejects everything else if the request is unauthenticated ([`97b2d66`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/97b2d6623c187a2889c06ac9c00a45d49f9a6142))
    - Add basic HTTP authentication support to the brc20_prog RPC server ([`ea2bd18`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ea2bd18608aa9d430996d56aa2ffc803b5f4db51))
    - Bump version of brc20_prog to 0.1.1 in Cargo files ([`492065f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/492065ff749fd418983fb51eb4f1b54a80eb1dc7))
    - Add data_or_input method to EthCall for improved data handling ([`000db49`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/000db4966fec5593798b50b0b19829a665ca2e18))
    - Rename variables for clarity in various modules ([`1ae9167`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1ae91676a99b5c5733276dfdd4776e229c3ed34b))
    - Simplify data decoding in decode_brc20_balance_result function ([`15e1650`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/15e16503608425a7d9af0ba0752a171f9c11e1c0))
    - Ignore transactions that happen in the future in tx details precompile ([`d17c4f6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d17c4f651213e2c0cb9fac829021352fa4b1f43f))
    - Refactor calls to unwrap() for Result/Options to avoid unnecessary panics ([`8c19cd0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8c19cd08292bf568d3209c72f2fa160262e95063))
    - Update BRC20 locked pkscript precompile result for integration tests ([`5e9bf0f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5e9bf0f04fc3662f914c788cbaa6f4ba5deca2d0))
    - Fixed pkscript pubkey things on get_locked_pkscript_precompile ([`bf614ce`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bf614ce033a05c17829b81a990d61d3c77834259))
    - Last_sat_location_precompile location finder now uses do-while ([`add1527`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/add15270a0184539f1ad74327f2be12d191b504d))
    - Get_brc20_balance now uses u128 which is more than brc20 balance limit (2**64*10**18) ([`f661579`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f6615799add6af0d2075358fc49c1402059544f0))
    - Remove unused FromHex import in brc20_controller ([`f2f18c4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f2f18c4bcd1dd654d8a67d4c0ba9cab587e7f2c6))
    - Change ticker parameter type to Bytes in transaction loading functions and add ticker_as_bytes utility function ([`d8d93bb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d8d93bb628e863e3b4fc629a100b1c900c7dac8d))
    - Rename verify_block_does_not_exist to require_block_does_not_exist ([`c4efb09`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c4efb097b7a55b973632faf1b09c6252f292ccfc))
    - Include assigned gas limit in txes ([`6e58ca1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6e58ca17a66ad7dffd8540b3cdfb7bc76391f959))
    - Verify new blocks don't already exist in the database, and generate new hashes for new blocks with hashes 0x00..00 ([`b0a754c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b0a754c1f0341e7fc569a942cfb9cd8e56d227fb))
    - Add gas_price and syncing methods to Brc20ProgApi (to enhance explorer compatibility) ([`f7e2a7b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f7e2a7bdf5054d951d2b08d6cfe97f51ee0dde79))
    - Change account parameter type to AddressWrapper in get_transaction_count method ([`912dfef`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/912dfefc8d93a0ad39d1d788acc93083a0907f8d))
    - Include data in returned error instance for eth_call and eth_estimateGas ([`66495a5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/66495a5c9cb72db613765e3e8abcefafddd966ae))
    - Add chain_id and tx_type fields to TxED and update related structures, run cargo fmt ([`fb475d4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fb475d44cbed75d3254061e25cf0ff6fde238e88))
    - Change transaction index parameter to be optional ([`4045ddb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4045ddbfa1c736529daa6f2fc79ba9482716f0df))
    - Prevent deadlock in get_block_by_hash by releasing the mutex after accessing it ([`1a56564`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1a56564a185f301dceb4bcb2c8fe82eb5dd9a8b8))
    - Update EthCall struct to make 'from' field optional and use zero in contract calls if from is empty ([`c02d371`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c02d3714fc22b376d0a1913f7f5806f65618de53))
    - Use LogResponse for eth_getTransactionReceipt for correct eth JSON-RPC format and add v,r,s fields to eth_getTransaction* methods ([`67f5b48`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/67f5b48efbf985abdb2138442c3deae2b71c88da))
    - Streamline byte, string and address conversions to use into() and parse() for type flexibility and consistency ([`1ca98dc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1ca98dcd0c689200218c297c38cec2d4ce29c3c3))
    - Simplify encode method to return Vec<u8> directly instead of Result, as it never fails ([`ac703fd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ac703fd54431b45f481e401129b648a0f3561d0f))
    - Rename block_hash field to blockHash for consistency in serialization ([`dd53a43`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/dd53a43ad2b938ed3d19c3842d9edbcc2f222963))
    - Update Brc20ProgApi call and estimate_gas methods to use EthCall struct and optional input data ([`48a52bd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/48a52bd4f4db178f9095958ad6b0576158225709))
    - Convert contract bytecode using to_string to add the 0x prefix ([`657a99b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/657a99b18db4041802e19c76e118270016b18f37))
    - Rename block_hash field for serialization ([`bbd9669`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bbd9669f3163af5932dccb9dab81e3fc0fd1671a))
    - Update README.md ([`be1d4b2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/be1d4b2b024112305125faeb991ac703ec1b5c96))
    - Update README with warnings about BRC20 balance server and BIP322_Verifier contract limitations ([`25afd99`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/25afd999173c616524fca02a8fa41136d911ce4d))
    - Made simulate test 5 byte ticker aware, added new log events to brc20 deployer test contract ([`5b2ab10`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5b2ab1082c326887bbe726c3c727ad5783173f73))
    - Mine_blocks genesis block count ([`17aa2a9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/17aa2a93f5059f62b4fc590e11433dde712425ca))
    - Refactor Brc20ProgApi to use EthCall struct for gas estimation and update get_balance method signature ([`7d1410c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7d1410c25aec24459a80bc96e2a7775e48448e5a))
    - Add hyper and tower-http dependencies; implement CORS middleware for RPC server, add eth_accounts and net_version ([`a0c37db`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a0c37dbb0e9603c8fcff18b6e2a869f92bc6bc43))
    - Add block_timestamp to TxReceiptED and related methods for enhanced transaction data ([`1de55ad`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1de55adb4e485ff12bcab74546cfcde9da16d5ce))
    - Refactor BIP322 verification precompile to use bytes instead of strings for parameters ([`11935ba`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/11935bad0e7779a18af2e65c7f030727d676d201))
    - Add inscription_byte_len parameter to BRC20 methods and update gas limit calculations ([`8eb2905`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8eb2905cefd3e4bdd47a905f500fe62d546c8ae9))
    - Update BRC20 balance check to use non-empty byte arrays for testing ([`32414d1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/32414d14112ec1e23cb273c8297a8d944554f062))
    - In precompiles, use bytes32 for packing txids, and bytes for packing pkscripts. Update the tests and the docs for them. ([`1853af1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1853af14d18f5ccaf75282fe27f06aea8d73b7af))
    - Reorg to current block height is allowed and no-op ([`c9b69a0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c9b69a08df124cced56a17cf7507783e9db194d3))
    - Remove gas usage for get_block_height, add ignored is_full parameter for eth compatibility ([`cbf0a49`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cbf0a49e1c49e710e683496400cea369db647b45))
    - Refactor performance test to include contract_inscription_id in calls and improve error handling ([`be791e4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/be791e43bba6d5cbcaeddd41b8cc65bfadf0528b))
    - Split brc20_addTxToBlock into brc20_deploy, and brc20_call RPC methods for easier contract address and inscription id handling ([`b12a057`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b12a057075e9176495cf821538778d518c790d9c))
    - Changes to make simulate_brc20_events.py running ([`083a672`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/083a6720b441969875fceff305d2bbf9038da9fc))
    - Remove BRC20 balance server status logging and add status check before server initialisation ([`19738e1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/19738e1c894f9d5de79923b337ec5cd4406d602e))
    - Update default Bitcoin RPC URL to use port 38332 ([`a3064a7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a3064a7ff76b72b8c8072a8e3e9c9bf9c9de1378))
    - Update default Bitcoin network to signet in btc_utils ([`95a177d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/95a177d166c0ec0fab607b9d563fa7954091c05c))
    - Add version method to Brc20ProgApi to return current package version ([`fda8651`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fda8651d27fabe193c3fa3fc35d0cd1332d383d8))
    - Update README.md to clarify environment variable requirements for precompiled contracts and add clang installation instructions ([`6116a0c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6116a0c6d8e350b9f3b2f2d80f5b19a03bb1786b))
    - Run cargo fmt ([`771dd01`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/771dd01f4ee7d8c2f13d9b04cb20fbd3efe20d1b))
    - Add logging for get_logs, call, and estimate_gas methods in RpcServer ([`45c8b55`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/45c8b556ce632f77e8250148ba6daca5e788eac1))
    - Refactor GetLogsFilter to use B256Wrapper for topics field ([`1eae2d2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1eae2d204dc3ae86b791a91a5b52c47beafa73fe))
    - Refactor GetLogsFilter to use AddressWrapper for address field ([`a2f32ff`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a2f32ffb4870c32d742bb5e18dac6e8f68b3915e))
    - Refactor JSON-RPC API to use wrapper types to consolidate sanitisation/deserialisation ([`d4ba35f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d4ba35fb52cf2b692b8f080f1c318a94c2ae752c))
    - Update expected hex output in BTC precompile tests ([`843822b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/843822be5603592c2dfbf3e791c128691bd36046))
    - Make precompile functions use a shared HTTP client for improved reliability ([`b86882b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b86882b9da45e0800b9fe640bb2f1b4627b67d6b))
    - Refactor performance test to load transaction data dynamically and update precompile test selection ([`210c487`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/210c487ae59a29b295b80a5299e330296939db04))
    - Refactor BRC20 precompile functions to use alloy_sol_types ([`cad81d1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cad81d1341082eb9a4226c7c30322eeefe81fc71))
    - Add input prompts to pause on failed precompile tests ([`9fbb46e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9fbb46e378d94d6941e54f712a69e3c61d86550c))
    - Update TxReceiptED to serialize result_bytes as an empty string when None ([`126cce0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/126cce05654df1d15b40c28da49ebf6d23ab19ef))
    - Avoid overflow in gas usage in case of errors ([`708a2e7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/708a2e7c4e8cb22b978f6067777e067741b67dbd))
    - Use tx.json files in Precompile Test ([`c297035`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c2970354c29bd2cf9e90f7b6d254ada7c20eace1))
    - Add support for inscription IDs in BRC20 transactions and receipts ([`51640e6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/51640e6c23ec6550a37fe8e42196398f5dc9cf15))
    - Check results of precompiles in precompile test ([`887a00c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/887a00c43f2f81ede0f68d729c65d10ed41fd603))
    - Check Bitcoin RPC and BRC20 Server status at startup ([`4a46fc3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4a46fc3efab19170968b2a2a10794f4d869f5e20))
    - Ignore u64::MAX for gas usage, and rename BRC20 Helper contract functions to match precompile signatures ([`ec82f24`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ec82f24e73921392d6a1e163ff1a51d7464c3923))
    - Add better error handling and sanitisation for BTC precompiles ([`9215baa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9215baa8ced2c1d156745fc94c0de52c73b4e2d5))
    - Move btc precompile tests to signet ([`2a6d050`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2a6d050358321b14c8d138408546617a746f5a1f))
    - Fix bytecode comparison and encoding to handle padding bytes correctly ([`8f7d042`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8f7d042c6fbea9b5a15b77c30dce6951c3fc2a38))
    - Change order in precompile test ([`20df50d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/20df50d49f32fff05b70a43fe06e00ebfa378754))
    - Refactor BRC20 precompile to use ureq for HTTP requests and add another test that runs all precompiles ([`ff4ff9f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ff4ff9f1fb781e1a7e04c16d3f851b046c35a359))
    - Refactor precompile interpreter result handling ([`da28dad`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/da28dadff21d5dc38767b5bdcd9184fba30152fa))
    - Upgrade to revm 20.0.0, add more tests for precompiles ([`95c514e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/95c514e7c3f6e04570733029945ad81cf1b8c7ba))
    - Comment out direct precompile call ([`fc30e4e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fc30e4e06ab5255ffb4f4e229f4ea6a1d80c8c74))
    - Implement direct static calls for BRC20_Prog contract ([`b1f7a01`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b1f7a01e7ca4fbc731036584d71f671d2daebd22))
    - Add transaction failure handling and clean up gas usage output in performance tests ([`173db03`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/173db03749aa106a91e1e167e53ffab31655b7e9))
    - Add a performance test for BTCTxDetails precompile ([`fbcea56`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fbcea56f61a87b43d90493dd48763ed248796026))
    - Make BTCTxDetailsPrecompile retrieve block height via another RPC in case it's not indexed ([`7ba3049`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7ba304905a1d1706890fb06d9a943a8e5df632cf))
    - Update sample environment variables for Bitcoin RPC URL and network ([`13921f8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/13921f89bcbf4679caaa859c92d9453d6b5fc3d6))
    - Use an environment variable for the RPC server address ([`0c93bb5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0c93bb599f20224a745ca83c9e80b1aa979f0e55))
    - Update README to specify JSON-RPC server address for BRC2.0 ([`2f47eef`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2f47eef4c40050ea20f72bbaa06297430cca8b04))
    - Add License and Notice ([`87eabea`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/87eabeaeaa73dbfea550cd456990b673417d4c66))
    - Enhance README with detailed return descriptions for BRC20 JSON-RPC methods ([`d1e412a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d1e412a7457f3c4e4b3effd429c1d0507aa5a7ae))
    - Format code blocks in README for better readability ([`e36c5dd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e36c5dda038bb911cd2fc43abb47775809a87eae))
    - Update README to reflect BRC2.0 branding and enhance module description ([`1ab94f0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1ab94f053e741ea014f05a7bbe2099f55d25c7a1))
    - Update README to include Rust requirement ([`fdf4cab`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fdf4cab7fce978f1a845ca3b48b80bb1ea96483e))
    - Add indexer integration guide to README ([`c855e09`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c855e09fc83e790e86a02ef26ae6a002d89a0330))
    - Update README to clarify HTTP call for BRC20 balance retrieval ([`20d64d6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/20d64d661d21d403527214c38ece1ff58eb644e3))
    - Clarify README description of JSON-RPC methods functionality ([`725f10d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/725f10d5a21e8725fdf9eafe02dc6d852772ef48))
    - Update README ([`8e363fb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8e363fbfc1a3aba3c1096acc49a55e31d2b9155a))
    - Add functions to get last satoshi location and locked pkscript in example helper contract ([`34afc47`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/34afc470f771348e390ac7f90e2a7e045aa7f0d5))
    - Add github actions and skip BTC tests if the environment variables are not set ([`2db6b81`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2db6b81e2e0084cceb26d0b58c91a1c0ebcb6247))
    - Add empty README and remove unnecessary files ([`c58678d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c58678dd286f988ad0dee1e27b8b842bb10b932b))
    - Run cargo fmt ([`f01095a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f01095a1bbf4a4403b5b604619c19b11bb7d8484))
    - Add transfer tx for sample erc20 contract ([`eb03c03`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/eb03c031f83b85693173b789f0e0bfced0c3aaae))
    - Update import path for ERC20 contract in BobCoin.sol ([`1903298`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/19032982d0af197ea6aefd64f710c9888696aa81))
    - Add an ERC20 contract example ([`fd9d875`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fd9d875ba40a6038a25d225a2604aeb63b0ad41d))
    - Show float numbers in test_generator.py ([`09d4de4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/09d4de4b3e5986c7732e60cc4d794cf4eac61344))
    - Rename block count variables for clarity ([`0eae24b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0eae24b7bd1b6c742853822adacd89a6c2793f8e))
    - Make get_range end key comparison exclusive ([`5c0b892`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5c0b892a5acc399cdd85ba59e2e2b98f94d5672b))
    - Add genesis height parameter to BRC20 initialisation methods and update related logic ([`5abf996`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5abf9969419002d1d398cc3e61cc6bf35c687d01))
    - Refactor module structure by reorganizing EVM and BRC20 controller modules, removing obsolete files ([`171fc07`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/171fc073ff5a0da292fe8de052267360d4dabe14))
    - Improve error message for call_contract ([`faebece`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/faebece4e1f349fb5a0800951f216553e4f07d1e))
    - Run cargo fmt ([`3e314c3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3e314c355877e053c017b4c9c8c9c1d8c024e344))
    - Set max lock block count to 65535 in GetLockedPkScript precompile and implement tests for edge cases ([`40b3c5a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/40b3c5ae2bf41812d63903378f4afe6d879add3d))
    - Add environment variable checks for Bitcoin RPC in BTC transaction details precompile tests ([`dff41b8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/dff41b841fca1a2449ea63e23218a04de1800fd0))
    - Add error handling for invalid lock block count in GetLockedPkScript precompile ([`f596d4a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f596d4aaa490de3848a546241202c3e195e01434))
    - GetLockedPkScript now uses environment variables to change network ([`14d0bb1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/14d0bb1fa5800dd6ec300794ee8aff751a2a6cd8))
    - Implement GetLockedPkScript precompile with locking script logic ([`7319f51`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7319f51e96de6b478e678b658476d9531611747f))
    - Refactor precompiles: rename and reorganize modules for clarity and add empty GetLockedPkScript precompile ([`c730724`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c73072483e0940db4575ee0674b146d87bca1db7))
    - Add error handling for missing block height in BTC precompile ([`04ba5e9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/04ba5e90a56187c6fa3fb4ceb9321dc29c438e6a))
    - Revert "Return empty values instead of an error for coinbase transactions" ([`8e05f7d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8e05f7d07956ebd236f18ecbe194e447bf11549b))
    - Return empty values instead of an error for coinbase transactions ([`5553a91`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5553a91e6cae866804ef70e64e19e785e6fba1da))
    - Add support for error handling of coinbase transactions in LastSatLocationPrecompile and update related tests ([`6720dca`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6720dca47b5ebb6cf6e0b63714ee4bc72a141e62))
    - Remove unnecessary usize casts ([`377a1b4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/377a1b4f85b533e31e000f17f4385344554619fd))
    - Implement Last Sat Location precompile, and reorganise the precompiles module ([`a58281d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a58281dc6de8b841cb917efcd363ffd2a9ef90f2))
    - Add multiple test cases for btc precompile that also checks gas usage ([`838606d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/838606dc653438bd6cbd28a7805be41cfea91ef4))
    - Return all vin/vouts as arrays in BTC precompile ([`8501e5e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8501e5e4fa3139a33683f7e5df936ac63cc1cd68))
    - Add commit and clear_cache methods for db_block_number_to_block ([`5f7cc30`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5f7cc3080787f0ac269fc5ea968d446d92970c47))
    - Remove unused requirements file from test_generator ([`6e470aa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6e470aa1a2a0ae3542c0ca4ffd5410fd6304887d))
    - Run cargo fmt ([`971e3cf`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/971e3cfff81a1088db17b0bc70e841bd6762daaf))
    - Format some python code ([`3e4ccb9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3e4ccb9b1e6ba565cbf200fb5154066bb1e0d9f9))
    - Refactor some integration tests to match current spec, and update brc20_deposit and brc20_withdraw methods to return TxReceipt ([`96f959f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/96f959f76daece7b848636368664852cd0495414))
    - Change log level from DEBUG to INFO for deposit, withdrawal, and balance checks ([`329e775`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/329e775b97c4f088318f141b589c86be052a946e))
    - Improve error handling in BRC20 balance decoding function to return zero on failure ([`dc82f7d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/dc82f7dc01d95026dfd12c1feef1f3f6591da299))
    - Refactor Makefiles to streamline ABI file formatting ([`b7a4ba9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b7a4ba91542ee633f4c5ef4781bf6fc5eafc8264))
    - Add BRC20 program helper Makefile and update contract interfaces; enhance error handling in precompile ([`6b45637`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6b4563705eaf60f1c9c82f67e8f09f1133a141c9))
    - Implement BTC precompile ([`1cf22ea`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1cf22eae75c0ff9192f7566c4b2d853932928954))
    - Update expected result in BRC20 balance precompile integration test (according to the hashing balance server in OPI brc20 indexer) ([`c8f3e0e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c8f3e0ee9a84c36e8de838b06a460ab9d1719488))
    - Add BRC20 balance precompile and integration test; update dependencies ([`4b51a21`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4b51a21fd9a80e4078e91c9f8f6e61f0461d8ecf))
    - Fix integration tests and enhance RPC server logging; store block responses in database for faster retrieval ([`fc2f881`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fc2f8814dfac51ea68425f6c4b6ac5e1707b8a1f))
    - Implement trace logging in brc20 RpcServer methods ([`345c7d1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/345c7d1b8722481a0eaaa176cb161972b040df0e))
    - Fix function selector for balanceOf in BRC20 precompile ([`2bf7622`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2bf762259df7615df65ff5aa646b07d1159769aa))
    - Add brc20_initialise method for genesis block ([`e2f83de`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e2f83de2d1b52bf88f64d18f233139c915df7cd9))
    - Move BRC20_Prog contract and interfaces to examples/brc20_prog_helper directory ([`d668bbb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d668bbbc319d736254c1511817d42bdc0485978c))
    - Add a Simple solidity contract and a compiler script that outputs deployment and call inscriptions ([`7b477b3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7b477b3630d0a0c808df040b9c2cb160626f99a0))
    - Add BIP322 precompile and an integration test for it ([`e82272d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e82272d13d7d34bdf31cc5f96f732d27104ea283))
    - Format BRC20_Prog contract ([`9a75bde`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9a75bde48a5433492b34af799c778fd93d46120a))
    - Add BRC20, BTC, and BIP322 precompiles with initial implementations and interfaces ([`a3f0581`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a3f0581714c209924189802cd412dd61e755053c))
    - Fix hex formatting in LogED serialization to remove '0x' prefix ([`86e527e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/86e527e95052de02039c9e8dae05a2c906681c11))
    - Fix u128 order in mint and burn transactions; remove unused deposit and withdrawal endpoints ([`ae0762f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ae0762fa0f32c5e8adb46c80ea653882708b9508))
    - Add BRC20 balance retrieval ([`909cac0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/909cac09923a928bb5154e177290692c947a84b4))
    - Implement BRC20 controller with mint and burn transaction handling ([`d37fcaa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d37fcaa2a3e6e07337ce214eb01eb5bc8b86fefb))
    - Add empty deposit & withdrawal methods in ServerInstance for future use ([`b4d7938`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b4d7938fc666e4a5d8ce2a6db8710ba58daa81e8))
    - Handle optional values in transaction and block retrieval; provide defaults to prevent panics ([`1b514ad`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1b514ad54b7df1f94f4e0b723c6e0f279a6fec94))
    - Update gas limit and nonce calculation in block response; improve contract bytecode retrieval handling ([`8b544a4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8b544a4251f47dbcadcd8697ba6e24c11beeeda3))
    - Handle missing parent hash by providing a default value of zeroed bytes ([`e8219e3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e8219e3f1302a2488d842b70199771578305cab8))
    - Update block parent hash handling and improve block hash generation logic (so every block has a unique non-zero hash) ([`a905c59`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a905c59e54f4a0518be1f3795544938063dcbe86))
    - Implement Merkle tree for transaction roots and add block #0 ([`a149a81`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a149a81141a34c5ac97009bec02a591b54424138))
    - Update miner field type to AddressED and change nonce serialization to StrictPfx ([`5f88944`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5f889448c14328e0302202c4052d606221ce8f65))
    - Add contract compilation script and update package.json scripts ([`0e3db78`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0e3db78b9c134d14a7f5bc26941f2ae0ec30c5dc))
    - Fix a comment in run_custom.js ([`c1c2544`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c1c25448c6618e2b464deb5313c0b96f9b6dcc08))
    - Update contract assets folder path ([`f1eff8e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f1eff8ef038f0ce6be2082ae7fd125297e77d21e))
    - Refactor contract loading to use JSON format ([`f268e4f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f268e4f7da8b95f8cafab28923c665f97408db35))
    - Deploy BRC20_Controller as part of server initialisation ([`9e8d475`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9e8d475ea76d70685746bd48bdd6ed2036666105))
    - Add gas limit parameter to get_evm and update eth_call to enforce gas limit ([`3c2a8e9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3c2a8e944090d5d6b286d0b119b0fdcc36e0f87c))
    - Refactor transaction receipt handling to include result type and reason in TxReceiptED ([`2b8ded1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2b8ded1542cc2da45f6918e6da86e21218ce7a20))
    - Update LogED encoding test to reflect new byte length ([`4083362`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/40833622e82ae674f69f8db63483d95814f7b68c))
    - Update LogED structure to include log index ([`d571c60`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d571c600811ce8c9c896a16129990b5016b15fd7))
    - Add BlockResponseED type and update getBlockBy* RPC methods ([`66877f0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/66877f0e75c5ab86d94e3b8f1b17d9683fb69c6c))
    - Add transaction count RPCs ([`f45fd08`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f45fd082d532b9c733cd38e08c61b6195c1f991c))
    - Add eth_getLogs endpoint and LogResponseED type for log retrieval ([`e88bdae`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e88bdaefde743e7e30d20b91553a593ee1db3861))
    - Rename brc20 indexer RPC method prefixes to brc20_ ([`17ae93d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/17ae93db8ef710f6145b1fd9a11ee38c1f0708b6))
    - Implement eth_getTransactionBy* and eth_getTransactionReceipt endpoints ([`a562668`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a5626683600f77428f718d98206505895a4b8160))
    - Flush databases after commit ([`ada0a5c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ada0a5c0ad735baff813b27bcc1499ac1683c62e))
    - Update Databases to use RocksDB ([`11948a0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/11948a0e2953b6e229b0539ed71673e3d913f400))
    - Remove old caches from memory after deletion ([`934878c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/934878c9c6df5aa15c710ced00fb092d6092eec8))
    - Rename EnvWrapper to TestEnvWrapper for clarity ([`b239630`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b2396300b4727bc2c05d7c38ccaab0f16537d724))
    - Refactor test environment creation and deletion ([`440334b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/440334b1f9222fc11d8da9ddac61c28954fdcc2a))
    - Add default-members to workspace in Cargo.toml ([`a195c5c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a195c5ce8dc5911d2c86b0cb30e30409dd2a9aa1))
    - Rename package from brc20_finance to brc20_prog ([`50e7299`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/50e72991c24b49785c7a9c6282336b027325c3d7))
    - Remove unused dependencies ([`6cf9267`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6cf9267f9c04f7608fb441a403b114cb1b6bc88c))
    - Refactor BlockDatabase to use BytesWrapper for database keys ([`fa5b7e5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fa5b7e5b9626db8ab6db9267667ea4741b255543))
    - Add new JSON-RPC methods for gas estimation and storage retrieval, update existing method signatures ([`efca574`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/efca574ea91cab7a367a6ff5db0e20422504136c))
    - Don't store unchanged values in the history caches to minimise disk usage ([`37142fd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/37142fd156b7327e5fc042297e5f790b0a3efb83))
    - Refactor some RPC methods to use hexadecimal format and update related tests, add some empty/static eth_ endpoints ([`1b1deb0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1b1deb0b7695ee845afb26e534f03541a9aad533))
    - Add Brc20ProgApi trait with JSON-RPC methods and remove HTTP server ([`59b1f5f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/59b1f5f9586214b9726221e78591cbfa90edce20))
    - Fix bug in run_custom.js to accept null responses ([`0cf6c51`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0cf6c51c50cc4441af2b44ef1e2b8b8ea573b94f))
    - Add JSON-RPC server support and refactor main server and tests ([`cb9882b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cb9882b8c1c5059663f9a5c6ceb79b02684f36ba))
    - Add RPC server implementation and update finalise_block method to handle optional start time ([`7c1e898`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7c1e89819b1355ede1cb7a043565dde1967f2602))
    - Add logging for transaction addition to the next block ([`d9f263a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d9f263af2bf42e575091221f2adee242dcc0a873))
    - Extract methods into ServerInstance object to prepare for RPC server setup ([`6373e7a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6373e7a9115757d2c52056ae1f4a63cc8a5e0f0b))
    - Refactor database access in transaction handling functions ([`6e541f0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6e541f02c70143e7bfa406080d5660b6ad0f8aa7))
    - Refactor EVM module structure and update dependencies ([`b5a747c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b5a747c82ca2cd8f759dea0759dde68043b846e0))
    - Remove .DS_Store files ([`2d4ae61`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2d4ae61c2228d79f72ce37322a56b34a4876c161))
    - Migrate db crate to use block history cache ([`9867a77`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9867a77effc1d817e38b6165d24706f815007fb8))
    - Implement block history cache [WIP] ([`9b5b39a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9b5b39acda3c0714c7b73f08aea0edae8a086982))
    - Fix decode for db/src/types/address_ed.rs ([`f67f8cf`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f67f8cf24347a2b59fc501230c0339a92d766553))
    - Add tests for types with traits BytesEncode/BytesDecode ([`33a5afa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/33a5afa3afa7f5851399dd99889c675bc4702b17))
    - Split database types and format files ([`dd383cd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/dd383cd3245c5a3a41dced28c0964dbf9be6a9b2))
    - Set up to work with localhost and add requirements.txt and npm files ([`fcd6e9c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fcd6e9c992ef9f1acf2bdbbadef7392265139219))
    - Added package.json ([`976e9ec`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/976e9eceed35fb7e5069d3d228fa3a45f4b4931c))
    - Added integration_test ([`4a700fa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4a700fa4413283f298c36f043e420837b96766b5))
    - Removed Mutex'es ([`9db0c5e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9db0c5efa8423bc490ab63ae6996d6fb0676a73c))
    - Added cache with mutexes ([`504828c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/504828cd4f89c0f586ff4e6ca2eaa341402ee7a9))
    - Initial commit, no reorg protection, only code memory cache ([`8a3494e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8a3494ec688e84374f9d6217072d244def32e773))
</details>

<csr-unknown>
revm to 31.0.0serde to 1.0.228tokio to 1.48.0criterion to 0.7.0tempfile to 3.23.0revm to 31.0.0serde to 1.0.228tokio to 1.48.0criterion to 0.7.0tempfile to 3.23.0revm to 31.0.0serde to 1.0.228tokio to 1.48.0criterion to 0.7.0tempfile to 3.23.0<csr-unknown/>

## v0.15.13 (2026-02-27)

### New Features

 - <csr-id-53fb5986b8364946d54c3413d999686823ec0b47/> eth_call* and eth_estimateGas* methods now wait for block finalisation instead of failing when they are called mid-block
   * feat: Wait for block finalisation on eth_call* methods
* test: Update mid-block read contract test to wait for block finalisation
* fix: Notify waiters when clearing caches to prevent indefinite waiting, and fail gas estimation if block height change during estimation

## v0.15.12 (2026-02-27)

### Bug Fixes

 - <csr-id-a565eb49632594d5ca488fc42421d408be8e13b1/> Update eth_get_storage_at to return full length hex-encoded storage values

## v0.15.11 (2026-01-12)

<csr-id-f07023132712482b9c64101d26d1ee0eded94f95/>

### New Features

 - <csr-id-e514fa702efab5e5e6649a70aed2fce18ae5bd1f/> Enhance gas estimation logic with binary search adjustments
 - <csr-id-fb2a26711eb019df968dc4be45e378747f8c57f1/> Add gas estimation with binary search for estimateGas and estimateGasMany

### Bug Fixes

 - <csr-id-66ea2725fdff575db41a21f2dcf0e713e3e2ff4b/> Update expected gas response values in test cases
 - <csr-id-cd7e50972b9b4a01a4d0ddd15a80ed4d21bcbea9/> keep midpoint instead of gas_used * 2 as upper bound to cover some edge cases
 - <csr-id-8f05677e441c3a198f8c42b93455942c5cd49b9c/> Clean up properly if transact_one(tx) returns an error
 - <csr-id-cda1f7170c55aecfbaa7b0923374387d6b11b438/> Clean up properly if transact_one(tx) returns an error

### Refactor

 - <csr-id-f07023132712482b9c64101d26d1ee0eded94f95/> Do a first run on eth_estimateGasMany and eth_estimateGas for an early exit in case calls fail

## v0.15.10 (2025-12-17)

<csr-id-b4042bcf3e9fd1deb5470a48492982d5a382c2bd/>

### Chore

 - <csr-id-b4042bcf3e9fd1deb5470a48492982d5a382c2bd/> Update API

### New Features

 - <csr-id-0ab266a7b5041dddd60ee7dcbea1ae883457d5e0/> Implement eth_callMany with precompile data injection

## v0.15.9 (2025-12-16)

### New Features

 - <csr-id-5df80c229fd343b1dc519c975f62dce487dccc7b/> Change tx hash computation to use RLP hash from a set block height

## v0.15.8 (2025-12-11)

### New Features

 - <csr-id-cf96ca1c546b973a6ce744749d2b6bb575049d58/> Sort transactions by index in block processing

## v0.15.7 (2025-12-05)

<csr-id-4d5c856b6819c3550be054a34435a6257245350c/>

### New Features

 - <csr-id-39f18e7cfc83cbb3c48455fe35f0fff1595d2aa6/> Add debug methods for block trace string retrieval

### Test

 - <csr-id-4d5c856b6819c3550be054a34435a6257245350c/> Update tests

## v0.15.6 (2025-12-04)

<csr-id-f0042b04f388804f0c5edf413f625349fbb66c7a/>

### Chore

 - <csr-id-f0042b04f388804f0c5edf413f625349fbb66c7a/> Update dependencies and add new packages
   - Updated `alloy` to version 1.1.2 and related packages to their latest versions.
   - Updated `hyper` to version 1.8.1 and `jsonrpsee` to version 0.26.0.
   - Updated `revm` and related packages to their latest versions.
   - Updated `criterion` and `insta` to their latest versions.
   - Updated `tower-http`, `tracing`, and `tracing-subscriber` to their latest versions.
   - Updated `test-utils` dependencies to match the main project.

### Bug Fixes

 - <csr-id-3ee6a869616eb9978f78dd9c266e47154e6b9cc9/> Add gas usage assertions in deploy_call, precompiles, and transact tests

## v0.15.5 (2025-12-01)

### Bug Fixes

 - <csr-id-995d6fcb20c8abb499fe7ad7ba59c7d379df01f3/> Handle None case in get_info_from_raw_tx and update related logic, invalid chain ID shouldn't return error, and just ignore the transaction

## v0.15.4 (2025-12-01)

<csr-id-d703fa1befd2b078c8a5a43a580442fb75929ca7/>

### Chore

 - <csr-id-d703fa1befd2b078c8a5a43a580442fb75929ca7/> cargo fmt

### New Features

 - <csr-id-9dc342c19a17a2d7cdcf4d9350f027707b6527fe/> Add PrecompileData struct and update eth_callMany and eth_estimateGasMany methods to include precompile_data parameter

### Documentation

 - <csr-id-6499438a4283a261bce0cc83cf4d4c2b9b952eec/> Add description for txpool_content method in README.md
 - <csr-id-2d9c92345d52ffbce9e47fec7362a35d671a5885/> Update brc20_deploy, brc20_call, brc20_transact method parameters to include op_return_tx_id field

### Bug Fixes

 - <csr-id-5d42a4709f451dd651f7327269e3d1ee1adc8fce/> Sort transactions by index in Brc20ProgDatabase
 - <csr-id-19388a7c3f42cebc4bda9f271e6b2022c8d34657/> Rename getBtcTxId to getTxId for consistency in function naming
 - <csr-id-774a85fccac6c1c5489210a430bc9fca764f4403/> Update comment to clarify placeholder usage for block hash in get_transaction_and_block_hash_with_overrides

## v0.15.3 (2025-11-05)

### Bug Fixes

 - <csr-id-0cdb5cabc4a5aee37445d22080f337ad956ee1b2/> Update public API (serde -> serde_core)

## v0.15.2 (2025-11-05)

### New Features

 - <csr-id-e765e287038b6a45aac9f50ef7d59e01fde85b69/> Add revm-bytecode dependency to avoid bringing the whole revm for non-server version and update imports

## v0.15.1 (2025-11-05)

### New Features

 - <csr-id-303738ef0a5f5389512d7f71f41e7d1a47a25b6b/> Update alloy to v1.1.0, revm to v31.0.0, remove revm_state and refactor accordingly
   - Updated dependencies in Cargo.toml to their latest versions, including:

## v0.15.0 (2025-10-14)

### New Features

 - <csr-id-887ec1c8d69fe63b3dab107b4892a2ab8e2694e2/> Enhance BRC20 precompiles with op_return_tx_id support (enabled after the prague upgrade)
   - Updated `get_evm` to accept `current_op_return_tx_id` and pass it to `BRC20Precompiles`.

## v0.14.0 (2025-10-13)

<csr-id-5e700a7cf08be8db9756a1f5ff9719c82aa91efb/>
<csr-id-eb99d55fdf4570d67d9027ce55713250dc4d326f/>

### New Features

 - <csr-id-41d6c24dedbc0620e484d4ad8e85f7e90ec5f902/> use the block height parameter to eth_call and eth_estimateGas
 - <csr-id-7354a7ee0a1ef87b2faeccf51c91f38745b52c33/> implement dynamic EVM spec selection based on block number and network

### Bug Fixes

 - <csr-id-d06a19ce4b873e02a5151fea7a75a6e366dab8ee/> ensure pending transactions are committed, cleared, and reorganized in Brc20ProgDatabase
 - <csr-id-048a6b5e953bb6fbbd1a009dcda02e4135595971/> allow "mainnet" and "bitcoin" to be used interchangeably
 - <csr-id-0fedc1f96abecb33b51813c872d307f239fefd0f/> update generate_block to accept block_hash as a parameter
 - <csr-id-63e964b29bcd091ecbae6dffe3d7acad3d160466/> reorder block hash setting to avoid race conditions

### Refactor

 - <csr-id-5e700a7cf08be8db9756a1f5ff9719c82aa91efb/> change inscription_id and inscription_byte_len from Option to required fields in API and update database structures

### Test

 - <csr-id-eb99d55fdf4570d67d9027ce55713250dc4d326f/> add unit test for Prague spec BLS precompiles inclusion

## v0.13.0 (2025-10-01)

### Bug Fixes

 - <csr-id-af550d39e12924cf1e5cf26b032fbf49318c2528/> revert version bump, this is automated now
 - <csr-id-60d600b7e42cd3e0dc5a1ccf9cb02ef23aa80f2a/> bump version to 0.13.0 in Cargo.toml

### New Features (BREAKING)

 - <csr-id-d5d74dc396b28fa20232e46a69f56a8a3ed1524d/> Remove BRC20 balance precompile

## v0.12.0 (2025-09-30)

<csr-id-e23f25e5274bbd203746a9610082544da75e3b4e/>

### Bug Fixes

 - <csr-id-baad2eb9bbb3c323637d4ed0f060edf82ecf188e/> serialize transaction type into hex
 - <csr-id-755f0b98654d099ea8e48695906c27cc39bdba32/> update homepage URL in Cargo.toml

### Refactor

 - <csr-id-e23f25e5274bbd203746a9610082544da75e3b4e/> remove legacy fields from TxReceiptED struct

## v0.11.6 (2025-09-23)

<csr-id-5eb4dfd331b085c44fb6d10a928d67cb26cd5b2a/>
<csr-id-d52f56d695795cd43b9968b8fb3f45bc192a1988/>
<csr-id-1ecb9e92b98b2de2d4615c5f6c1325e58e8ee3eb/>

### Other

 - <csr-id-5eb4dfd331b085c44fb6d10a928d67cb26cd5b2a/> remove release configuration file and associated pre-release hooks to allow regeneration

### Other

 - <csr-id-1ecb9e92b98b2de2d4615c5f6c1325e58e8ee3eb/> Add changelog file (generated by cargo smart-release)

### Bug Fixes

 - <csr-id-631d84b09b8ffcd4a75227b372a680fcf95d77c3/> update pre-release hook to correctly format CHANGELOG.md generation
 - <csr-id-3999a309aedb4bd739743ce58a1af5e81a4904c1/> update pre-release hook to correctly generate and add CHANGELOG.md

### Chore

 - <csr-id-d52f56d695795cd43b9968b8fb3f45bc192a1988/> add release configuration file for versioning and changelog generation

## v0.11.5 (2025-09-23)

<csr-id-3eb4db14aae057f56ddfd978fa332c01e623c57e/>
<csr-id-de610e27f74bb3ef6950816519689ed05ad1ccf4/>

### Chore

 - <csr-id-3eb4db14aae057f56ddfd978fa332c01e623c57e/> Release brc20-prog version 0.11.5

### New Features (BREAKING)

 - <csr-id-45a6dc89a8e7905df8bc693c602067c44604a397/> Remove BRC20 balance precompile and update version to 0.12.0
   Associated proposal: https://github.com/bestinslot-xyz/brc20-proposals/blob/main/002-prog-brc20-precompile-removal/index.md

### New Features

 - <csr-id-5ad7fca362376f7466ac7b425c0ef82843722779/> enhance error handling for eth_call and eth_estimateGas with status codes and detailed messages in RPC server

### Bug Fixes

 - <csr-id-69c059efb075cbac4ac2623ea06eba7badce21de/> update read method signature to use lifetime annotation
 - <csr-id-47b99ccb210fac54adb7138f87d85fd71189ef08/> update read method signature to use lifetime annotation
 - <csr-id-c823151e76db2c0e9dc0ca630ef9f24742aebf26/> remove test data files for BRC20 balance calls

### Refactor

 - <csr-id-de610e27f74bb3ef6950816519689ed05ad1ccf4/> generate receipts for invalid transactions

## v0.11.4 (2025-09-20)

<csr-id-96f7476b28ab71fe02e27b47886975725abaf427/>
<csr-id-33612b040d0920a4cd817ef9278cd291acbf5ecb/>
<csr-id-6b68978593114382ba8993a96fd90e78d014336e/>
<csr-id-ffd5d864ccef9f40dc4b8b065026fe30f7844061/>
<csr-id-3ccf4f51ed13d3123d243f922b8f37e89a8166a1/>
<csr-id-0705d0af565737c211ace3288eaa7215331fd3bc/>
<csr-id-55867806867426a9d738e95be3273b444f05f2a8/>
<csr-id-f82269722861855063e9e7d411456efac1e7b086/>
<csr-id-1c998031c385aed14bcac6d16a6ce676d784c6df/>
<csr-id-8a9e7194c3c56dfcfada975f4aae175d648682e7/>
<csr-id-723e992b4dcf1e853f78d9b2b4d7a0938e23a06a/>
<csr-id-c7b0017bcdce7de6d83571944c2a8ad4ac0229cd/>
<csr-id-e6e81baaa3a25b067e46aaf44856a6da57d460b4/>
<csr-id-641626a921b3c3dfbcf78ecd0fb95d1d622c039a/>
<csr-id-e89e27ceaf73f604bc767bc7eb4f728544a18002/>
<csr-id-3246f34c7cfabf308b14286f20064748af8a2941/>
<csr-id-8957d48ddb8ac104edd5467e0ef33c9ef09d2859/>
<csr-id-0bb4d936e680f489141887ab73ee4977da7f2409/>

### Chore

 - <csr-id-96f7476b28ab71fe02e27b47886975725abaf427/> cargo fmt

### Documentation

 - <csr-id-2b677ae2a402f0cf8f0a20365734835f44757124/> Simplify cargo run command in README
   Removed the 'server' feature requirement from the run command, as it's now enabled by default
 - <csr-id-aaa224991d058c1fbc7cc671b0f45300a6d9924f/> Add warning about ticker case sensitivity and format in README
 - <csr-id-865544daf41a3e0baeb70e10c5349132de9b268c/> Add caution note about basic auth and secure connections in README

### New Features

 - <csr-id-b098ae0a779cfc0335a0828b9bdd09bafcb9d3a9/> Implement web3_ api in Brc20ProgApi
 - <csr-id-ffd7e2ea0ccb42840722e38489e43158659336d6/> Add a global values database to Brc20ProgDatabase and disallow multiple reorgs by storing a global maximum reorg height
 - <csr-id-a74ffad81aa965518922484709913b26f8378aae/> Refactor transaction handling by removing unnecessary fields, and simplify the code
   - Updated the `TxED` struct to use the new `Signature` struct, simplifying the transaction data structure.

### Bug Fixes

 - <csr-id-7837caa6e4a9dc0206e6139b52db763fac0b8001/> bump version to 0.11.4
 - <csr-id-267d7c2b560902bd76bce4aff43a41c1d11c8ff7/> remove flag from RawBytes#value() function
 - <csr-id-c220e12057772b6909d25d5c740a735524e8aed0/> add step to build with no default features in CI workflow
 - <csr-id-bac9950915b272dc4790f799c74571fc30a48df3/> bump version to 0.11.3
 - <csr-id-e5f38403bdb7d2ca75b5cef4f8626d1192ed8b80/> update keccak256 import from revm to alloy
 - <csr-id-d33734def8cab2d5f99c6f89d58b3c9f5e822249/> bump version to 0.11.2
 - <csr-id-386448cd2681a53feaf7bc0a36d286abc50e231d/> update public_api snapshot
 - <csr-id-af1d7a35ea86f68e63212eb9db8ac3910a5397fd/> Correct version string format in web3_client_version method
 - <csr-id-0141858a32ade666db80d2cf71d819767ebf582b/> Improve error handling in contract address resolution
 - <csr-id-e84fe1b027595660cc0dba5bb3c0bf9579c5f577/> Refactor Brc20ProgDatabase to remove redundant timestamp and gas methods, they are stored in the block database already
 - <csr-id-4c796c680ca41ddaf790f59aae0501f7f973b56a/> Migrate back the bip322 crate after the fix and remove the fork
 - <csr-id-ec8835f2d0c0b9dd6dbd428610bf19f86aa884cb/> Simplify reorg logic in BlockHistoryCache and UintED serialization/deserialization
 - <csr-id-5fd5b4d25fcea6477074e226450e256e464dae5d/> Enhance bytecode handling with error checks during deserialization and decoding
 - <csr-id-30cf858ff7fa8c47baba10f8088c9f94f7686a63/> Update inconsistent docs
 - <csr-id-38da95965b63ed09d1463584e9a5b8a6b56c10bf/> Fix test_reorg_all_blocks to reorg only 10 blocks
 - <csr-id-42bd00d5b8d2da8ecdbea0c35fa998e9884dfb53/> Bump version of brc20-prog to 0.10.4
 - <csr-id-cd05179fa05f730fa44eec52ab04b4cd5db255f5/> Use a fork of bip322 for a patch where it crashes
 - <csr-id-da66f74e19a68c9f21cdc571f3e6605707d3f8a9/> Ensure latest value retrieval from the cache is never empty, and reorgs also panic if there's an attempt to empty the cache
 - <csr-id-bca334dd1cafc4d0ddd8ce6fb369c119f38e6ef8/> Update BITCOIN_RPC_URL to use address
 - <csr-id-fb28f3092294946c257d7b7aa5301fc42277e9a2/> Replace "DB Error" with a static error message and panic instead of returning an error, as it's not recoverable
 - <csr-id-4ab801b940a721c5ffd0126c70bcb9c4821fa1f8/> Couple of bug fixes for reorg handling on database
   - Use get_next_block_height method in DatabaseCommit methods to fix an off-by-one error in reorgs, which might incorrectly keep old values

### Refactor

 - <csr-id-33612b040d0920a4cd817ef9278cd291acbf5ecb/> Update public api and fix a visibility error
 - <csr-id-6b68978593114382ba8993a96fd90e78d014336e/> Add configurable request/response size limits and batch request settings to RPC server
 - <csr-id-ffd5d864ccef9f40dc4b8b065026fe30f7844061/> Remove batch request limit from RPC server configuration
 - <csr-id-3ccf4f51ed13d3123d243f922b8f37e89a8166a1/> ditto, improve error message for Bitcoin RPC status check failure
 - <csr-id-0705d0af565737c211ace3288eaa7215331fd3bc/> Improve error handling in Bitcoin RPC status check
 - <csr-id-55867806867426a9d738e95be3273b444f05f2a8/> Rename DB_PATH to BRC20_PROG_DB_PATH for consistency in env.sample
 - <csr-id-f82269722861855063e9e7d411456efac1e7b086/> Enhance RPC server configuration with request/response size limits and batch request settings
 - <csr-id-1c998031c385aed14bcac6d16a6ce676d784c6df/> Print the error message in get_brc20_balance function when it fails
 - <csr-id-8a9e7194c3c56dfcfada975f4aae175d648682e7/> Bump version to 0.11.1 and update binary name in Cargo.toml
 - <csr-id-723e992b4dcf1e853f78d9b2b4d7a0938e23a06a/> Update eth_getCode to return 0x if there's no code present for compliance
 - <csr-id-c7b0017bcdce7de6d83571944c2a8ad4ac0229cd/> Modify eth_call in RpcServer to return 0x or the revert reason for compliance
 - <csr-id-e6e81baaa3a25b067e46aaf44856a6da57d460b4/> Add legacy fields back in BlockResponseED and TxReceiptED for backwards compatibility
 - <csr-id-641626a921b3c3dfbcf78ecd0fb95d1d622c039a/> Add input size limit warning for bip322_verify_precompile to prevent excessive resource usage
 - <csr-id-e89e27ceaf73f604bc767bc7eb4f728544a18002/> Enhance ConfigDatabase by adding caching and implementing flush functionality
 - <csr-id-3246f34c7cfabf308b14286f20064748af8a2941/> Use catch_unwind for BRC20 balance server checks to handle potential panics gracefully
 - <csr-id-8957d48ddb8ac104edd5467e0ef33c9ef09d2859/> update public api
 - <csr-id-0bb4d936e680f489141887ab73ee4977da7f2409/> Simplify balance retrieval by removing error handling from get_brc20_balance function, it now panics when balance server is unreachable or provides invalid data instead of silencing it

## v0.11.0-audit (2025-09-09)

<csr-id-96f7476b28ab71fe02e27b47886975725abaf427/>
<csr-id-e6e81baaa3a25b067e46aaf44856a6da57d460b4/>
<csr-id-641626a921b3c3dfbcf78ecd0fb95d1d622c039a/>
<csr-id-e89e27ceaf73f604bc767bc7eb4f728544a18002/>
<csr-id-3246f34c7cfabf308b14286f20064748af8a2941/>
<csr-id-8957d48ddb8ac104edd5467e0ef33c9ef09d2859/>
<csr-id-0bb4d936e680f489141887ab73ee4977da7f2409/>

### Chore

 - <csr-id-96f7476b28ab71fe02e27b47886975725abaf427/> cargo fmt

### Documentation

 - <csr-id-aaa224991d058c1fbc7cc671b0f45300a6d9924f/> Add warning about ticker case sensitivity and format in README
 - <csr-id-865544daf41a3e0baeb70e10c5349132de9b268c/> Add caution note about basic auth and secure connections in README

### New Features

 - <csr-id-ffd7e2ea0ccb42840722e38489e43158659336d6/> Add a global values database to Brc20ProgDatabase and disallow multiple reorgs by storing a global maximum reorg height
 - <csr-id-a74ffad81aa965518922484709913b26f8378aae/> Refactor transaction handling by removing unnecessary fields, and simplify the code
   - Updated the `TxED` struct to use the new `Signature` struct, simplifying the transaction data structure.

### Bug Fixes

 - <csr-id-0141858a32ade666db80d2cf71d819767ebf582b/> Improve error handling in contract address resolution
 - <csr-id-e84fe1b027595660cc0dba5bb3c0bf9579c5f577/> Refactor Brc20ProgDatabase to remove redundant timestamp and gas methods, they are stored in the block database already
 - <csr-id-4c796c680ca41ddaf790f59aae0501f7f973b56a/> Migrate back the bip322 crate after the fix and remove the fork
 - <csr-id-ec8835f2d0c0b9dd6dbd428610bf19f86aa884cb/> Simplify reorg logic in BlockHistoryCache and UintED serialization/deserialization
 - <csr-id-5fd5b4d25fcea6477074e226450e256e464dae5d/> Enhance bytecode handling with error checks during deserialization and decoding
 - <csr-id-30cf858ff7fa8c47baba10f8088c9f94f7686a63/> Update inconsistent docs
 - <csr-id-38da95965b63ed09d1463584e9a5b8a6b56c10bf/> Fix test_reorg_all_blocks to reorg only 10 blocks
 - <csr-id-42bd00d5b8d2da8ecdbea0c35fa998e9884dfb53/> Bump version of brc20-prog to 0.10.4
 - <csr-id-cd05179fa05f730fa44eec52ab04b4cd5db255f5/> Use a fork of bip322 for a patch where it crashes
 - <csr-id-da66f74e19a68c9f21cdc571f3e6605707d3f8a9/> Ensure latest value retrieval from the cache is never empty, and reorgs also panic if there's an attempt to empty the cache
 - <csr-id-bca334dd1cafc4d0ddd8ce6fb369c119f38e6ef8/> Update BITCOIN_RPC_URL to use address
 - <csr-id-fb28f3092294946c257d7b7aa5301fc42277e9a2/> Replace "DB Error" with a static error message and panic instead of returning an error, as it's not recoverable
 - <csr-id-4ab801b940a721c5ffd0126c70bcb9c4821fa1f8/> Couple of bug fixes for reorg handling on database
   - Use get_next_block_height method in DatabaseCommit methods to fix an off-by-one error in reorgs, which might incorrectly keep old values

### Refactor

 - <csr-id-e6e81baaa3a25b067e46aaf44856a6da57d460b4/> Add legacy fields back in BlockResponseED and TxReceiptED for backwards compatibility
 - <csr-id-641626a921b3c3dfbcf78ecd0fb95d1d622c039a/> Add input size limit warning for bip322_verify_precompile to prevent excessive resource usage
 - <csr-id-e89e27ceaf73f604bc767bc7eb4f728544a18002/> Enhance ConfigDatabase by adding caching and implementing flush functionality
 - <csr-id-3246f34c7cfabf308b14286f20064748af8a2941/> Use catch_unwind for BRC20 balance server checks to handle potential panics gracefully
 - <csr-id-8957d48ddb8ac104edd5467e0ef33c9ef09d2859/> update public api
 - <csr-id-0bb4d936e680f489141887ab73ee4977da7f2409/> Simplify balance retrieval by removing error handling from get_brc20_balance function, it now panics when balance server is unreachable or provides invalid data instead of silencing it

## v0.10.2 (2025-07-31)

<csr-id-84bded4dda8da50c0147a7cb7e2c85a3cc4006d5/>

### New Features

 - <csr-id-1570523a05e047509c5f3d708aead7bc2466c668/> Add brc20_transact method for sending raw signed transactions and update parameter names

### Bug Fixes

 - <csr-id-46536f22a131d86447f5f9c53364b81b47759b9a/> Bump version to 0.10.2 for brc20-prog
 - <csr-id-decba4477ab08c2b068228b2f92559898710ddb3/> Refactor Bitcoin RPC validation logic and remove unnecessary checks
 - <csr-id-87bc7e2b1d7c80157404779cd7febe6236ede1ba/> Clarify initialization process for BRC20_Controller and empty block handling in README
 - <csr-id-fb8eca2807a000a76b955c750ae3c6a58b41494a/> Bump version to 0.10.1 and ensure raw_block_ed module is conditionally compiled
 - <csr-id-c680e6e97cc981bfdefe3ac134b87401822bd21a/> Update Discord (badge was broken) and add Telegram link to README

### Other

 - <csr-id-84bded4dda8da50c0147a7cb7e2c85a3cc4006d5/> Add support for raw signed transaction inscriptions and update relevant methods in README

## v0.10.0 (2025-07-15)

<csr-id-94aec21d6b51ed8e6cd0cb6e9d7ff2438a306c6b/>

### Chore

 - <csr-id-94aec21d6b51ed8e6cd0cb6e9d7ff2438a306c6b/> Cargo fmt

### New Features

 - <csr-id-b512cee522053941c800a785d537a8485a2f8dbd/> Add debug_getRaw* methods in the API and update version to 0.10.0

### Bug Fixes

 - <csr-id-887b39bada8e575f971a2cfc9f0e891f793a0003/> Prefix hex-encoded strings with "0x" in RawBlock methods
 - <csr-id-31e796498bdf9a3bfe5f8381c2d5b5072aa3bdf6/> Correct comment formatting for RawBytes struct documentation

## v0.9.0 (2025-07-04)

<csr-id-cf8921d1528c3c6615546805709237b544224061/>
<csr-id-c78bae850abbd0587cc2d79a4ebc656c0ac497c2/>
<csr-id-1cfa3d16f06a5144ef846338a96cef64e7adfdbe/>
<csr-id-63ed832295355e7883c67046303446a7421d98f9/>
<csr-id-2113bdd73430a8c3757e537cb63124a6cb33dfab/>
<csr-id-444dd4425529239ff86962c72ff3d2dab820b788/>
<csr-id-d7be56ee4ae29832267cbb0ea057a7deab2332b6/>
<csr-id-7bdaa5dce7389cf1d8ab367db29da631d5d81286/>
<csr-id-5a8d6a98c3017761b034efde804b18c8f367c07f/>
<csr-id-5a21c928ddc19ed69543566fcf99aaaf90a12c38/>
<csr-id-a807ccc301e1abe19174c54c690fd19bf0316ec3/>
<csr-id-b682785eb92ed5570c191be80db56f8b6bdca273/>
<csr-id-2b3fd06e250c19cea6e6148ac301d7cd980bd92b/>
<csr-id-8b82397cd220c31cbb22f04b4c612adf34f4d0ad/>
<csr-id-ebc6f9f78a5f309606e2c27a913903b589b9fc5b/>
<csr-id-358147b546742441d3256e4b8b2e6db6df9845b5/>
<csr-id-1a7130d676e11e17e63efad46a8f99dc969e42fd/>
<csr-id-42d13763d3b40e9f31509722414ad620d573eb27/>
<csr-id-fcfded73f27b4cd16fabeb65a72aae4e4ab61c14/>
<csr-id-fcacaa47e1b840caa5d8341ddf573f1b28af028a/>
<csr-id-234579584061e13984fa6ec11137083e2b2c2207/>
<csr-id-a7f5a36e31a9be01919cd29e0d1352263cbb1172/>
<csr-id-9d2db98b4aa92bb21fcedbf92a52c4304b8aae35/>
<csr-id-a2eb1ad26ae2efc700182100e5ff63be82b65eac/>
<csr-id-71b3790b3e836557c0fb0d6b83f3e792f6b93532/>
<csr-id-a9da2511194989cee40310c742bace9c6279cc73/>
<csr-id-e5b8d4fc84cca21daa131fe9832160e5c3c10157/>
<csr-id-b70947fa7ee21cf0b2ecd50d928250df39e36474/>
<csr-id-72ecdac855e12cd7434a87e011337fd6cf2c702b/>
<csr-id-cacc59a37580f6e96b8ae25e18f69609efb46bcb/>
<csr-id-8e6f676036169a9a5b4d6d3de9065bce98f2212c/>
<csr-id-cbbc6ee00f9505c4afebca8a5ab8d2275ae3a4b5/>
<csr-id-b082d52f90f18ed229eccaac996b13be1e2bbd55/>
<csr-id-0a8d187c541f8b934a0a09bebba6248f061c41f5/>
<csr-id-d87ba8ca04d87c46e069b1056f0b95b5149159af/>
<csr-id-87739b5b7a5d56a402e75a8c051c62f753ef4539/>
<csr-id-4f31079272c6659a18c6f2cc64a4c5f7eb844974/>
<csr-id-37f309447e4e5e2b4fece64581577df1507819f9/>
<csr-id-e82db5ed40ee0f4c8783cb888a55347ade98f7d4/>
<csr-id-92a8d4933ac88d7b0eeb63080e8ca3336384f460/>
<csr-id-a6c02ef52253f0b4996b6984249774516d41a834/>
<csr-id-774800d0290fae4c48e6857353f343fcb9013cf6/>
<csr-id-eade2ffaa72cbc6e94646a47f4ed7780e51e5389/>
<csr-id-3ba65ce66c4840136133589681891c64768a158e/>
<csr-id-258df9ac2cadbbfc554cfd6788818ee64f966d78/>
<csr-id-e1ccb2c49953256d3c9f439ac2a51d8afdbba25a/>
<csr-id-c353e81880c792e404a8708fb5a0f86f5d4cc2e3/>
<csr-id-1c0b7be3e0332d9f16d89669e9e815a3213e1072/>
<csr-id-592390af2345610797f49727878f3311be8ad936/>

### Chore

 - <csr-id-cf8921d1528c3c6615546805709237b544224061/> Run cargo fmt
 - <csr-id-c78bae850abbd0587cc2d79a4ebc656c0ac497c2/> Bump version to 0.6.2
 - <csr-id-1cfa3d16f06a5144ef846338a96cef64e7adfdbe/> Update package metadata in Cargo.toml
 - <csr-id-63ed832295355e7883c67046303446a7421d98f9/> Move documentation for Brc20ProgApi to top-level module
 - <csr-id-2113bdd73430a8c3757e537cb63124a6cb33dfab/> Move super:: imports to crates
 - <csr-id-444dd4425529239ff86962c72ff3d2dab820b788/> Run cargo fmt
 - <csr-id-d7be56ee4ae29832267cbb0ea057a7deab2332b6/> Update dependencies and update RPC authorization middleware to latest jsonrpsee library
 - <csr-id-7bdaa5dce7389cf1d8ab367db29da631d5d81286/> Bump version of brc20_prog to 0.2.0

### Documentation

 - <csr-id-2dbee8d8fa12d39481ebc3d4ef315f1861f4490c/> Remove warning about zstd compression stack size requirement from README, as it now uses Vec instead of a fixed size slice
 - <csr-id-bfd38291287b48012828ec78ac3825518103312d/> Update README to include note about BRC20 indexer integration and enhance operation checks for deploy/call inscriptions
 - <csr-id-2e2588bfff348861c6ea8b436b8bfeb9b65cdc98/> Update module documentation for clarity and completeness

### New Features

<csr-id-73829880e7bd0c7001aa4dfcb3ef50f4e8dd3354/>
<csr-id-1cf9d8f490bcc7d79d4e795391004984130d8f53/>
<csr-id-dbdb3e8dabffb1e8002277c283558316fb4bc005/>
<csr-id-0dc7ecf30d7847ce14878897c35516e3fcb08d02/>
<csr-id-5cfdd0033e181de47d25ab1c64322ae69cee190f/>
<csr-id-ec43700f156c33691a8813ba027848a75ef1baff/>
<csr-id-ddb953702f29431084aed2e4255d73e05c291b76/>
<csr-id-c275f7580792d73520a5ed914b8b80639354cd04/>
<csr-id-8091f91eed4e9d6b7d96637778e5ab10af4a6561/>
<csr-id-ff3f48ca2c8d3ab0df493abacd0f4e948047f82b/>
<csr-id-21e20d6d4080a372ac0407dbbf8faf67d3890956/>
<csr-id-21ec1b685a5ad56e737328358157511eba75b6df/>
<csr-id-15a9698993fbc6bf98de20b1456cdb9b25cf817b/>
<csr-id-42c82e851c787d3f2d442798c48cfa4b2c8c553c/>
<csr-id-0fdf4f4fe23c51da2d0167a8078845d96105f74d/>
<csr-id-88a3f8b1d8445511b9b30bd55a2e2bdbadd85cad/>
<csr-id-5de832a30482391e2cebd34558ab337faab9a536/>
<csr-id-16585cb3ffb436ae158277675254db73bc71e161/>
<csr-id-b268d3780c370ae03bcb822fc27189438aa17deb/>
<csr-id-987aaea26382ccb339dd1e5fd317a63cb46efb05/>
<csr-id-af5e1fb534492e97607f9a4ae5635f1a6f4bbbd6/>
<csr-id-575c942972771a36694c8e4dcbd0b34405733e19/>
<csr-id-b5ffdace5bf2c8e3e01e1a9120b88dc2cc55a32c/>
<csr-id-e7158f1dc2924b81a251a5a604a1a48b068f0c14/>
<csr-id-d94fe38a9f08faf432c611c7d32b4b2fe86849dc/>

 - <csr-id-0d46345de13d5a744c24c7a1ba738bae0889e4e0/> Add "brc20_transact" method to indexer methods list
 - <csr-id-c193e11a0ade6a3956eb45f53197a41230be4b97/> Store and serve v,r,s fields from raw signed transactions, bump version to 0.9.0
 - <csr-id-3857b9e137adf1a1a58f2994d80748be65843aab/> Enhance BRC2.0 with pending transaction management
   - Added a new database structure to handle pending transactions, allowing for efficient retrieval and management of transactions by account and nonce.

### Bug Fixes

 - <csr-id-5fb4b57577d7aade6ffc034b9619ab7e1c30a67a/> Handle extra padding in base64 decoding for inscription data
 - <csr-id-d9d19fcd218ed919d3ccacb585fdbd74d107e440/> Correctly increment transaction index in pending transaction execution
 - <csr-id-09d14ce3430cefd954e80c23587af6953999cf55/> Update DB_VERSION to 5 in configuration
 - <csr-id-0226d532f1ae7220ae26cc3e4abf596a1ee513dc/> Update ecrecover call to use adjusted v value and remove ignore from tests
 - <csr-id-6b118cb4c0cf1e86defc3b1c1645a3a805e3fedb/> Ignore raw tx tests as they fail to recover the correct address at the moment
 - <csr-id-6d5c6a25652d50db586290f37555bdce275c4c70/> Remove conditional flags for some imports
 - <csr-id-c116c5953069e7977cef271fabd9ded3c88cf475/> Remove optional flag from nada and zstd-safe dependencies, as clients can use these to generate encoded data
 - <csr-id-5a3c817bf9d84606d576c5d1dc0135c3622edaff/> Rename db.rs > brc20_prog_database.rs
 - <csr-id-849c728f202779c1e8f3a49b37977f9ca8b67b64/> Update LastBlockInfo to track total processing time
 - <csr-id-dd94405867d6d8a3365ebc3e222e33f6c94722e9/> Reduce gas usage from 100k to 20k for bip322 precompile
 - <csr-id-1c23942dc35b74766987e9cb88d3e3b0c263685e/> Check if the transaction is in the future in last sat location precompile
 - <csr-id-0eddafdebe0aac5820ad197660237d5621d86e8e/> Readjust stack size to 4MB in GitHub Actions workflow (again)
 - <csr-id-7d15efcc680b03fafefe666f5aba9c69ab1f49a4/> Update alloy-sol-types dependency version to 1.1.0
 - <csr-id-9f5041fe8888f913ede0cee97346983a585101e2/> Update result field name from resultBytes to output in transaction receipt
 - <csr-id-f1b17bf4e314977409b9798d4d8f9b5e4edfdb14/> Remove unused import of E from std::f32::consts
 - <csr-id-77749c615722d500acbc9c3ed688b71dc6518dee/> Return an error if block range is too large for eth_getLogs
 - <csr-id-29f15b3386da729fcc63aa34d887ab341d6403d0/> Better eth_getLogs topic and address handling
 - <csr-id-7e5551954845676dbe5a2f6f3c0f8827bdb51ed9/> Remove unused rust-embed feature "include-exclude"
 - <csr-id-89b0c72b4290171047dbfe020380433b039ea7d7/> Use dotenvy instead of unmainained dotenv, avoid embedding the source for brc20_controller contract to reduce binary size
 - <csr-id-55a890ccbcc0e438c7607e2c2cd02432df4c6c1a/> Simplify assertions in tests for AccountInfoED conversions
 - <csr-id-89607b82052d28cb90320b0e553a5f58b18e5f42/> Serialize block nonce to full hex length
 - <csr-id-bcb3d7683185fc8f85ac76617505b797697f43be/> Make authorisation logic exclusive to brc20_ methods, and fix TraceED encoding
 - <csr-id-4ffc7623a9f2d50d08818f87e3f41f4686759479/> Simplify serialization and encoding of BytecodeED by using original_bytes
 - <csr-id-b82bfb5d8bdcb0ebcf27cca405928f4543e56265/> Return BytecodeED from eth_getCode, so it gets serialised without the trailing zeroes
 - <csr-id-b9caea9e4aa23b03f7f0238f96431355ee6c670a/> Handle missing contract addresses by using Address::ZERO in Brc20ProgApiServer
   This way, receipts for invalid inscriptions are also recorded in the chain
 - <csr-id-bb1c3fd5e6b1ed50bd93710c3ed60cb922af4cf0/> Move Id import to tests
 - <csr-id-1836eeed4680639dc2e13a285098d198a047ca62/> Return correct ID ffrom RpcAuthAllowEth
 - <csr-id-747fe034d3eb1ab48f449e0bc77cb9a854e7f942/> Fix HttpNonBlockingAuth for allow_all(...)

### Refactor

 - <csr-id-5a8d6a98c3017761b034efde804b18c8f367c07f/> Remove brc20 event simulator test
 - <csr-id-5a21c928ddc19ed69543566fcf99aaaf90a12c38/> Replace CHAIN_ID with Brc20ProgConfig.chain_id in various modules to enable using different chain IDs for signet and mainnet
 - <csr-id-a807ccc301e1abe19174c54c690fd19bf0316ec3/> Update raw tx handling to use alloy TxLegacy type
 - <csr-id-b682785eb92ed5570c191be80db56f8b6bdca273/> Fix cargo doc
 - <csr-id-2b3fd06e250c19cea6e6148ac301d7cd980bd92b/> Move conditional compilation for Brc20ProgDatabase to a separate line
 - <csr-id-8b82397cd220c31cbb22f04b4c612adf34f4d0ad/> Disable logging for common block retrieval calls to reduce noise
 - <csr-id-ebc6f9f78a5f309606e2c27a913903b589b9fc5b/> Update database version from 3 to 4 in configuration, as previous gas values need adjustment
 - <csr-id-358147b546742441d3256e4b8b2e6db6df9845b5/> Move gas usage and history size constants to the global package, adjust gas usage for precompiles sending an RPC
 - <csr-id-1a7130d676e11e17e63efad46a8f99dc969e42fd/> Update BRC20 types to replace EncodedBytes with InscriptionBytes and EthBytes for improved clarity and functionality
 - <csr-id-42d13763d3b40e9f31509722414ad620d573eb27/> Update decompression buffer to use dynamic Vec instead of fixed array
 - <csr-id-fcfded73f27b4cd16fabeb65a72aae4e4ab61c14/> Update benchmark functions by removing unused transaction detail cases, add min stack requirement to readme
 - <csr-id-fcacaa47e1b840caa5d8341ddf573f1b28af028a/> Add more test cases for bitcoin mainnet rpc tests and run bitcoin rpc tests sequentially
 - <csr-id-234579584061e13984fa6ec11137083e2b2c2207/> Update BTC precompiles to use faster transaction and block height retrieval methods
 - <csr-id-a7f5a36e31a9be01919cd29e0d1352263cbb1172/> Refactor to use a "Transaction" instance instead of "GetRawTransactionResult" in BTC precompiles
 - <csr-id-9d2db98b4aa92bb21fcedbf92a52c4304b8aae35/> Remove redundant names from instrument attributes in Brc20ProgApiServer implementation
 - <csr-id-a2eb1ad26ae2efc700182100e5ff63be82b65eac/> Refactor Brc20ProgApiClient methods to include 'brc20_' and 'eth_' prefixes and update related client calls
   - Adjusted public API snapshot to match the updated API method signatures.
 - <csr-id-71b3790b3e836557c0fb0d6b83f3e792f6b93532/> Reorganise engine into its own package alongside the evm, and public api types into the API package, add a public_api test to check for changes in the API
 - <csr-id-a9da2511194989cee40310c742bace9c6279cc73/> Replace Wrapper types with ED types in API and RPC server
 - <csr-id-e5b8d4fc84cca21daa131fe9832160e5c3c10157/> Handle JSON-RPC error code -5 without panicking
 - <csr-id-b70947fa7ee21cf0b2ecd50d928250df39e36474/> Remove minimum gas limit constant and handle inscription byte length in tests
 - <csr-id-72ecdac855e12cd7434a87e011337fd6cf2c702b/> Move dotenv call to Brc20ProgConfig
 - <csr-id-cacc59a37580f6e96b8ae25e18f69609efb46bcb/> Update BRC20Precompiles initialization to use a constructor
 - <csr-id-8e6f676036169a9a5b4d6d3de9065bce98f2212c/> Simplify revm integration
   - Add a TracingInspector to use later
   - block.prevrandao is now set to current block hash
 - <csr-id-cbbc6ee00f9505c4afebca8a5ab8d2275ae3a4b5/> Simplify EVM setup
 - <csr-id-b082d52f90f18ed229eccaac996b13be1e2bbd55/> Make tests ignore the result of .initialise(...)
 - <csr-id-0a8d187c541f8b934a0a09bebba6248f061c41f5/> Remove unused test imports, use alloy_primitives instead of revm re-exports
 - <csr-id-d87ba8ca04d87c46e069b1056f0b95b5149159af/> Split db/mod.rs into db.rs / mod.rs, rename server_instance.rs to engine.rs
 - <csr-id-87739b5b7a5d56a402e75a8c051c62f753ef4539/> Use read/write locks for evm database and propagate errors into the RPC layer to wrap and display
   - Renamed ServerInstance to BRC20ProgEngine (will rename the file in a follow-up to keep the diff)
   - Refactored error handling for various operations e.g. transaction validation and block finalization.
   - Created a shared data structure for access to database and block information, RwLocks now allow faster/parallelised reads
 - <csr-id-4f31079272c6659a18c6f2cc64a4c5f7eb844974/> Update module visibility across various files

### Test

 - <csr-id-37f309447e4e5e2b4fece64581577df1507819f9/> Change out of order test to send 3 transactions
 - <csr-id-e82db5ed40ee0f4c8783cb888a55347ade98f7d4/> Split benchmark for BTC precompiles
 - <csr-id-92a8d4933ac88d7b0eeb63080e8ca3336384f460/> Split env setup in tests into .env.mainnet and .env.signet
 - <csr-id-a6c02ef52253f0b4996b6984249774516d41a834/> Add mainnet precompile tests (works with a .env setup)
 - <csr-id-774800d0290fae4c48e6857353f343fcb9013cf6/> Add microsecond and second tests for gas processing times in opcode generator test
 - <csr-id-eade2ffaa72cbc6e94646a47f4ed7780e51e5389/> Migrate precompile benchmarks to Rust
 - <csr-id-3ba65ce66c4840136133589681891c64768a158e/> Move test utilities to their own package, add a benchmark for calling the bip322_verify precompile
   Refactored test files to utilize the new test-utils crate for loading files and spawning test servers.
 - <csr-id-258df9ac2cadbbfc554cfd6788818ee64f966d78/> Add BRC20 balance test and a deploy/call test
 - <csr-id-e1ccb2c49953256d3c9f439ac2a51d8afdbba25a/> Update test cases to use a non-zero value for inscription byte length
 - <csr-id-c353e81880c792e404a8708fb5a0f86f5d4cc2e3/> Add serialization test for BlockResponseED
 - <csr-id-1c0b7be3e0332d9f16d89669e9e815a3213e1072/> Add unit tests for BRC20ProgEngine and RpcServer methods
 - <csr-id-592390af2345610797f49727878f3311be8ad936/> Add unit test for HttpNonBlockingAuth allow_all method

## v0.1.1 (2025-04-03)

<csr-id-614c74a32ad5014e28ffd9653bd1e7b01c35ff7e/>
<csr-id-74f4220a55bd44792b97f046e25001e3eb1fda36/>
<csr-id-01db2c03dc4e68febb2b4bda49d35c02243473d8/>
<csr-id-78f440e8b620b064b2eb58cf3cbf6f7d003295fe/>
<csr-id-372b8ac38a8923fa5f7888f1501bc75d50057683/>
<csr-id-c7d62041ae510be583dca05bf640d5602b5a50c9/>
<csr-id-4afd2a04d0c26474c1e51f423f35531090b8a096/>
<csr-id-77a47aeec08aff62136efcf62c56b9a93e6cabe8/>
<csr-id-eae60ba505b1d2dd3babf9f0d8bb84306ec4023a/>
<csr-id-cdc017b16f9af0a5a2ddcbd5865280fd0a303ece/>
<csr-id-49de6943e0d880c66f500817f651cc8a23572318/>
<csr-id-2ce9df6c937052fcd1fc48a7e7e94d5f04192d30/>
<csr-id-2db1eed8876cce908913cc8367139f0214e8e8b8/>
<csr-id-590563efff5fbab51e8f4f3a0a203dc5e8981ce9/>

### Chore

 - <csr-id-614c74a32ad5014e28ffd9653bd1e7b01c35ff7e/> Bump version of brc20_prog to 0.1.1 in Cargo files

### New Features

 - <csr-id-a9c9937cf7672dbe785e168c3f66fd79908527b8/> Add data_or_input method to EthCall for improved data handling
 - <csr-id-a8f6405b9b39c3ee072f6a184b412fa8f79152b2/> Ignore transactions that happen in the future in tx details precompile
   Use a PrecompileCall struct to pass parameters to precompiles
 - <csr-id-ca2f1f985eedfafa7818636a48a63b431982523a/> change ticker parameter type to Bytes in transaction loading functions and add ticker_as_bytes utility function
 - <csr-id-e31ed1f5a98da2b9f727fcd20d1e759ab85cc7f3/> include assigned gas limit in txes
 - <csr-id-c28bbac6d2e396fd243303057d242042ab759ba0/> verify new blocks don't already exist in the database, and generate new hashes for new blocks with hashes 0x00..00
 - <csr-id-ad1712ad302fcc8ff5563c3f29d77646986f6e65/> add gas_price and syncing methods to Brc20ProgApi (to enhance explorer compatibility)

### Bug Fixes

 - <csr-id-58319779251015d3e41de54e60f8120388f49e85/> update BRC20 locked pkscript precompile result for integration tests
 - <csr-id-8d2ad1903c655bb587765e103956448e69258f93/> Include data in returned error instance for eth_call and eth_estimateGas
 - <csr-id-6a8ced344cfb268dedd7f3da10a62b7101d87971/> prevent deadlock in get_block_by_hash by releasing the mutex after accessing it
 - <csr-id-5ca15e20b0de70cce6a3bd51aee64a3ac8daf7e3/> rename block_hash field to blockHash for consistency in serialization
 - <csr-id-54b6c26278ddc1c9e1f11ce56cdecb2ddd9e4618/> convert contract bytecode using to_string to add the 0x prefix
 - <csr-id-542c795a660479973068c7e602ea2be3c89213a8/> rename block_hash field for serialization

### Refactor

 - <csr-id-74f4220a55bd44792b97f046e25001e3eb1fda36/> Rename variables for clarity in various modules
 - <csr-id-01db2c03dc4e68febb2b4bda49d35c02243473d8/> Simplify data decoding in decode_brc20_balance_result function
 - <csr-id-78f440e8b620b064b2eb58cf3cbf6f7d003295fe/> Refactor calls to unwrap() for Result/Options to avoid unnecessary panics
   Use rust-bitcoincore-rpc library for precompiles
 - <csr-id-372b8ac38a8923fa5f7888f1501bc75d50057683/> remove unused FromHex import in brc20_controller
 - <csr-id-c7d62041ae510be583dca05bf640d5602b5a50c9/> rename verify_block_does_not_exist to require_block_does_not_exist
 - <csr-id-4afd2a04d0c26474c1e51f423f35531090b8a096/> change account parameter type to AddressWrapper in get_transaction_count method
 - <csr-id-77a47aeec08aff62136efcf62c56b9a93e6cabe8/> add chain_id and tx_type fields to TxED and update related structures, run cargo fmt
 - <csr-id-eae60ba505b1d2dd3babf9f0d8bb84306ec4023a/> change transaction index parameter to be optional
 - <csr-id-cdc017b16f9af0a5a2ddcbd5865280fd0a303ece/> update EthCall struct to make 'from' field optional and use zero in contract calls if from is empty
 - <csr-id-49de6943e0d880c66f500817f651cc8a23572318/> Use LogResponse for eth_getTransactionReceipt for correct eth JSON-RPC format and add v,r,s fields to eth_getTransaction* methods
 - <csr-id-2ce9df6c937052fcd1fc48a7e7e94d5f04192d30/> streamline byte, string and address conversions to use into() and parse() for type flexibility and consistency
 - <csr-id-2db1eed8876cce908913cc8367139f0214e8e8b8/> simplify encode method to return Vec<u8> directly instead of Result, as it never fails
 - <csr-id-590563efff5fbab51e8f4f3a0a203dc5e8981ce9/> update Brc20ProgApi call and estimate_gas methods to use EthCall struct and optional input data

## v0.1.0 (2025-03-31)

<csr-id-f395953ca03507ca9f37fc77a6068dc11a885a9e/>

### Documentation

 - <csr-id-aae8ab89d1ec80974e5aa6d515731a9981550ae0/> update README with warnings about BRC20 balance server and BIP322_Verifier contract limitations

### New Features

 - <csr-id-1f31eed331e1e3568a1ac37ef8f52d1fdbb01085/> Add transfer tx for sample erc20 contract
 - <csr-id-3adf9ddcd3881d591ae6b6f5e9464e252cfdd4a7/> Add an ERC20 contract example

### Bug Fixes

 - <csr-id-19654aa69c0ffa2f310457b8ce19dcf5c54ba98b/> mine_blocks genesis block count
 - <csr-id-7dfc45d0afd0af9cdc488f2a71f887d4e9860758/> Update import path for ERC20 contract in BobCoin.sol
 - <csr-id-be008714bda08dc2be67f5485d678a0ed1ab2fde/> Show float numbers in test_generator.py
 - <csr-id-b302e55b65807960e28c760eafd8d64a5f193073/> Make get_range end key comparison exclusive

### Refactor

 - <csr-id-f395953ca03507ca9f37fc77a6068dc11a885a9e/> Rename block count variables for clarity

