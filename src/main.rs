use std::sync::Mutex;

use db::DB;
use server::start_server;

fn main() {
    start_server(Mutex::new(DB::new().unwrap()));
}
