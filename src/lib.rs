#![warn(missing_docs)]
//! This crate provides a BRC20 programmable module implementation.
//!
//! It has a JSON-RPC server that runs the BRC20 programmable module, and a client
//! for interacting with the server. The server is built using the `jsonrpsee` crate.
//!
//! Types used in the BRC20 programmable module are defined in the `types` module.
//!
//! The server is started using the `start` function, and it allows configuring the server
//! with a custom configuration. The server listens for incoming JSON-RPC requests and
//! responds to them. The server supports the Ethereum JSON-RPC API, which allows users
//! to interact with the BRC20 programmable module using standard Ethereum JSON-RPC methods.
//!
//! The client includes methods for deploying contracts, calling contracts, depositing and
//! withdrawing tokens, it also includes methods for interacting with the underlying EVM, such as
//! getting block information, transaction information, and logs.
//!
//! Refer to [README](https://github.com/bestinslot-xyz/brc20-programmable-module/blob/main/README.md)
//! for more details on JSON-RPC methods and their usage.
//!
//! # Example
//!
//! ```
//! use std::error::Error;
//!
//! use brc20_prog::Brc20ProgApiClient;
//! use jsonrpsee::http_client::HttpClientBuilder;
//!
//! async fn print_block_number() -> Result<(), Box<dyn Error>> {
//!     let client = HttpClientBuilder::default().build("https://url:port")?;
//!     println!("eth_blockNumber: {}", client.block_number().await?);
//!     Ok(())
//! }
//! ```
//!

pub(crate) mod brc20_controller;
pub(crate) mod db;
pub(crate) mod evm;
pub(crate) mod global;
pub(crate) mod server;
pub(crate) mod shared_data;

pub use global::Brc20ProgConfig;
pub use server::api::Brc20ProgApiClient;
pub use server::start;

pub mod types {
    //! This module contains the types used in the BRC20 programmable module.
    //!
    //! The types are used to interact with the BRC20 programmable module and the JSON-RPC server.
    pub use crate::db::types::{
        AddressED, BlockResponseED, BytecodeED, BytesED, FixedBytesED, LogED, TraceED, TxED,
        TxReceiptED, UintED, B2048ED, B256ED, U128ED, U256ED, U512ED, U64ED, U8ED,
    };
    pub use crate::server::types::{EncodedBytes, EthCall, GetLogsFilter};
}
