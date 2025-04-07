pub mod types;

mod rpc_server;
use alloy_primitives::Address;
pub use rpc_server::start_rpc_server;

mod server_instance;
pub use server_instance::ServerInstance;

mod auth;

mod api;

lazy_static::lazy_static! {
    pub static ref CHAIN_ID: u64 = 0x4252433230;
    pub static ref CHAIN_ID_STRING: String = CHAIN_ID.to_string();
    pub static ref INDEXER_ADDRESS: Address = "0x0000000000000000000000000000000000003Ca6".parse().expect("Failed to parse indexer address");
    pub static ref INDEXER_ADDRESS_STRING: String = INDEXER_ADDRESS.to_string();
}
