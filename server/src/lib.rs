mod types;

mod evm;

mod rpc_server;
pub use rpc_server::start_rpc_server;

mod server_instance;
pub use server_instance::ServerInstance;

mod api;
pub use api::Brc20ProgApiServer;

mod contracts;
pub use contracts::*;

pub static INDEXER_ADDRESS: &str = "0x0000000000000000000000000000000000003Ca6";
