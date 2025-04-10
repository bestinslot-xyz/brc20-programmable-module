use std::error::Error;

use hex::FromHexError;
use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use tracing::{event, Level};

pub fn wrap_rpc_error(error: Box<dyn Error>) -> ErrorObject<'static> {
    event!(Level::ERROR, "Error: {:?}", error);
    ErrorObjectOwned::owned(400, error.to_string(), None::<String>)
}

pub fn wrap_hex_error(error: FromHexError) -> ErrorObject<'static> {
    event!(Level::ERROR, "Error: {:?}", error);
    ErrorObjectOwned::owned(400, error.to_string(), None::<String>)
}

pub fn wrap_rpc_error_string(error: &str) -> ErrorObject<'static> {
    event!(Level::ERROR, "Error: {:?}", error);
    ErrorObjectOwned::owned(400, error, None::<String>)
}

pub fn wrap_rpc_error_string_with_data(error: &str, data: String) -> ErrorObject<'static> {
    event!(Level::ERROR, "Error: {:?}", error);
    ErrorObjectOwned::owned(400, error, data.into())
}
