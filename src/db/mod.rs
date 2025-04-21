mod cached_database;
mod database;
mod db;

pub mod types;
pub use db::DB;
pub const MAX_HISTORY_SIZE: u64 = 10;
