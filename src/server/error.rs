use std::error::Error;

use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use tracing::error;

pub fn wrap_rpc_error(error: Box<dyn Error>) -> ErrorObject<'static> {
    error!("{:?}", error);
    ErrorObjectOwned::owned(400, error.to_string(), None::<String>)
}

pub fn wrap_rpc_error_string(error: &str) -> ErrorObject<'static> {
    error!("{:?}", error);
    ErrorObjectOwned::owned(400, error, None::<String>)
}

pub fn wrap_rpc_error_string_with_data(
    code: i32,
    error: &str,
    data: String,
) -> ErrorObject<'static> {
    error!("{:?}", error);
    ErrorObjectOwned::owned(code, error, data.into())
}
