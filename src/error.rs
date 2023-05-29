use ethers_providers::{JsonRpcError, Middleware};
use crate::native::ccip_request::{CCIPGatewayErrors};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CCIPMiddlewareError<M: Middleware> {
    #[error(transparent)]
    RPCError(#[from] JsonRpcError),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    GatewayError(#[from] CCIPGatewayErrors),

    #[error("Max redirection attempts reached")]
    MaxRedirectionError,

    #[error("Todo but Error is {0}")]
    TodoError(String),

    /// Thrown when the internal middleware errors
    #[error(transparent)]
    MiddlewareError(M::Error),
}
