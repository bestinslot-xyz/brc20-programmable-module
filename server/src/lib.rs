mod types;

mod evm;

mod rpc_server;
pub use rpc_server::start_rpc_server;

mod server_instance;
pub use server_instance::ServerInstance;

mod api;
pub use api::Brc20ProgApiServer;

pub static BRC20_CONTROLLER_ADDRESS: &str = "0xc54dd4581af2dbf18e4d90840226756e9d2b3cdb";
pub static INDEXER_ADDRESS: &str = "0x0000000000000000000000000000000000003Ca6";
