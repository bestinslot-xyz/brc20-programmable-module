mod cached_database;
mod database;
mod db;

pub(crate) const MAX_HISTORY_SIZE: u64 = 10;

pub mod types;
pub use db::DB;
