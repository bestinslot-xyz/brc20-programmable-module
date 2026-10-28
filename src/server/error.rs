use std::error::Error;

use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use tracing::error;

use crate::engine::BITCOIN_DATA_UNAVAILABLE;

pub fn wrap_rpc_error(error: Box<dyn Error>) -> ErrorObject<'static> {
    error!("{:?}", error);
    ErrorObjectOwned::owned(400, error.to_string(), None::<String>)
}

pub fn wrap_rpc_error_string(error: &str) -> ErrorObject<'static> {
    error!("{:?}", error);
    ErrorObjectOwned::owned(400, error, None::<String>)
}

/// Error for a public call that could not run. A failed Bitcoin lookup is named in the message.
pub fn call_failed(error: Box<dyn Error>) -> ErrorObject<'static> {
    let message = if error.to_string().contains(BITCOIN_DATA_UNAVAILABLE) {
        format!("Call failed: {}", BITCOIN_DATA_UNAVAILABLE)
    } else {
        "Call failed".to_string()
    };
    wrap_rpc_error_string_with_data(3, &message, "0x".into())
}

pub fn wrap_rpc_error_string_with_data(
    code: i32,
    error: &str,
    data: String,
) -> ErrorObject<'static> {
    error!("{:?}", error);
    ErrorObjectOwned::owned(code, error, data.into())
}
