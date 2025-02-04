use db::DB;
use server::{start_http_server, ServerInstance};

fn main() {
    start_http_server(ServerInstance::new(DB::new().unwrap()));
}
