pub mod types;

mod rpc_server;
pub use rpc_server::start_rpc_server;

mod server_instance;
pub use server_instance::ServerInstance;

mod api;
pub use api::Brc20ProgApiServer;

pub static INDEXER_ADDRESS: &str = "0x0000000000000000000000000000000000003Ca6";
