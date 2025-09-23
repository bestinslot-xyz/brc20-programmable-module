## [unreleased]

### 🐛 Bug Fixes

- Update pre-release hook to correctly generate and add CHANGELOG.md
## [0.11.6] - 2025-09-23

### ⚙️ Miscellaneous Tasks

- Add release configuration file for versioning and changelog generation
## [0.11.5] - 2025-09-23

### 🚀 Features

- Enhance error handling for eth_call and eth_estimateGas with status codes and detailed messages in RPC server

### 🐛 Bug Fixes

- Update read method signature to use lifetime annotation

### 🚜 Refactor

- Generate receipts for invalid transactions

### ⚙️ Miscellaneous Tasks

- Release brc20-prog version 0.11.5
## [0.11.4] - 2025-09-20

### 🚀 Features

- Refactor transaction handling by removing unnecessary fields, and simplify the code
- Add a global values database to Brc20ProgDatabase and disallow multiple reorgs by storing a global maximum reorg height
- Implement web3_ api in Brc20ProgApi

### 🐛 Bug Fixes

- Couple of bug fixes for reorg handling on database
- Replace "DB Error" with a static error message and panic instead of returning an error, as it's not recoverable
- Update BITCOIN_RPC_URL to use address
- Ensure latest value retrieval from the cache is never empty, and reorgs also panic if there's an attempt to empty the cache
- Use a fork of bip322 for a patch where it crashes
- Bump version of brc20-prog to 0.10.4
- Fix test_reorg_all_blocks to reorg only 10 blocks
- Update inconsistent docs
- Enhance bytecode handling with error checks during deserialization and decoding
- Simplify reorg logic in BlockHistoryCache and UintED serialization/deserialization
- Migrate back the bip322 crate after the fix and remove the fork
- Refactor Brc20ProgDatabase to remove redundant timestamp and gas methods, they are stored in the block database already
- Improve error handling in contract address resolution
- Correct version string format in web3_client_version method
- Update public_api snapshot
- Bump version to 0.11.2
- Update keccak256 import from revm to alloy
- Bump version to 0.11.3
- Add step to build with no default features in CI workflow
- Remove flag from RawBytes#value() function
- Bump version to 0.11.4

### 🚜 Refactor

- Simplify balance retrieval by removing error handling from get_brc20_balance function, it now panics when balance server is unreachable or provides invalid data instead of silencing it
- Update public api
- Use catch_unwind for BRC20 balance server checks to handle potential panics gracefully
- Enhance ConfigDatabase by adding caching and implementing flush functionality
- Add input size limit warning for bip322_verify_precompile to prevent excessive resource usage
- Add legacy fields back in BlockResponseED and TxReceiptED for backwards compatibility
- Modify eth_call in RpcServer to return 0x or the revert reason for compliance
- Update eth_getCode to return 0x if there's no code present for compliance
- Bump version to 0.11.1 and update binary name in Cargo.toml
- Print the error message in get_brc20_balance function when it fails
- Enhance RPC server configuration with request/response size limits and batch request settings
- Rename DB_PATH to BRC20_PROG_DB_PATH for consistency in env.sample
- Improve error handling in Bitcoin RPC status check
- Ditto, improve error message for Bitcoin RPC status check failure
- Remove batch request limit from RPC server configuration
- Add configurable request/response size limits and batch request settings to RPC server
- Update public api and fix a visibility error

### 📚 Documentation

- Add caution note about basic auth and secure connections in README
- Add warning about ticker case sensitivity and format in README
- Simplify cargo run command in README

### ⚙️ Miscellaneous Tasks

- Cargo fmt
## [0.10.2] - 2025-07-31

### 🚀 Features

- Add brc20_transact method for sending raw signed transactions and update parameter names

### 🐛 Bug Fixes

- Update Discord (badge was broken) and add Telegram link to README
- Bump version to 0.10.1 and ensure raw_block_ed module is conditionally compiled
- Clarify initialization process for BRC20_Controller and empty block handling in README
- Refactor Bitcoin RPC validation logic and remove unnecessary checks
- Bump version to 0.10.2 for brc20-prog

### 📚 Documentation

- Add support for raw signed transaction inscriptions and update relevant methods in README
## [0.10.0] - 2025-07-15

### 🚀 Features

- Add debug_getRaw* methods in the API and update version to 0.10.0

### 🐛 Bug Fixes

- Correct comment formatting for RawBytes struct documentation
- Prefix hex-encoded strings with "0x" in RawBlock methods

### ⚙️ Miscellaneous Tasks

- Cargo fmt
## [0.9.0] - 2025-07-04

### 🚀 Features

- Add basic HTTP authentication support to the brc20_prog RPC server
- Implement HTTP Basic Authentication middleware for RPC server, it allows all methods that start with eth_*, and rejects everything else if the request is unauthenticated
- Add a js script for BRC20_Controller contract to fix solc version
- Enable optimizer and update output file generation in BRC20_Controller contract
- Upgrade Solidity compiler to version 0.8.28 and update related dependencies
- Add inscription_id field to TxED and implement API method to retrieve it
- Refactor contract call methods to redirect invalid transactions to 0xdead
- Add an environment variable to toggle trace recording
- Add EVM_RECORD_TRACES to env sample
- Validate bitcoin network using bitcoin rpc
- Add dotenv support and update dependencies
- Update environment variable name for BITCOIN_RPC_NETWORK, and use a denylist for RPC method auth
- Add PROTOCOL_VERSION to configuration and database validation
- Handle nada encoding in inscription data (not activated until compression activation height is set)
- Add functionality to retrieve inscription ID by contract address, bump db version to 2
- Increase block gas limit to 4MB (block size) * 12K (gas per byte)
- Add zstd compression support with 1MB buffer size, reorganise some internal variables
- Implement some precompile tests in rust
- Reduce GitHub Actions stack size to 2MB, move long responses in precompile tests into assets
- Add EVM call gas limit configuration
- Implement TryFrom trait for AddressED and FixedBytesED, add is_zero method to UintED
- Implement TryFrom trait for AddressED and FixedBytesED, add is_zero method to UintED
- Gate server-side features under "server" feature, this reduces client compilation time
- Add brc20_transact endpoint to handle signed evm transactions, currently disabled via config
- Support using RawBytes and Base64Bytes types together in BRC2.0 API
- Enhance BRC2.0 with pending transaction management
- Store and serve v,r,s fields from raw signed transactions, bump version to 0.9.0
- Add "brc20_transact" method to indexer methods list

### 🐛 Bug Fixes

- Fix HttpNonBlockingAuth for allow_all(...)
- Return correct ID ffrom RpcAuthAllowEth
- Move Id import to tests
- Handle missing contract addresses by using Address::ZERO in Brc20ProgApiServer
- Return BytecodeED from eth_getCode, so it gets serialised without the trailing zeroes
- Simplify serialization and encoding of BytecodeED by using original_bytes
- Make authorisation logic exclusive to brc20_ methods, and fix TraceED encoding
- Serialize block nonce to full hex length
- Simplify assertions in tests for AccountInfoED conversions
- Use dotenvy instead of unmainained dotenv, avoid embedding the source for brc20_controller contract to reduce binary size
- Remove unused rust-embed feature "include-exclude"
- Better eth_getLogs topic and address handling
- Return an error if block range is too large for eth_getLogs
- Remove unused import of E from std::f32::consts
- Update result field name from resultBytes to output in transaction receipt
- Update alloy-sol-types dependency version to 1.1.0
- Readjust stack size to 4MB in GitHub Actions workflow (again)
- Check if the transaction is in the future in last sat location precompile
- Reduce gas usage from 100k to 20k for bip322 precompile
- Update LastBlockInfo to track total processing time
- Rename db.rs > brc20_prog_database.rs
- Remove optional flag from nada and zstd-safe dependencies, as clients can use these to generate encoded data
- Remove conditional flags for some imports
- Ignore raw tx tests as they fail to recover the correct address at the moment
- Update ecrecover call to use adjusted v value and remove ignore from tests
- Update DB_VERSION to 5 in configuration
- Correctly increment transaction index in pending transaction execution
- Handle extra padding in base64 decoding for inscription data

### 🚜 Refactor

- Update module visibility across various files
- Use read/write locks for evm database and propagate errors into the RPC layer to wrap and display
- Split db/mod.rs into db.rs / mod.rs, rename server_instance.rs to engine.rs
- Remove unused test imports, use alloy_primitives instead of revm re-exports
- Make tests ignore the result of .initialise(...)
- Simplify EVM setup
- Simplify revm integration
- Update BRC20Precompiles initialization to use a constructor
- Move dotenv call to Brc20ProgConfig
- Remove minimum gas limit constant and handle inscription byte length in tests
- Handle JSON-RPC error code -5 without panicking
- Replace Wrapper types with ED types in API and RPC server
- Reorganise engine into its own package alongside the evm, and public api types into the API package, add a public_api test to check for changes in the API
- Refactor Brc20ProgApiClient methods to include 'brc20_' and 'eth_' prefixes and update related client calls
- Remove redundant names from instrument attributes in Brc20ProgApiServer implementation
- Refactor to use a "Transaction" instance instead of "GetRawTransactionResult" in BTC precompiles
- Update BTC precompiles to use faster transaction and block height retrieval methods
- Add more test cases for bitcoin mainnet rpc tests and run bitcoin rpc tests sequentially
- Update benchmark functions by removing unused transaction detail cases, add min stack requirement to readme
- Update decompression buffer to use dynamic Vec instead of fixed array
- Update BRC20 types to replace EncodedBytes with InscriptionBytes and EthBytes for improved clarity and functionality
- Move gas usage and history size constants to the global package, adjust gas usage for precompiles sending an RPC
- Update database version from 3 to 4 in configuration, as previous gas values need adjustment
- Disable logging for common block retrieval calls to reduce noise
- Move conditional compilation for Brc20ProgDatabase to a separate line
- Fix cargo doc
- Update raw tx handling to use alloy TxLegacy type
- Replace CHAIN_ID with Brc20ProgConfig.chain_id in various modules to enable using different chain IDs for signet and mainnet
- Remove brc20 event simulator test

### 📚 Documentation

- Update module documentation for clarity and completeness
- Update README to include note about BRC20 indexer integration and enhance operation checks for deploy/call inscriptions
- Remove warning about zstd compression stack size requirement from README, as it now uses Vec instead of a fixed size slice

### 🧪 Testing

- Add unit test for HttpNonBlockingAuth allow_all method
- Add unit tests for BRC20ProgEngine and RpcServer methods
- Add serialization test for BlockResponseED
- Update test cases to use a non-zero value for inscription byte length
- Add BRC20 balance test and a deploy/call test
- Move test utilities to their own package, add a benchmark for calling the bip322_verify precompile
- Migrate precompile benchmarks to Rust
- Add microsecond and second tests for gas processing times in opcode generator test
- Add mainnet precompile tests (works with a .env setup)
- Split env setup in tests into .env.mainnet and .env.signet
- Split benchmark for BTC precompiles
- Change out of order test to send 3 transactions

### ⚙️ Miscellaneous Tasks

- Bump version of brc20_prog to 0.2.0
- Update dependencies and update RPC authorization middleware to latest jsonrpsee library
- Run cargo fmt
- Move super:: imports to crates
- Move documentation for Brc20ProgApi to top-level module
- Update package metadata in Cargo.toml
- Bump version to 0.6.2
- Run cargo fmt
## [0.1.1] - 2025-04-03

### 🚀 Features

- Add gas_price and syncing methods to Brc20ProgApi (to enhance explorer compatibility)
- Verify new blocks don't already exist in the database, and generate new hashes for new blocks with hashes 0x00..00
- Include assigned gas limit in txes
- Change ticker parameter type to Bytes in transaction loading functions and add ticker_as_bytes utility function
- Ignore transactions that happen in the future in tx details precompile
- Add data_or_input method to EthCall for improved data handling

### 🐛 Bug Fixes

- Rename block_hash field for serialization
- Convert contract bytecode using to_string to add the 0x prefix
- Rename block_hash field to blockHash for consistency in serialization
- Prevent deadlock in get_block_by_hash by releasing the mutex after accessing it
- Include data in returned error instance for eth_call and eth_estimateGas
- Update BRC20 locked pkscript precompile result for integration tests

### 🚜 Refactor

- Update Brc20ProgApi call and estimate_gas methods to use EthCall struct and optional input data
- Simplify encode method to return Vec<u8> directly instead of Result, as it never fails
- Streamline byte, string and address conversions to use into() and parse() for type flexibility and consistency
- Use LogResponse for eth_getTransactionReceipt for correct eth JSON-RPC format and add v,r,s fields to eth_getTransaction* methods
- Update EthCall struct to make 'from' field optional and use zero in contract calls if from is empty
- Change transaction index parameter to be optional
- Add chain_id and tx_type fields to TxED and update related structures, run cargo fmt
- Change account parameter type to AddressWrapper in get_transaction_count method
- Rename verify_block_does_not_exist to require_block_does_not_exist
- Remove unused FromHex import in brc20_controller
- Refactor calls to unwrap() for Result/Options to avoid unnecessary panics
- Simplify data decoding in decode_brc20_balance_result function
- Rename variables for clarity in various modules

### ⚙️ Miscellaneous Tasks

- Bump version of brc20_prog to 0.1.1 in Cargo files
## [0.1.0] - 2025-03-31

### 🚀 Features

- Add an ERC20 contract example
- Add transfer tx for sample erc20 contract

### 🐛 Bug Fixes

- Make get_range end key comparison exclusive
- Show float numbers in test_generator.py
- Update import path for ERC20 contract in BobCoin.sol
- Mine_blocks genesis block count

### 🚜 Refactor

- Rename block count variables for clarity

### 📚 Documentation

- Update README with warnings about BRC20 balance server and BIP322_Verifier contract limitations
