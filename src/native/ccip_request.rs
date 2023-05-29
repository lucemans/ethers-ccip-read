use std::fmt::Display;

use ethers_core::types::{transaction::eip2718::TypedTransaction, Address, Bytes};
use ethers_providers::Middleware;
use serde::Deserialize;
use thiserror::Error;

use crate::{error::CCIPMiddlewareError, CCIPReadMiddleware};

#[derive(Debug, Deserialize)]
pub struct CCIPReturnType {
    message: Option<String>,
    data: Option<String>,
}

#[derive(Error, Debug)]
pub enum CCIPRequestError {
    #[error("Gateway Error: {0}")]
    GatewayError(String),

    #[error("No message")]
    NoMessage(),

    #[error("Message")]
    Message(String),

    #[error("Failed to decode {0}")]
    DecodeDataHex(String),
}

#[derive(Error, Debug)]
pub struct CCIPGatewayErrors {
    inner: Vec<CCIPRequestError>,
}

impl Display for CCIPGatewayErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut error_string = String::new();

        for error in &self.inner {
            error_string.push_str(&format!("{}\n", error));
        }

        write!(f, "{}", error_string)
    }
}

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    /// This function makes a Cross-Chain Interoperability Protocol (CCIP-Read) request
    /// and returns the result as `Bytes` or an error message.
    ///
    /// # Arguments
    ///
    /// * `sender`: The sender's address.
    /// * `tx`: The typed transaction.
    /// * `calldata`: The function call data as bytes.
    /// * `urls`: A vector of Offchain Gateway URLs to send the request to.
    ///
    /// # Returns
    ///
    /// an opaque byte string to send to callbackFunction on Offchain Resolver contract.
    pub async fn _ccip_request(
        &self,
        sender: Address,
        tx: &TypedTransaction,
        calldata: &[u8],
        urls: Vec<&str>,
    ) -> Result<Bytes, CCIPMiddlewareError<M>> {
        // If there are no URLs or the transaction's destination is empty, return an empty result
        if urls.is_empty() || tx.to().is_none() {
            return Ok(Bytes::from([]));
        }

        // Convert calldata to a hex string
        let data: String = calldata
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();

        let mut error_messages = CCIPGatewayErrors { inner: vec![] };

        for (_i, url) in urls.iter().enumerate() {
            // Replace the placeholders in the URL with the sender address and data
            let href = url
                .replace("{sender}", &format!("0x{:x}", sender))
                .replace("{data}", &format!("0x{}", &data.to_lowercase()).to_string());

            let result: CCIPReturnType = match url.contains("{data}") {
                true => reqwest::Client::new().get(&href),
                // If the URL does not contain the "{data}" placeholder, create a POST request instead
                false => reqwest::Client::new()
                    .post(&href)
                    .json(&serde_json::json!({ "data": data, "sender": sender })),
            }
            .send()
            .await?
            .json()
            .await?;

            // If the result contains the "data" field, decode the data and return it as Bytes
            if let Some(returned_data) = result.data {
                match hex::decode(&returned_data[2..]) {
                    Ok(decoded) => return Ok(Bytes::from(decoded)),
                    Err(e) => {
                        error_messages.inner.push(CCIPRequestError::DecodeDataHex(e.to_string()));
                        
                        continue;
                    },
                }
            };

            error_messages.inner.push(match result.message {
                Some(message) => CCIPRequestError::GatewayError(message),
                None => CCIPRequestError::NoMessage(),
            });
        }

        Err(CCIPMiddlewareError::GatewayError(error_messages))
    }
}
