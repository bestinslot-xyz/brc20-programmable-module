use std::error::Error;

use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use tracing::error;

use crate::engine::BitcoinDataUnavailable;

pub fn wrap_rpc_error(error: Box<dyn Error>) -> ErrorObject<'static> {
    error!("{:?}", error);
    ErrorObjectOwned::owned(400, error.to_string(), None::<String>)
}

pub fn wrap_rpc_error_string(error: &str) -> ErrorObject<'static> {
    error!("{:?}", error);
    ErrorObjectOwned::owned(400, error, None::<String>)
}

/// Whether a public call failed because a Bitcoin lookup failed.
pub fn is_bitcoin_unavailable(error: &(dyn Error + 'static)) -> bool {
    error.downcast_ref::<BitcoinDataUnavailable>().is_some()
}

/// Error for a public call that could not run. A failed Bitcoin lookup is named in the message.
pub fn call_failed(error: Box<dyn Error>) -> ErrorObject<'static> {
    let message = if is_bitcoin_unavailable(error.as_ref()) {
        format!("Call failed: {}", BitcoinDataUnavailable)
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
