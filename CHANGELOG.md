# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.15.8 (2025-12-11)

### New Features

 - <csr-id-cf96ca1c546b973a6ce744749d2b6bb575049d58/> Sort transactions by index in block processing

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 5 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sort transactions by index in block processing ([`cf96ca1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cf96ca1c546b973a6ce744749d2b6bb575049d58))
</details>

## v0.15.7 (2025-12-05)

<csr-id-4d5c856b6819c3550be054a34435a6257245350c/>

### New Features

 - <csr-id-39f18e7cfc83cbb3c48455fe35f0fff1595d2aa6/> Add debug methods for block trace string retrieval

### Test

 - <csr-id-4d5c856b6819c3550be054a34435a6257245350c/> Update tests

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 1 day passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.15.7 ([`88b574a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/88b574a3bc2a7aec279004b68dcee2cdba0bdbfd))
    - Update tests ([`4d5c856`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4d5c856b6819c3550be054a34435a6257245350c))
    - Add debug methods for block trace string retrieval ([`39f18e7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/39f18e7cfc83cbb3c48455fe35f0fff1595d2aa6))
    - Reapply "chore: Update dependencies and add new packages" ([`08058e8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/08058e8da3b830b17c93f0478a96d8e541aef0b6))
    - Revert "chore: Update dependencies and add new packages" ([`af51836`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/af51836cf4e02a44a3616fa6244d5f43309854a0))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.15.6 ([`6bfbf80`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6bfbf8093944d421f343fbd7b88cd81f1976fc1d))
    - Update dependencies and add new packages ([`f0042b0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f0042b04f388804f0c5edf413f625349fbb66c7a))
    - Add gas usage assertions in deploy_call, precompiles, and transact tests ([`3ee6a86`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3ee6a869616eb9978f78dd9c266e47154e6b9cc9))
</details>

## v0.15.5 (2025-12-01)

### Bug Fixes

 - <csr-id-995d6fcb20c8abb499fe7ad7ba59c7d379df01f3/> Handle None case in get_info_from_raw_tx and update related logic, invalid chain ID shouldn't return error, and just ignore the transaction

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.15.5 ([`f4a3299`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f4a32990d0927cc65d1844d4f8845db2f541c556))
    - Handle None case in get_info_from_raw_tx and update related logic, invalid chain ID shouldn't return error, and just ignore the transaction ([`995d6fc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/995d6fcb20c8abb499fe7ad7ba59c7d379df01f3))
</details>

## v0.15.4 (2025-12-01)

<csr-id-d703fa1befd2b078c8a5a43a580442fb75929ca7/>

### Chore

 - <csr-id-d703fa1befd2b078c8a5a43a580442fb75929ca7/> cargo fmt

### Documentation

 - <csr-id-6499438a4283a261bce0cc83cf4d4c2b9b952eec/> Add description for txpool_content method in README.md
 - <csr-id-2d9c92345d52ffbce9e47fec7362a35d671a5885/> Update brc20_deploy, brc20_call, brc20_transact method parameters to include op_return_tx_id field

### Bug Fixes

 - <csr-id-5d42a4709f451dd651f7327269e3d1ee1adc8fce/> Sort transactions by index in Brc20ProgDatabase
 - <csr-id-19388a7c3f42cebc4bda9f271e6b2022c8d34657/> Rename getBtcTxId to getTxId for consistency in function naming

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.15.4 ([`9fae28d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9fae28d1d960fefa060031f37a2bdf0cf28c23df))
    - Cargo fmt ([`d703fa1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d703fa1befd2b078c8a5a43a580442fb75929ca7))
    - Sort transactions by index in Brc20ProgDatabase ([`5d42a47`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5d42a4709f451dd651f7327269e3d1ee1adc8fce))
    - Add description for txpool_content method in README.md ([`6499438`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6499438a4283a261bce0cc83cf4d4c2b9b952eec))
    - Update brc20_deploy, brc20_call, brc20_transact method parameters to include op_return_tx_id field ([`2d9c923`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2d9c92345d52ffbce9e47fec7362a35d671a5885))
    - Rename getBtcTxId to getTxId for consistency in function naming ([`19388a7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/19388a7c3f42cebc4bda9f271e6b2022c8d34657))
</details>

## v0.15.3 (2025-11-05)

### Bug Fixes

 - <csr-id-0cdb5cabc4a5aee37445d22080f337ad956ee1b2/> Update public API (serde -> serde_core)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.15.3 ([`7e1ef06`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7e1ef060e4ea1ae1559dfa52c3b7cfab60dd80ab))
    - Update public API (serde -> serde_core) ([`0cdb5ca`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0cdb5cabc4a5aee37445d22080f337ad956ee1b2))
</details>

## v0.15.2 (2025-11-05)

### New Features

 - <csr-id-e765e287038b6a45aac9f50ef7d59e01fde85b69/> Add revm-bytecode dependency to avoid bringing the whole revm for non-server version and update imports

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.15.2 ([`96df24e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/96df24ede4898720d9a8147910bf00d48e7212d7))
    - Add revm-bytecode dependency to avoid bringing the whole revm for non-server version and update imports ([`e765e28`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e765e287038b6a45aac9f50ef7d59e01fde85b69))
</details>

## v0.15.1 (2025-11-05)

### New Features

 - <csr-id-303738ef0a5f5389512d7f71f41e7d1a47a25b6b/> Update alloy to v1.1.0, revm to v31.0.0, remove revm_state and refactor accordingly
   - Updated dependencies in Cargo.toml to their latest versions, including:

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.15.1 ([`a7f58b7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a7f58b7ccefe8815b53a7f3913b8319a096aad24))
    - Update alloy to v1.1.0, revm to v31.0.0, remove revm_state and refactor accordingly ([`303738e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/303738ef0a5f5389512d7f71f41e7d1a47a25b6b))
</details>

## v0.15.0 (2025-10-14)

### New Features

 - <csr-id-887ec1c8d69fe63b3dab107b4892a2ab8e2694e2/> Enhance BRC20 precompiles with op_return_tx_id support (enabled after the prague upgrade)
   - Updated `get_evm` to accept `current_op_return_tx_id` and pass it to `BRC20Precompiles`.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.15.0 ([`ff66def`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ff66def9291cc0dd0583649d2e8f4104429e561e))
    - Enhance BRC20 precompiles with op_return_tx_id support (enabled after the prague upgrade) ([`887ec1c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/887ec1c8d69fe63b3dab107b4892a2ab8e2694e2))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 7 calendar days.
 - 12 days passed between releases.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.14.0 ([`8cc6e08`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8cc6e089b1658d780608c0edacf2909f3503c50e))
    - Change inscription_id and inscription_byte_len from Option to required fields in API and update database structures ([`5e700a7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5e700a7cf08be8db9756a1f5ff9719c82aa91efb))
    - Ensure pending transactions are committed, cleared, and reorganized in Brc20ProgDatabase ([`d06a19c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d06a19ce4b873e02a5151fea7a75a6e366dab8ee))
    - Add unit test for Prague spec BLS precompiles inclusion ([`eb99d55`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/eb99d55fdf4570d67d9027ce55713250dc4d326f))
    - Use the block height parameter to eth_call and eth_estimateGas ([`41d6c24`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/41d6c24dedbc0620e484d4ad8e85f7e90ec5f902))
    - Allow "mainnet" and "bitcoin" to be used interchangeably ([`048a6b5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/048a6b5e953bb6fbbd1a009dcda02e4135595971))
    - Implement dynamic EVM spec selection based on block number and network ([`7354a7e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7354a7ee0a1ef87b2faeccf51c91f38745b52c33))
    - Update generate_block to accept block_hash as a parameter ([`0fedc1f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0fedc1f96abecb33b51813c872d307f239fefd0f))
    - Reorder block hash setting to avoid race conditions ([`63e964b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/63e964b29bcd091ecbae6dffe3d7acad3d160466))
</details>

## v0.13.0 (2025-10-01)

### Bug Fixes

 - <csr-id-af550d39e12924cf1e5cf26b032fbf49318c2528/> revert version bump, this is automated now
 - <csr-id-60d600b7e42cd3e0dc5a1ccf9cb02ef23aa80f2a/> bump version to 0.13.0 in Cargo.toml

### New Features (BREAKING)

 - <csr-id-d5d74dc396b28fa20232e46a69f56a8a3ed1524d/> Remove BRC20 balance precompile

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 1 day passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.13.0 ([`ad4971c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ad4971c7705a9a5174c9c5093d494d5b36ae8670))
    - Remove BRC20 balance precompile ([`d5d74dc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d5d74dc396b28fa20232e46a69f56a8a3ed1524d))
    - Revert version bump, this is automated now ([`af550d3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/af550d39e12924cf1e5cf26b032fbf49318c2528))
    - Merge branch 'main' into remove-brc20-balance-precompile ([`48bf33c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/48bf33c20b74a76afbd6769739cda9094530935e))
    - Bump version to 0.13.0 in Cargo.toml ([`60d600b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/60d600b7e42cd3e0dc5a1ccf9cb02ef23aa80f2a))
</details>

## v0.12.0 (2025-09-30)

<csr-id-e23f25e5274bbd203746a9610082544da75e3b4e/>

### Bug Fixes

 - <csr-id-baad2eb9bbb3c323637d4ed0f060edf82ecf188e/> serialize transaction type into hex
 - <csr-id-755f0b98654d099ea8e48695906c27cc39bdba32/> update homepage URL in Cargo.toml

### Refactor

 - <csr-id-e23f25e5274bbd203746a9610082544da75e3b4e/> remove legacy fields from TxReceiptED struct

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 6 calendar days.
 - 6 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.12.0 ([`cae8a3d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cae8a3d5e28cd2d150c2d77cfff976d55c4c4bd7))
    - Remove legacy fields from TxReceiptED struct ([`e23f25e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e23f25e5274bbd203746a9610082544da75e3b4e))
    - Serialize transaction type into hex ([`baad2eb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/baad2eb9bbb3c323637d4ed0f060edf82ecf188e))
    - Merge branch 'main' into remove-brc20-balance-precompile ([`d6b190c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d6b190ccb7db42a76b409faed43a50cbba1e09dc))
    - Update homepage URL in Cargo.toml ([`755f0b9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/755f0b98654d099ea8e48695906c27cc39bdba32))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog v0.11.6 ([`235db26`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/235db265b47ff91e8e85b56906099819e9df5f53))
    - Add changelog file (generated by cargo smart-release) ([`1ecb9e9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1ecb9e92b98b2de2d4615c5f6c1325e58e8ee3eb))
    - Remove release configuration file and associated pre-release hooks to allow regeneration ([`5eb4dfd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5eb4dfd331b085c44fb6d10a928d67cb26cd5b2a))
    - Update pre-release hook to correctly format CHANGELOG.md generation ([`631d84b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/631d84b09b8ffcd4a75227b372a680fcf95d77c3))
    - Update pre-release hook to correctly generate and add CHANGELOG.md ([`3999a30`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3999a309aedb4bd739743ce58a1af5e81a4904c1))
    - Release v0.11.6 ([`dafba1d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/dafba1d85bb02aae0f26f923c680d4872da5b78a))
    - Add release configuration file for versioning and changelog generation ([`d52f56d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d52f56d695795cd43b9968b8fb3f45bc192a1988))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 2 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release brc20-prog version 0.11.5 ([`3eb4db1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3eb4db14aae057f56ddfd978fa332c01e623c57e))
    - Generate receipts for invalid transactions ([`de610e2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/de610e27f74bb3ef6950816519689ed05ad1ccf4))
    - Enhance error handling for eth_call and eth_estimateGas with status codes and detailed messages in RPC server ([`5ad7fca`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5ad7fca362376f7466ac7b425c0ef82843722779))
    - Update read method signature to use lifetime annotation ([`47b99cc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/47b99ccb210fac54adb7138f87d85fd71189ef08))
    - Update read method signature to use lifetime annotation ([`69c059e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/69c059efb075cbac4ac2623ea06eba7badce21de))
    - Remove test data files for BRC20 balance calls ([`c823151`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c823151e76db2c0e9dc0ca630ef9f24742aebf26))
    - Remove BRC20 balance precompile and update version to 0.12.0 ([`45a6dc8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/45a6dc89a8e7905df8bc693c602067c44604a397))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 21 commits contributed to the release over the course of 10 calendar days.
 - 11 days passed between releases.
 - 21 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump version to 0.11.4 ([`7837caa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7837caa6e4a9dc0206e6139b52db763fac0b8001))
    - Remove flag from RawBytes#value() function ([`267d7c2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/267d7c2b560902bd76bce4aff43a41c1d11c8ff7))
    - Add step to build with no default features in CI workflow ([`c220e12`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c220e12057772b6909d25d5c740a735524e8aed0))
    - Bump version to 0.11.3 ([`bac9950`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bac9950915b272dc4790f799c74571fc30a48df3))
    - Update keccak256 import from revm to alloy ([`e5f3840`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e5f38403bdb7d2ca75b5cef4f8626d1192ed8b80))
    - Bump version to 0.11.2 ([`d33734d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d33734def8cab2d5f99c6f89d58b3c9f5e822249))
    - Update public_api snapshot ([`386448c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/386448cd2681a53feaf7bc0a36d286abc50e231d))
    - Correct version string format in web3_client_version method ([`af1d7a3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/af1d7a35ea86f68e63212eb9db8ac3910a5397fd))
    - Implement web3_ api in Brc20ProgApi ([`b098ae0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b098ae0a779cfc0335a0828b9bdd09bafcb9d3a9))
    - Update public api and fix a visibility error ([`33612b0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/33612b040d0920a4cd817ef9278cd291acbf5ecb))
    - Add configurable request/response size limits and batch request settings to RPC server ([`6b68978`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6b68978593114382ba8993a96fd90e78d014336e))
    - Remove batch request limit from RPC server configuration ([`ffd5d86`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ffd5d864ccef9f40dc4b8b065026fe30f7844061))
    - Ditto, improve error message for Bitcoin RPC status check failure ([`3ccf4f5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3ccf4f51ed13d3123d243f922b8f37e89a8166a1))
    - Improve error handling in Bitcoin RPC status check ([`0705d0a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0705d0af565737c211ace3288eaa7215331fd3bc))
    - Rename DB_PATH to BRC20_PROG_DB_PATH for consistency in env.sample ([`5586780`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/55867806867426a9d738e95be3273b444f05f2a8))
    - Enhance RPC server configuration with request/response size limits and batch request settings ([`f822697`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f82269722861855063e9e7d411456efac1e7b086))
    - Simplify cargo run command in README ([`2b677ae`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2b677ae2a402f0cf8f0a20365734835f44757124))
    - Print the error message in get_brc20_balance function when it fails ([`1c99803`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1c998031c385aed14bcac6d16a6ce676d784c6df))
    - Bump version to 0.11.1 and update binary name in Cargo.toml ([`8a9e719`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8a9e7194c3c56dfcfada975f4aae175d648682e7))
    - Update eth_getCode to return 0x if there's no code present for compliance ([`723e992`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/723e992b4dcf1e853f78d9b2b4d7a0938e23a06a))
    - Modify eth_call in RpcServer to return 0x or the revert reason for compliance ([`c7b0017`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c7b0017bcdce7de6d83571944c2a8ad4ac0229cd))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 24 commits contributed to the release over the course of 28 calendar days.
 - 40 days passed between releases.
 - 24 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Add legacy fields back in BlockResponseED and TxReceiptED for backwards compatibility ([`e6e81ba`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e6e81baaa3a25b067e46aaf44856a6da57d460b4))
    - Add input size limit warning for bip322_verify_precompile to prevent excessive resource usage ([`641626a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/641626a921b3c3dfbcf78ecd0fb95d1d622c039a))
    - Add warning about ticker case sensitivity and format in README ([`aaa2249`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/aaa224991d058c1fbc7cc671b0f45300a6d9924f))
    - Enhance ConfigDatabase by adding caching and implementing flush functionality ([`e89e27c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e89e27ceaf73f604bc767bc7eb4f728544a18002))
    - Use catch_unwind for BRC20 balance server checks to handle potential panics gracefully ([`3246f34`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3246f34c7cfabf308b14286f20064748af8a2941))
    - Update public api ([`8957d48`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8957d48ddb8ac104edd5467e0ef33c9ef09d2859))
    - Simplify balance retrieval by removing error handling from get_brc20_balance function, it now panics when balance server is unreachable or provides invalid data instead of silencing it ([`0bb4d93`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0bb4d936e680f489141887ab73ee4977da7f2409))
    - Improve error handling in contract address resolution ([`0141858`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0141858a32ade666db80d2cf71d819767ebf582b))
    - Add caution note about basic auth and secure connections in README ([`865544d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/865544daf41a3e0baeb70e10c5349132de9b268c))
    - Add a global values database to Brc20ProgDatabase and disallow multiple reorgs by storing a global maximum reorg height ([`ffd7e2e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ffd7e2ea0ccb42840722e38489e43158659336d6))
    - Refactor transaction handling by removing unnecessary fields, and simplify the code ([`a74ffad`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a74ffad81aa965518922484709913b26f8378aae))
    - Cargo fmt ([`96f7476`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/96f7476b28ab71fe02e27b47886975725abaf427))
    - Refactor Brc20ProgDatabase to remove redundant timestamp and gas methods, they are stored in the block database already ([`e84fe1b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e84fe1b027595660cc0dba5bb3c0bf9579c5f577))
    - Migrate back the bip322 crate after the fix and remove the fork ([`4c796c6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4c796c680ca41ddaf790f59aae0501f7f973b56a))
    - Simplify reorg logic in BlockHistoryCache and UintED serialization/deserialization ([`ec8835f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ec8835f2d0c0b9dd6dbd428610bf19f86aa884cb))
    - Enhance bytecode handling with error checks during deserialization and decoding ([`5fd5b4d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5fd5b4d25fcea6477074e226450e256e464dae5d))
    - Update inconsistent docs ([`30cf858`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/30cf858ff7fa8c47baba10f8088c9f94f7686a63))
    - Fix test_reorg_all_blocks to reorg only 10 blocks ([`38da959`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/38da95965b63ed09d1463584e9a5b8a6b56c10bf))
    - Bump version of brc20-prog to 0.10.4 ([`42bd00d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/42bd00d5b8d2da8ecdbea0c35fa998e9884dfb53))
    - Use a fork of bip322 for a patch where it crashes ([`cd05179`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cd05179fa05f730fa44eec52ab04b4cd5db255f5))
    - Ensure latest value retrieval from the cache is never empty, and reorgs also panic if there's an attempt to empty the cache ([`da66f74`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/da66f74e19a68c9f21cdc571f3e6605707d3f8a9))
    - Update BITCOIN_RPC_URL to use address ([`bca334d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bca334dd1cafc4d0ddd8ce6fb369c119f38e6ef8))
    - Replace "DB Error" with a static error message and panic instead of returning an error, as it's not recoverable ([`fb28f30`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fb28f3092294946c257d7b7aa5301fc42277e9a2))
    - Couple of bug fixes for reorg handling on database ([`4ab801b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4ab801b940a721c5ffd0126c70bcb9c4821fa1f8))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 12 calendar days.
 - 16 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump version to 0.10.2 for brc20-prog ([`46536f2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/46536f22a131d86447f5f9c53364b81b47759b9a))
    - Refactor Bitcoin RPC validation logic and remove unnecessary checks ([`decba44`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/decba4477ab08c2b068228b2f92559898710ddb3))
    - Clarify initialization process for BRC20_Controller and empty block handling in README ([`87bc7e2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/87bc7e2b1d7c80157404779cd7febe6236ede1ba))
    - Add support for raw signed transaction inscriptions and update relevant methods in README ([`84bded4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/84bded4dda8da50c0147a7cb7e2c85a3cc4006d5))
    - Add brc20_transact method for sending raw signed transactions and update parameter names ([`1570523`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1570523a05e047509c5f3d708aead7bc2466c668))
    - Bump version to 0.10.1 and ensure raw_block_ed module is conditionally compiled ([`fb8eca2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fb8eca2807a000a76b955c750ae3c6a58b41494a))
    - Update Discord (badge was broken) and add Telegram link to README ([`c680e6e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c680e6e97cc981bfdefe3ac134b87401822bd21a))
</details>

## v0.10.0 (2025-07-15)

<csr-id-94aec21d6b51ed8e6cd0cb6e9d7ff2438a306c6b/>

### Chore

 - <csr-id-94aec21d6b51ed8e6cd0cb6e9d7ff2438a306c6b/> Cargo fmt

### New Features

 - <csr-id-b512cee522053941c800a785d537a8485a2f8dbd/> Add debug_getRaw* methods in the API and update version to 0.10.0

### Bug Fixes

 - <csr-id-887b39bada8e575f971a2cfc9f0e891f793a0003/> Prefix hex-encoded strings with "0x" in RawBlock methods
 - <csr-id-31e796498bdf9a3bfe5f8381c2d5b5072aa3bdf6/> Correct comment formatting for RawBytes struct documentation

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 5 calendar days.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Prefix hex-encoded strings with "0x" in RawBlock methods ([`887b39b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/887b39bada8e575f971a2cfc9f0e891f793a0003))
    - Cargo fmt ([`94aec21`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/94aec21d6b51ed8e6cd0cb6e9d7ff2438a306c6b))
    - Add debug_getRaw* methods in the API and update version to 0.10.0 ([`b512cee`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b512cee522053941c800a785d537a8485a2f8dbd))
    - Correct comment formatting for RawBytes struct documentation ([`31e7964`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/31e796498bdf9a3bfe5f8381c2d5b5072aa3bdf6))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 119 commits contributed to the release over the course of 88 calendar days.
 - 91 days passed between releases.
 - 108 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Remove brc20 event simulator test ([`5a8d6a9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5a8d6a98c3017761b034efde804b18c8f367c07f))
    - Add "brc20_transact" method to indexer methods list ([`0d46345`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0d46345de13d5a744c24c7a1ba738bae0889e4e0))
    - Store and serve v,r,s fields from raw signed transactions, bump version to 0.9.0 ([`c193e11`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c193e11a0ade6a3956eb45f53197a41230be4b97))
    - Replace CHAIN_ID with Brc20ProgConfig.chain_id in various modules to enable using different chain IDs for signet and mainnet ([`5a21c92`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5a21c928ddc19ed69543566fcf99aaaf90a12c38))
    - Handle extra padding in base64 decoding for inscription data ([`5fb4b57`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5fb4b57577d7aade6ffc034b9619ab7e1c30a67a))
    - Change out of order test to send 3 transactions ([`37f3094`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/37f309447e4e5e2b4fece64581577df1507819f9))
    - Correctly increment transaction index in pending transaction execution ([`d9d19fc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d9d19fcd218ed919d3ccacb585fdbd74d107e440))
    - Update DB_VERSION to 5 in configuration ([`09d14ce`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/09d14ce3430cefd954e80c23587af6953999cf55))
    - Enhance BRC2.0 with pending transaction management ([`3857b9e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3857b9e137adf1a1a58f2994d80748be65843aab))
    - Update raw tx handling to use alloy TxLegacy type ([`a807ccc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a807ccc301e1abe19174c54c690fd19bf0316ec3))
    - Update ecrecover call to use adjusted v value and remove ignore from tests ([`0226d53`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0226d532f1ae7220ae26cc3e4abf596a1ee513dc))
    - Ignore raw tx tests as they fail to recover the correct address at the moment ([`6b118cb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6b118cb4c0cf1e86defc3b1c1645a3a805e3fedb))
    - Remove conditional flags for some imports ([`6d5c6a2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6d5c6a25652d50db586290f37555bdce275c4c70))
    - Remove optional flag from nada and zstd-safe dependencies, as clients can use these to generate encoded data ([`c116c59`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c116c5953069e7977cef271fabd9ded3c88cf475))
    - Support using RawBytes and Base64Bytes types together in BRC2.0 API ([`7382988`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/73829880e7bd0c7001aa4dfcb3ef50f4e8dd3354))
    - Run cargo fmt ([`cf8921d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cf8921d1528c3c6615546805709237b544224061))
    - Add brc20_transact endpoint to handle signed evm transactions, currently disabled via config ([`1cf9d8f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1cf9d8f490bcc7d79d4e795391004984130d8f53))
    - Fix cargo doc ([`b682785`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b682785eb92ed5570c191be80db56f8b6bdca273))
    - Move conditional compilation for Brc20ProgDatabase to a separate line ([`2b3fd06`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2b3fd06e250c19cea6e6148ac301d7cd980bd92b))
    - Rename db.rs > brc20_prog_database.rs ([`5a3c817`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5a3c817bf9d84606d576c5d1dc0135c3622edaff))
    - Gate server-side features under "server" feature, this reduces client compilation time ([`dbdb3e8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/dbdb3e8dabffb1e8002277c283558316fb4bc005))
    - Bump version to 0.6.2 ([`c78bae8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c78bae850abbd0587cc2d79a4ebc656c0ac497c2))
    - Implement TryFrom trait for AddressED and FixedBytesED, add is_zero method to UintED ([`0dc7ecf`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0dc7ecf30d7847ce14878897c35516e3fcb08d02))
    - Disable logging for common block retrieval calls to reduce noise ([`8b82397`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8b82397cd220c31cbb22f04b4c612adf34f4d0ad))
    - Implement TryFrom trait for AddressED and FixedBytesED, add is_zero method to UintED ([`5cfdd00`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5cfdd0033e181de47d25ab1c64322ae69cee190f))
    - Update database version from 3 to 4 in configuration, as previous gas values need adjustment ([`ebc6f9f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ebc6f9f78a5f309606e2c27a913903b589b9fc5b))
    - Move gas usage and history size constants to the global package, adjust gas usage for precompiles sending an RPC ([`358147b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/358147b546742441d3256e4b8b2e6db6df9845b5))
    - Update BRC20 types to replace EncodedBytes with InscriptionBytes and EthBytes for improved clarity and functionality ([`1a7130d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1a7130d676e11e17e63efad46a8f99dc969e42fd))
    - Remove warning about zstd compression stack size requirement from README, as it now uses Vec instead of a fixed size slice ([`2dbee8d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2dbee8d8fa12d39481ebc3d4ef315f1861f4490c))
    - Update decompression buffer to use dynamic Vec instead of fixed array ([`42d1376`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/42d13763d3b40e9f31509722414ad620d573eb27))
    - Update benchmark functions by removing unused transaction detail cases, add min stack requirement to readme ([`fcfded7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fcfded73f27b4cd16fabeb65a72aae4e4ab61c14))
    - Split benchmark for BTC precompiles ([`e82db5e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e82db5ed40ee0f4c8783cb888a55347ade98f7d4))
    - Add more test cases for bitcoin mainnet rpc tests and run bitcoin rpc tests sequentially ([`fcacaa4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fcacaa47e1b840caa5d8341ddf573f1b28af028a))
    - Update BTC precompiles to use faster transaction and block height retrieval methods ([`2345795`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/234579584061e13984fa6ec11137083e2b2c2207))
    - Refactor to use a "Transaction" instance instead of "GetRawTransactionResult" in BTC precompiles ([`a7f5a36`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a7f5a36e31a9be01919cd29e0d1352263cbb1172))
    - Add EVM call gas limit configuration ([`ec43700`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ec43700f156c33691a8813ba027848a75ef1baff))
    - Split env setup in tests into .env.mainnet and .env.signet ([`92a8d49`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/92a8d4933ac88d7b0eeb63080e8ca3336384f460))
    - Add mainnet precompile tests (works with a .env setup) ([`a6c02ef`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a6c02ef52253f0b4996b6984249774516d41a834))
    - Update LastBlockInfo to track total processing time ([`849c728`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/849c728f202779c1e8f3a49b37977f9ca8b67b64))
    - Add microsecond and second tests for gas processing times in opcode generator test ([`774800d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/774800d0290fae4c48e6857353f343fcb9013cf6))
    - Reduce gas usage from 100k to 20k for bip322 precompile ([`dd94405`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/dd94405867d6d8a3365ebc3e222e33f6c94722e9))
    - Check if the transaction is in the future in last sat location precompile ([`1c23942`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1c23942dc35b74766987e9cb88d3e3b0c263685e))
    - Migrate precompile benchmarks to Rust ([`eade2ff`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/eade2ffaa72cbc6e94646a47f4ed7780e51e5389))
    - Move test utilities to their own package, add a benchmark for calling the bip322_verify precompile ([`3ba65ce`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3ba65ce66c4840136133589681891c64768a158e))
    - Update README to include note about BRC20 indexer integration and enhance operation checks for deploy/call inscriptions ([`bfd3829`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bfd38291287b48012828ec78ac3825518103312d))
    - Add BRC20 balance test and a deploy/call test ([`258df9a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/258df9ac2cadbbfc554cfd6788818ee64f966d78))
    - Readjust stack size to 4MB in GitHub Actions workflow (again) ([`0eddafd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0eddafdebe0aac5820ad197660237d5621d86e8e))
    - Remove redundant names from instrument attributes in Brc20ProgApiServer implementation ([`9d2db98`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9d2db98b4aa92bb21fcedbf92a52c4304b8aae35))
    - Refactor Brc20ProgApiClient methods to include 'brc20_' and 'eth_' prefixes and update related client calls ([`a2eb1ad`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a2eb1ad26ae2efc700182100e5ff63be82b65eac))
    - Reduce GitHub Actions stack size to 2MB, move long responses in precompile tests into assets ([`ddb9537`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ddb953702f29431084aed2e4255d73e05c291b76))
    - Implement some precompile tests in rust ([`c275f75`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c275f7580792d73520a5ed914b8b80639354cd04))
    - Bump rust stack size in github actions for zstd tests ([`ff501b0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ff501b0f478d04ef0b1d1ac8688cb86bb41a0b53))
    - Reorganise engine into its own package alongside the evm, and public api types into the API package, add a public_api test to check for changes in the API ([`71b3790`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/71b3790b3e836557c0fb0d6b83f3e792f6b93532))
    - Add zstd compression support with 1MB buffer size, reorganise some internal variables ([`8091f91`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8091f91eed4e9d6b7d96637778e5ab10af4a6561))
    - Update package metadata in Cargo.toml ([`1cfa3d1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1cfa3d16f06a5144ef846338a96cef64e7adfdbe))
    - Update module documentation for clarity and completeness ([`2e2588b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2e2588bfff348861c6ea8b436b8bfeb9b65cdc98))
    - Move documentation for Brc20ProgApi to top-level module ([`63ed832`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/63ed832295355e7883c67046303446a7421d98f9))
    - Made brc20-prog into a library with a binary, bumped version to 0.4.0 ([`f558ed1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f558ed1c79d3992458d886e98aee156915eb2808))
    - Increase block gas limit to 4MB (block size) * 12K (gas per byte) ([`ff3f48c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ff3f48ca2c8d3ab0df493abacd0f4e948047f82b))
    - Move super:: imports to crates ([`2113bdd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2113bdd73430a8c3757e537cb63124a6cb33dfab))
    - Generate std json output for example contracts ([`bc16916`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bc16916ad3588fcf587efc7c4d5b5a81a043b8b3))
    - Refactor integration tests: Remove legacy files and update tests to use BRC20ProgClient ([`9bdfe66`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9bdfe66bc5bdacc8f13b3bfb2d329ab835f21148))
    - Replace Wrapper types with ED types in API and RPC server ([`a9da251`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a9da2511194989cee40310c742bace9c6279cc73))
    - Add deserialization for various data types ([`a4e6e4e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a4e6e4ee2299cd15a314c258f696fac046c3ff3a))
    - Update alloy-sol-types dependency version to 1.1.0 ([`7d15efc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7d15efcc680b03fafefe666f5aba9c69ab1f49a4))
    - Add functionality to retrieve inscription ID by contract address, bump db version to 2 ([`21e20d6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/21e20d6d4080a372ac0407dbbf8faf67d3890956))
    - Update result field name from resultBytes to output in transaction receipt ([`9f5041f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9f5041fe8888f913ede0cee97346983a585101e2))
    - Remove unused import of E from std::f32::consts ([`f1b17bf`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f1b17bf4e314977409b9798d4d8f9b5e4edfdb14))
    - Run cargo fmt ([`444dd44`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/444dd4425529239ff86962c72ff3d2dab820b788))
    - Handle JSON-RPC error code -5 without panicking ([`e5b8d4f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e5b8d4fc84cca21daa131fe9832160e5c3c10157))
    - Return an error if block range is too large for eth_getLogs ([`77749c6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/77749c615722d500acbc9c3ed688b71dc6518dee))
    - Update test cases to use a non-zero value for inscription byte length ([`e1ccb2c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e1ccb2c49953256d3c9f439ac2a51d8afdbba25a))
    - Better eth_getLogs topic and address handling ([`29f15b3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/29f15b3386da729fcc63aa34d887ab341d6403d0))
    - Remove minimum gas limit constant and handle inscription byte length in tests ([`b70947f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b70947fa7ee21cf0b2ecd50d928250df39e36474))
    - Update dependencies and update RPC authorization middleware to latest jsonrpsee library ([`d7be56e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d7be56ee4ae29832267cbb0ea057a7deab2332b6))
    - Handle nada encoding in inscription data (not activated until compression activation height is set) ([`21ec1b6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/21ec1b685a5ad56e737328358157511eba75b6df))
    - Add PROTOCOL_VERSION to configuration and database validation ([`15a9698`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/15a9698993fbc6bf98de20b1456cdb9b25cf817b))
    - Remove unused rust-embed feature "include-exclude" ([`7e55519`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7e5551954845676dbe5a2f6f3c0f8827bdb51ed9))
    - Use dotenvy instead of unmainained dotenv, avoid embedding the source for brc20_controller contract to reduce binary size ([`89b0c72`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/89b0c72b4290171047dbfe020380433b039ea7d7))
    - Move dotenv call to Brc20ProgConfig ([`72ecdac`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/72ecdac855e12cd7434a87e011337fd6cf2c702b))
    - Update environment variable name for BITCOIN_RPC_NETWORK, and use a denylist for RPC method auth ([`42c82e8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/42c82e851c787d3f2d442798c48cfa4b2c8c553c))
    - Add dotenv support and update dependencies ([`0fdf4f4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0fdf4f4fe23c51da2d0167a8078845d96105f74d))
    - Validate bitcoin network using bitcoin rpc ([`88a3f8b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/88a3f8b1d8445511b9b30bd55a2e2bdbadd85cad))
    - Bump version of brc20_prog to 0.2.0 ([`7bdaa5d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7bdaa5dce7389cf1d8ab367db29da631d5d81286))
    - Refactor Encode/Decode to use buffers for efficiency and code simplicity ([`a84329b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a84329be997d266e05839f614f48e4d9ed1c2e89))
    - Simplify assertions in tests for AccountInfoED conversions ([`55a890c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/55a890ccbcc0e438c7607e2c2cd02432df4c6c1a))
    - Refactor data structures and conversions for AddressED, BytecodeED, BytesED, and UintED ([`9ff10ce`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9ff10ce04f4406c510db95da65618c3231ba103d))
    - Add EVM_RECORD_TRACES to env sample ([`5de832a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5de832a30482391e2cebd34558ab337faab9a536))
    - Add an environment variable to toggle trace recording ([`16585cb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/16585cb3ffb436ae158277675254db73bc71e161))
    - Add serialization test for BlockResponseED ([`c353e81`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c353e81880c792e404a8708fb5a0f86f5d4cc2e3))
    - Serialize block nonce to full hex length ([`89607b8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/89607b82052d28cb90320b0e553a5f58b18e5f42))
    - Make authorisation logic exclusive to brc20_ methods, and fix TraceED encoding ([`bcb3d76`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bcb3d7683185fc8f85ac76617505b797697f43be))
    - Record internal transactions and add an API endpoint to return them ([`03d62f8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/03d62f8245033cc96d00d84d61a5b64a0ec30b37))
    - Update BRC20Precompiles initialization to use a constructor ([`cacc59a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cacc59a37580f6e96b8ae25e18f69609efb46bcb))
    - Simplify revm integration ([`8e6f676`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8e6f676036169a9a5b4d6d3de9065bce98f2212c))
    - Bump revm, alloy deps and migrate ABI decoding and precompile logic ([`a64bf8d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a64bf8db36bcbc0c91fe63bf38be7f9899c59585))
    - Simplify EVM setup ([`cbbc6ee`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cbbc6ee00f9505c4afebca8a5ab8d2275ae3a4b5))
    - Make tests ignore the result of .initialise(...) ([`b082d52`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b082d52f90f18ed229eccaac996b13be1e2bbd55))
    - Add unit tests for BRC20ProgEngine and RpcServer methods ([`1c0b7be`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1c0b7be3e0332d9f16d89669e9e815a3213e1072))
    - Refactor contract call methods to redirect invalid transactions to 0xdead ([`b268d37`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b268d3780c370ae03bcb822fc27189438aa17deb))
    - Add inscription_id field to TxED and implement API method to retrieve it ([`987aaea`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/987aaea26382ccb339dd1e5fd317a63cb46efb05))
    - Remove unused test imports, use alloy_primitives instead of revm re-exports ([`0a8d187`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0a8d187c541f8b934a0a09bebba6248f061c41f5))
    - Split db/mod.rs into db.rs / mod.rs, rename server_instance.rs to engine.rs ([`d87ba8c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d87ba8ca04d87c46e069b1056f0b95b5149159af))
    - Use read/write locks for evm database and propagate errors into the RPC layer to wrap and display ([`87739b5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/87739b5b7a5d56a402e75a8c051c62f753ef4539))
    - Simplify serialization and encoding of BytecodeED by using original_bytes ([`4ffc762`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4ffc7623a9f2d50d08818f87e3f41f4686759479))
    - Return BytecodeED from eth_getCode, so it gets serialised without the trailing zeroes ([`b82bfb5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b82bfb5d8bdcb0ebcf27cca405928f4543e56265))
    - Update module visibility across various files ([`4f31079`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4f31079272c6659a18c6f2cc64a4c5f7eb844974))
    - Handle missing contract addresses by using Address::ZERO in Brc20ProgApiServer ([`b9caea9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b9caea9e4aa23b03f7f0238f96431355ee6c670a))
    - Enhance contract metadata handling in BRC20 controller ([`676eb34`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/676eb344e750d592e705995ca2b2a6dcf4c32d1a))
    - Updated BRC20_Controller.sol to deploy Wrapped BRC-20 on new ticker mints. ([`02ac743`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/02ac74349dd4e4584c0130d4b04f1b7816ed9b6c))
    - Upgrade Solidity compiler to version 0.8.28 and update related dependencies ([`af5e1fb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/af5e1fb534492e97607f9a4ae5635f1a6f4bbbd6))
    - Enable optimizer and update output file generation in BRC20_Controller contract ([`575c942`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/575c942972771a36694c8e4dcbd0b34405733e19))
    - Add a js script for BRC20_Controller contract to fix solc version ([`b5ffdac`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b5ffdace5bf2c8e3e01e1a9120b88dc2cc55a32c))
    - Move Id import to tests ([`bb1c3fd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bb1c3fd5e6b1ed50bd93710c3ed60cb922af4cf0))
    - Return correct ID ffrom RpcAuthAllowEth ([`1836eee`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1836eeed4680639dc2e13a285098d198a047ca62))
    - Add unit test for HttpNonBlockingAuth allow_all method ([`592390a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/592390af2345610797f49727878f3311be8ad936))
    - Fix HttpNonBlockingAuth for allow_all(...) ([`747fe03`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/747fe034d3eb1ab48f449e0bc77cb9a854e7f942))
    - Implement HTTP Basic Authentication middleware for RPC server, it allows all methods that start with eth_*, and rejects everything else if the request is unauthenticated ([`e7158f1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e7158f1dc2924b81a251a5a604a1a48b068f0c14))
    - Add basic HTTP authentication support to the brc20_prog RPC server ([`d94fe38`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d94fe38a9f08faf432c611c7d32b4b2fe86849dc))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 30 commits contributed to the release over the course of 3 calendar days.
 - 3 days passed between releases.
 - 26 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Bump version of brc20_prog to 0.1.1 in Cargo files ([`614c74a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/614c74a32ad5014e28ffd9653bd1e7b01c35ff7e))
    - Add data_or_input method to EthCall for improved data handling ([`a9c9937`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a9c9937cf7672dbe785e168c3f66fd79908527b8))
    - Rename variables for clarity in various modules ([`74f4220`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/74f4220a55bd44792b97f046e25001e3eb1fda36))
    - Simplify data decoding in decode_brc20_balance_result function ([`01db2c0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/01db2c03dc4e68febb2b4bda49d35c02243473d8))
    - Ignore transactions that happen in the future in tx details precompile ([`a8f6405`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a8f6405b9b39c3ee072f6a184b412fa8f79152b2))
    - Refactor calls to unwrap() for Result/Options to avoid unnecessary panics ([`78f440e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/78f440e8b620b064b2eb58cf3cbf6f7d003295fe))
    - Update BRC20 locked pkscript precompile result for integration tests ([`5831977`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/58319779251015d3e41de54e60f8120388f49e85))
    - Fixed pkscript pubkey things on get_locked_pkscript_precompile ([`80c2b54`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/80c2b5407928394f518a050280b4234ed3cf98ea))
    - Last_sat_location_precompile location finder now uses do-while ([`a7db66e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a7db66e1432aa84e0eb8e76b9b38770bffaa73cf))
    - Get_brc20_balance now uses u128 which is more than brc20 balance limit (2**64*10**18) ([`98d9a5b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/98d9a5b6191a2599f438657dfde4dadafbd0615a))
    - Remove unused FromHex import in brc20_controller ([`372b8ac`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/372b8ac38a8923fa5f7888f1501bc75d50057683))
    - Change ticker parameter type to Bytes in transaction loading functions and add ticker_as_bytes utility function ([`ca2f1f9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ca2f1f985eedfafa7818636a48a63b431982523a))
    - Rename verify_block_does_not_exist to require_block_does_not_exist ([`c7d6204`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c7d62041ae510be583dca05bf640d5602b5a50c9))
    - Include assigned gas limit in txes ([`e31ed1f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e31ed1f5a98da2b9f727fcd20d1e759ab85cc7f3))
    - Verify new blocks don't already exist in the database, and generate new hashes for new blocks with hashes 0x00..00 ([`c28bbac`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c28bbac6d2e396fd243303057d242042ab759ba0))
    - Add gas_price and syncing methods to Brc20ProgApi (to enhance explorer compatibility) ([`ad1712a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ad1712ad302fcc8ff5563c3f29d77646986f6e65))
    - Change account parameter type to AddressWrapper in get_transaction_count method ([`4afd2a0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4afd2a04d0c26474c1e51f423f35531090b8a096))
    - Include data in returned error instance for eth_call and eth_estimateGas ([`8d2ad19`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8d2ad1903c655bb587765e103956448e69258f93))
    - Add chain_id and tx_type fields to TxED and update related structures, run cargo fmt ([`77a47ae`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/77a47aeec08aff62136efcf62c56b9a93e6cabe8))
    - Change transaction index parameter to be optional ([`eae60ba`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/eae60ba505b1d2dd3babf9f0d8bb84306ec4023a))
    - Prevent deadlock in get_block_by_hash by releasing the mutex after accessing it ([`6a8ced3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6a8ced344cfb268dedd7f3da10a62b7101d87971))
    - Update EthCall struct to make 'from' field optional and use zero in contract calls if from is empty ([`cdc017b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cdc017b16f9af0a5a2ddcbd5865280fd0a303ece))
    - Use LogResponse for eth_getTransactionReceipt for correct eth JSON-RPC format and add v,r,s fields to eth_getTransaction* methods ([`49de694`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/49de6943e0d880c66f500817f651cc8a23572318))
    - Streamline byte, string and address conversions to use into() and parse() for type flexibility and consistency ([`2ce9df6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2ce9df6c937052fcd1fc48a7e7e94d5f04192d30))
    - Simplify encode method to return Vec<u8> directly instead of Result, as it never fails ([`2db1eed`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2db1eed8876cce908913cc8367139f0214e8e8b8))
    - Rename block_hash field to blockHash for consistency in serialization ([`5ca15e2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5ca15e20b0de70cce6a3bd51aee64a3ac8daf7e3))
    - Update Brc20ProgApi call and estimate_gas methods to use EthCall struct and optional input data ([`590563e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/590563efff5fbab51e8f4f3a0a203dc5e8981ce9))
    - Convert contract bytecode using to_string to add the 0x prefix ([`54b6c26`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/54b6c26278ddc1c9e1f11ce56cdecb2ddd9e4618))
    - Rename block_hash field for serialization ([`542c795`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/542c795a660479973068c7e602ea2be3c89213a8))
    - Update README.md ([`e816b69`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e816b691f50b27077cc12cfcf7f48cab4d17ba54))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 167 commits contributed to the release over the course of 413 calendar days.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update README with warnings about BRC20 balance server and BIP322_Verifier contract limitations ([`aae8ab8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/aae8ab89d1ec80974e5aa6d515731a9981550ae0))
    - Made simulate test 5 byte ticker aware, added new log events to brc20 deployer test contract ([`f79cf02`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f79cf0267e02009437c382b2e05dc779ae34c414))
    - Mine_blocks genesis block count ([`19654aa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/19654aa69c0ffa2f310457b8ce19dcf5c54ba98b))
    - Refactor Brc20ProgApi to use EthCall struct for gas estimation and update get_balance method signature ([`02d8762`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/02d8762983983c5a0c0b614a479cc635458963ee))
    - Add hyper and tower-http dependencies; implement CORS middleware for RPC server, add eth_accounts and net_version ([`5b9ba13`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5b9ba130b26780b6789bd20a2fa8093d5214e43e))
    - Add block_timestamp to TxReceiptED and related methods for enhanced transaction data ([`ee0d05d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ee0d05db70a526a015e14689f0e314a11ba3141d))
    - Refactor BIP322 verification precompile to use bytes instead of strings for parameters ([`f24ad6e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f24ad6ebdf1f055dd48142d69c3c8e8c0fe5c065))
    - Add inscription_byte_len parameter to BRC20 methods and update gas limit calculations ([`d79c2cc`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d79c2cc42a1d5013db0d76530bcf2fa632d12fee))
    - Update BRC20 balance check to use non-empty byte arrays for testing ([`cea6af4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cea6af4b73ea2ef9e6a27099b295684980dccfa8))
    - In precompiles, use bytes32 for packing txids, and bytes for packing pkscripts. Update the tests and the docs for them. ([`49dc0a9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/49dc0a93193e5bf33d3d64eef7f65a46b8e2c72c))
    - Reorg to current block height is allowed and no-op ([`b11dcc9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b11dcc9a26f3eb9b8c79635ed41e3c5622e037c6))
    - Remove gas usage for get_block_height, add ignored is_full parameter for eth compatibility ([`1701576`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/170157671f045ea6eadd8ae46942982410e842c6))
    - Refactor performance test to include contract_inscription_id in calls and improve error handling ([`89a8a24`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/89a8a2411745f781418546481fa7eab3fa9ca79c))
    - Split brc20_addTxToBlock into brc20_deploy, and brc20_call RPC methods for easier contract address and inscription id handling ([`2bc074c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2bc074c2e67bf1de42a79ee59617dfffc6b4d840))
    - Changes to make simulate_brc20_events.py running ([`eccb624`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/eccb624d8e438fcdc0467810873965537345f161))
    - Remove BRC20 balance server status logging and add status check before server initialisation ([`ff94bc2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ff94bc2f7c0578becf18ea069e8a52f7c1b0587c))
    - Update default Bitcoin RPC URL to use port 38332 ([`c21e7c3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c21e7c3b0c646431a23f508b09a440dac8e80781))
    - Update default Bitcoin network to signet in btc_utils ([`28ae3f2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/28ae3f2558b662fad39350fc7d47df43e54a5ebd))
    - Add version method to Brc20ProgApi to return current package version ([`09f0cef`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/09f0cefb238ad3a0629d14f3dd0849d0e492a31b))
    - Update README.md to clarify environment variable requirements for precompiled contracts and add clang installation instructions ([`50003d7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/50003d71df07ba6c627a99a15e3b5276de9c9451))
    - Run cargo fmt ([`34c4353`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/34c43534d5b02d2b30bda08490e8709dc9008069))
    - Add logging for get_logs, call, and estimate_gas methods in RpcServer ([`fa356de`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fa356deb3ea4bf6095181139d8a69675da5c4805))
    - Refactor GetLogsFilter to use B256Wrapper for topics field ([`66133c2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/66133c29a8140373d701380aded6e67ef1024237))
    - Refactor GetLogsFilter to use AddressWrapper for address field ([`4293b3f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4293b3f6d676fba789c8fa0d085ae42e111c0899))
    - Refactor JSON-RPC API to use wrapper types to consolidate sanitisation/deserialisation ([`3ab0cf1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3ab0cf1580e2b69f41d9a6526716d51b10578ec0))
    - Update expected hex output in BTC precompile tests ([`5a0eeae`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5a0eeae63e492b36f8f42b48b382d900bb00fc1d))
    - Make precompile functions use a shared HTTP client for improved reliability ([`541cf24`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/541cf24e5358fe47c13c6021749d903b75624787))
    - Refactor performance test to load transaction data dynamically and update precompile test selection ([`fbdf432`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fbdf43297b89c330dc30fd199c8dfb860a5701c8))
    - Refactor BRC20 precompile functions to use alloy_sol_types ([`7acd841`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7acd8410ea1b68792bef202dd6424f6532182489))
    - Add input prompts to pause on failed precompile tests ([`6fdfcad`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6fdfcad94bc92c3d3597579ce51c95bef4513910))
    - Update TxReceiptED to serialize result_bytes as an empty string when None ([`030b67b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/030b67b3c60a2493bce5eead4d4bb822defbbf15))
    - Avoid overflow in gas usage in case of errors ([`774f532`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/774f53215247f59fa340a4ead53d23d448c71386))
    - Use tx.json files in Precompile Test ([`c419e17`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c419e172b9dbeb5e7c468cf55cd5e7b9e95338b8))
    - Add support for inscription IDs in BRC20 transactions and receipts ([`a9a4751`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a9a4751a4e21aa3c8a66c23a5761509ac9c5fc1b))
    - Check results of precompiles in precompile test ([`91722fa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/91722fae8a19c3ecf977758a5d4148137e9dcd26))
    - Check Bitcoin RPC and BRC20 Server status at startup ([`e7e0e9e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e7e0e9e570bf0fb2f4fde44111a4633369f63197))
    - Ignore u64::MAX for gas usage, and rename BRC20 Helper contract functions to match precompile signatures ([`434eb09`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/434eb09289f3e2c8f589e2c42b555ef89c8214d2))
    - Add better error handling and sanitisation for BTC precompiles ([`0aed901`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0aed901fee214d880c23c8907fd35cd8b344b2a0))
    - Move btc precompile tests to signet ([`aa72231`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/aa72231e8d0eca523cb7eef5a27c4f612b07eb54))
    - Fix bytecode comparison and encoding to handle padding bytes correctly ([`9b12a15`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9b12a15ca9a13dc0db7268045dd7f828e638431d))
    - Change order in precompile test ([`fda28cf`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fda28cfc7057d00b9195ce9479e66c026f794173))
    - Refactor BRC20 precompile to use ureq for HTTP requests and add another test that runs all precompiles ([`423b154`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/423b1546331bd3e06c0ffe00ed5b1f02341e661a))
    - Refactor precompile interpreter result handling ([`463b58e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/463b58e4234338581b26ff5a258b51901f6727e9))
    - Upgrade to revm 20.0.0, add more tests for precompiles ([`8907cd7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8907cd7fd5baefdbbd3681fde24c0856402c7542))
    - Comment out direct precompile call ([`edeb2a9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/edeb2a9ba7d40e591758127738e4db69d7c95d6f))
    - Implement direct static calls for BRC20_Prog contract ([`9cdc74d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9cdc74dedf009af81a4a26e5e45b0bdf9096f2d4))
    - Add transaction failure handling and clean up gas usage output in performance tests ([`b9f05ea`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b9f05ea79a7c633f1ad98258d96b61b494b27af1))
    - Add a performance test for BTCTxDetails precompile ([`e056b5b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e056b5b353076656547cc352c39402e65723ad61))
    - Make BTCTxDetailsPrecompile retrieve block height via another RPC in case it's not indexed ([`d90aae5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d90aae58144faccf8e7a5092e7044999c8d15041))
    - Update sample environment variables for Bitcoin RPC URL and network ([`5fafce8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5fafce806e3dd30bd4065917c09c5757611cee0b))
    - Use an environment variable for the RPC server address ([`8b489ec`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8b489ecf2b3e6582096dbf9645d4e35272f25253))
    - Update README to specify JSON-RPC server address for BRC2.0 ([`ff7cfaa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ff7cfaaab0dbf4366f165458b049ec562e2bfa31))
    - Add License and Notice ([`cf76cbf`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/cf76cbf5a1ca9a1a79130fa83f2d89d34792682c))
    - Enhance README with detailed return descriptions for BRC20 JSON-RPC methods ([`3bf6fb2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3bf6fb240c1e49ef7e9a3905543eff3efb39ff51))
    - Format code blocks in README for better readability ([`9a25091`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9a25091ca9806910a91fb594d710863714044f46))
    - Update README to reflect BRC2.0 branding and enhance module description ([`2875950`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2875950cd35114e316a4dfba5e016938b06846b4))
    - Update README to include Rust requirement ([`608c348`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/608c348e4d4a043ea7c6a8d49a4fecb1417a3eaf))
    - Add indexer integration guide to README ([`f0de05e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f0de05eab4be5bde6edcab4edb84ea3b0b38c4a7))
    - Update README to clarify HTTP call for BRC20 balance retrieval ([`e8ec1af`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e8ec1afe042866dabecdf9b058cafca6a8fd0788))
    - Clarify README description of JSON-RPC methods functionality ([`c15cba8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c15cba850edc6c20c5e22d8c11580714bf07dcf8))
    - Update README ([`9e3eaae`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9e3eaae17015dfe5d3d8feac766c20a426c1b46c))
    - Add functions to get last satoshi location and locked pkscript in example helper contract ([`71f513a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/71f513aba9967f0dddee103af58cdc50626465e6))
    - Add github actions and skip BTC tests if the environment variables are not set ([`bfb6cde`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bfb6cde991bd2a06e460f05c2273b01ea2cf2f7b))
    - Add empty README and remove unnecessary files ([`2a85899`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2a858990fa69a15d45b688bd752e0c010f938130))
    - Run cargo fmt ([`e088a10`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e088a1050842e5cd3a034e0d58508e3d0ca7c57d))
    - Add transfer tx for sample erc20 contract ([`1f31eed`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1f31eed331e1e3568a1ac37ef8f52d1fdbb01085))
    - Update import path for ERC20 contract in BobCoin.sol ([`7dfc45d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/7dfc45d0afd0af9cdc488f2a71f887d4e9860758))
    - Add an ERC20 contract example ([`3adf9dd`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3adf9ddcd3881d591ae6b6f5e9464e252cfdd4a7))
    - Show float numbers in test_generator.py ([`be00871`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/be008714bda08dc2be67f5485d678a0ed1ab2fde))
    - Rename block count variables for clarity ([`f395953`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f395953ca03507ca9f37fc77a6068dc11a885a9e))
    - Make get_range end key comparison exclusive ([`b302e55`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b302e55b65807960e28c760eafd8d64a5f193073))
    - Add genesis height parameter to BRC20 initialisation methods and update related logic ([`ef6e413`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ef6e4136d2bb5a0ea011aeef461d2cd451658d77))
    - Refactor module structure by reorganizing EVM and BRC20 controller modules, removing obsolete files ([`6e02cce`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6e02cceede8abc089cd826fbc57914720b9df976))
    - Improve error message for call_contract ([`85b6888`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/85b68884a1073c55f63079d05ef845b5d87407b4))
    - Run cargo fmt ([`5315986`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5315986e228d31ad92ae59c0cd55e524a3518517))
    - Set max lock block count to 65535 in GetLockedPkScript precompile and implement tests for edge cases ([`2c0ee9c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2c0ee9c5a1d38874542c0df543a5d49fb786d01f))
    - Add environment variable checks for Bitcoin RPC in BTC transaction details precompile tests ([`8cf0039`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8cf0039cd21b0e5b91e7b4500aafe5de55c96fbd))
    - Add error handling for invalid lock block count in GetLockedPkScript precompile ([`9522de3`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9522de33bf377a96ca79c8b4a4d6ebabfa7f4741))
    - GetLockedPkScript now uses environment variables to change network ([`d4ac795`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d4ac795f715cf8f79e5872e2553769fdad9a8624))
    - Implement GetLockedPkScript precompile with locking script logic ([`16f3d1c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/16f3d1c994d76c5c769e5a2500e4bf446228aeef))
    - Refactor precompiles: rename and reorganize modules for clarity and add empty GetLockedPkScript precompile ([`6fcbe4e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6fcbe4eb2239f5618df4645c46c2fef0755fbc87))
    - Add error handling for missing block height in BTC precompile ([`a31a3ac`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a31a3acc464110f6ea90fc15f60dd08da9a15c77))
    - Revert "Return empty values instead of an error for coinbase transactions" ([`6390284`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6390284ac0a552c86468ffb15235329fa56e3786))
    - Return empty values instead of an error for coinbase transactions ([`85d940b`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/85d940baabb3e862a4e9bdfd82338556a0119ca5))
    - Add support for error handling of coinbase transactions in LastSatLocationPrecompile and update related tests ([`10878c6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/10878c6a19944ff3290596ef4fcd4aec40edddf6))
    - Remove unnecessary usize casts ([`d5b79d4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d5b79d42afa545540a80b02cf440d456447ed81f))
    - Implement Last Sat Location precompile, and reorganise the precompiles module ([`f167236`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f167236f086383d928c1f495b87027dfd5489f3a))
    - Add multiple test cases for btc precompile that also checks gas usage ([`90df89d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/90df89dc682fabf5bbe9da00dd0a3c5f70affbd9))
    - Return all vin/vouts as arrays in BTC precompile ([`f479ca7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f479ca774bb576b20e112085a9160af85645066d))
    - Add commit and clear_cache methods for db_block_number_to_block ([`c06ecd2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c06ecd25a4f7be2fba2c08f33c718bbcb52ed4c9))
    - Remove unused requirements file from test_generator ([`fd83894`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fd83894c3c0ccdebd16d0830d51a96e54349f666))
    - Run cargo fmt ([`2e18173`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2e1817333a9a06cd18b64594b2ef9ae3fb17b21b))
    - Format some python code ([`0bc81d7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0bc81d7e63cc57e949f42cfabab5a407330b3ccb))
    - Refactor some integration tests to match current spec, and update brc20_deposit and brc20_withdraw methods to return TxReceipt ([`0ee0bbb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0ee0bbba23d08c0e1381e66e1366d60b4f9f0fc3))
    - Change log level from DEBUG to INFO for deposit, withdrawal, and balance checks ([`6c08b9d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6c08b9d4f4087c0891293a0a94d2edb2fd630396))
    - Improve error handling in BRC20 balance decoding function to return zero on failure ([`66a8552`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/66a8552d88a0b269063aafde474e41ab52ddde7f))
    - Refactor Makefiles to streamline ABI file formatting ([`182c42e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/182c42e3bca8ad1338c94cfecb522649e7f48320))
    - Add BRC20 program helper Makefile and update contract interfaces; enhance error handling in precompile ([`8964dd9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8964dd9ea738e25473e85a0902ff4059d491da15))
    - Implement BTC precompile ([`8346b66`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8346b6615e0c1b5fb0002d74fdaf0dd5a5d5abad))
    - Update expected result in BRC20 balance precompile integration test (according to the hashing balance server in OPI brc20 indexer) ([`0a61e11`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0a61e114d244cca7c9f9e122d6a482078260bbc7))
    - Add BRC20 balance precompile and integration test; update dependencies ([`5213b28`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5213b28853f2c412d148cac73a1da2442f4ce8c8))
    - Fix integration tests and enhance RPC server logging; store block responses in database for faster retrieval ([`61deffa`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/61deffaf0ced3cdc11378c3e10e65ede561bdd0e))
    - Implement trace logging in brc20 RpcServer methods ([`f7c2030`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f7c20300f05facefd6e444d3ec435ff7465b9dcc))
    - Fix function selector for balanceOf in BRC20 precompile ([`18a6d5f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/18a6d5f7d98a3435b5252d13e981a1b983dec9ab))
    - Add brc20_initialise method for genesis block ([`1509045`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1509045e61ed84dd2d2b7728814e5bae0feded5d))
    - Move BRC20_Prog contract and interfaces to examples/brc20_prog_helper directory ([`9a050b8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9a050b81e57ff8601ddfc794a0a121792c57bfa5))
    - Add a Simple solidity contract and a compiler script that outputs deployment and call inscriptions ([`57f537d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/57f537d11449fe166f07948f8e783b81ee84db2b))
    - Add BIP322 precompile and an integration test for it ([`f8cb526`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f8cb5268515a2459af2afb0d3722c9eab7923900))
    - Format BRC20_Prog contract ([`129d39d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/129d39d8138575f850187e027d27d70beea6152a))
    - Add BRC20, BTC, and BIP322 precompiles with initial implementations and interfaces ([`e436ccb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e436ccb99ae546eed505f2561d9078452f5b2234))
    - Fix hex formatting in LogED serialization to remove '0x' prefix ([`a1bca25`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/a1bca256932f6f0d75ce0b245a9581edf2ae2e6e))
    - Fix u128 order in mint and burn transactions; remove unused deposit and withdrawal endpoints ([`9f1a07c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9f1a07c8c7093a492e51963af31b84322c32f2a2))
    - Add BRC20 balance retrieval ([`6a8671f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6a8671f7dad6374230bd84a87224d82c5cb57471))
    - Implement BRC20 controller with mint and burn transaction handling ([`4348057`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/434805737775840ac8cff6ae625624a5475428ae))
    - Add empty deposit & withdrawal methods in ServerInstance for future use ([`807dade`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/807dade306a6538fbd025b74148f4ce2ada423f0))
    - Handle optional values in transaction and block retrieval; provide defaults to prevent panics ([`0b9738a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0b9738a237403af2cc41e32e69128638f8f368ad))
    - Update gas limit and nonce calculation in block response; improve contract bytecode retrieval handling ([`e86ac23`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e86ac2351716e5ae6d510f7dbdc4d4c637dfce3c))
    - Handle missing parent hash by providing a default value of zeroed bytes ([`4d0fe94`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4d0fe94a10030667d1bc8cfbc228357b6bc654fe))
    - Update block parent hash handling and improve block hash generation logic (so every block has a unique non-zero hash) ([`718cde9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/718cde9f58ec22115e37a6a2fb02dcdae98e86a0))
    - Implement Merkle tree for transaction roots and add block #0 ([`99f0e73`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/99f0e735695782cf6a6f712dac9c0f743ebf699c))
    - Update miner field type to AddressED and change nonce serialization to StrictPfx ([`8653ec0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8653ec01da4550a4bb2fb80a16a69cb361c4ee53))
    - Add contract compilation script and update package.json scripts ([`b670e5a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b670e5a7e7a94f2a1fc009c7e81909a1875bb386))
    - Fix a comment in run_custom.js ([`e12a71f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/e12a71fd38336bf69cdbfffea981730f88d99e08))
    - Update contract assets folder path ([`5498d7d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/5498d7d76ba0492927451cdf18cf5c0ef2b6545c))
    - Refactor contract loading to use JSON format ([`daa20f6`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/daa20f630396c0bc73fa20a9ebabfd2d59dd461d))
    - Deploy BRC20_Controller as part of server initialisation ([`d0a3b23`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/d0a3b23ca5382e9d13663e847fb39149f63da003))
    - Add gas limit parameter to get_evm and update eth_call to enforce gas limit ([`554071d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/554071dd330a685740d273ca54b2d0f5dce15cf6))
    - Refactor transaction receipt handling to include result type and reason in TxReceiptED ([`09f9649`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/09f9649dc92470c29c19b0bbd9cc97f2faea1d50))
    - Update LogED encoding test to reflect new byte length ([`1a3e69f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/1a3e69ffeb2047a28e21f8b878e72ecddb374ea7))
    - Update LogED structure to include log index ([`bc9fb00`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/bc9fb00cad753332e0736c1ff0d5938d375385aa))
    - Add BlockResponseED type and update getBlockBy* RPC methods ([`79838da`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/79838da6f9b4c9af8240e344ba592dc2e3b4d368))
    - Add transaction count RPCs ([`ddec120`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ddec120feef2ca4014d30f44dd2cf243ca93e1c7))
    - Add eth_getLogs endpoint and LogResponseED type for log retrieval ([`45a738d`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/45a738dc6e163e6af89ec80268c255e12595b8ff))
    - Rename brc20 indexer RPC method prefixes to brc20_ ([`819cecf`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/819cecf5dee09d7278a4da61c83231cd78d50560))
    - Implement eth_getTransactionBy* and eth_getTransactionReceipt endpoints ([`0d531a7`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/0d531a7177104b53b986cd06c7d525323092a5f2))
    - Flush databases after commit ([`051fc4c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/051fc4c68d41ce2fa8b19f90fea4abf2e3cb9ea3))
    - Update Databases to use RocksDB ([`4b741cb`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4b741cbfe07d8f90203ee5697954517be5422455))
    - Remove old caches from memory after deletion ([`3274a18`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/3274a18c1cce50bf4a3d9f5e9c3f25cab5174f9c))
    - Rename EnvWrapper to TestEnvWrapper for clarity ([`2d1f5a0`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/2d1f5a0c33df14a60bc730e2bb299a08494bd3f4))
    - Refactor test environment creation and deletion ([`505ca3e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/505ca3ef2df37c7fb308c51797902e5f4823b8ae))
    - Add default-members to workspace in Cargo.toml ([`38a4b19`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/38a4b19164f5b78c99fd718eb4123fd95f304cd9))
    - Rename package from brc20_finance to brc20_prog ([`3525638`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/35256387113a11dfc61a2d518b6e6f3a00ec6ae5))
    - Remove unused dependencies ([`4b82964`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/4b82964d01e6ce6d6d7a0f362998288281a89139))
    - Refactor BlockDatabase to use BytesWrapper for database keys ([`f922143`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f9221438efed8e617f1275507144a51adab93056))
    - Add new JSON-RPC methods for gas estimation and storage retrieval, update existing method signatures ([`53c41f5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/53c41f50dcf4ad0d2433e0fef956884970ec0c72))
    - Don't store unchanged values in the history caches to minimise disk usage ([`c99ab02`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/c99ab0289160979cf2279d32bcb65429716817a4))
    - Refactor some RPC methods to use hexadecimal format and update related tests, add some empty/static eth_ endpoints ([`6b73e18`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/6b73e184ecd234f209fd5995875c1a0b024b9d6a))
    - Add Brc20ProgApi trait with JSON-RPC methods and remove HTTP server ([`ed164b4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ed164b45235f1227a73ddf010dcaacc8de1dbcfa))
    - Fix bug in run_custom.js to accept null responses ([`61fedae`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/61fedaee0709344ce80be1ed0ea48436b9beae9b))
    - Add JSON-RPC server support and refactor main server and tests ([`10a245f`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/10a245fd4f7ff18e7c65a45b71b7893d9398d075))
    - Add RPC server implementation and update finalise_block method to handle optional start time ([`ab53f06`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ab53f069428f9ded7fb04e4d4fd9562537d5ba3c))
    - Add logging for transaction addition to the next block ([`77c2de2`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/77c2de25ea3649d9ef2ee31a12b8e84e5d6d3a5a))
    - Extract methods into ServerInstance object to prepare for RPC server setup ([`77b3be5`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/77b3be51db0c90a4da155a8577b6d7c170d6c807))
    - Refactor database access in transaction handling functions ([`b8832ac`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/b8832aca300de1063deb0bdff748c57937a11e9e))
    - Refactor EVM module structure and update dependencies ([`f36e4c8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f36e4c882859ba7b52610f71a6718bd3c5694a39))
    - Remove .DS_Store files ([`052ac8e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/052ac8e991c9a51b91c813834c85fa8086608135))
    - Migrate db crate to use block history cache ([`54edf94`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/54edf946e53b6344f8bd1ae61eb7cadad0a7fd16))
    - Implement block history cache [WIP] ([`73fdee1`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/73fdee16545611a6cb02d4dfa3a760acf696dc9a))
    - Fix decode for db/src/types/address_ed.rs ([`ded82a4`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ded82a405c7868ab301fbfc9c7d437d8e7f4c443))
    - Add tests for types with traits BytesEncode/BytesDecode ([`f47175a`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/f47175a54338ede69ed90b10fd54b2d1a2df7a22))
    - Split database types and format files ([`ba4ef20`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/ba4ef20b124eefc4e177fc9b3bf2df3c93cd7bad))
    - Set up to work with localhost and add requirements.txt and npm files ([`92a6f83`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/92a6f8381efe282597a192a7537e5171f8f6dd94))
    - Added package.json ([`fa32ab9`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/fa32ab989d62a51fd998eccae9f3fc7e4678ddd8))
    - Added integration_test ([`9f669e8`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9f669e8d20bc15fc8955627a94dafcd63d051257))
    - Removed Mutex'es ([`9db0c5e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/9db0c5efa8423bc490ab63ae6996d6fb0676a73c))
    - Added cache with mutexes ([`504828c`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/504828cd4f89c0f586ff4e6ca2eaa341402ee7a9))
    - Initial commit, no reorg protection, only code memory cache ([`8a3494e`](https://github.com/bestinslot-xyz/brc20-programmable-module/commit/8a3494ec688e84374f9d6217072d244def32e773))
</details>

