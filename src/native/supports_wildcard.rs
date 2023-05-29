use ethers_core::{
    abi::ParamType,
    types::{H160, TransactionRequest, NameOrAddress, Bytes, U256},
};
use ethers_providers::{Middleware};

use crate::{CCIPReadMiddleware, error::CCIPMiddlewareError, utils::decode_bytes::decode_bytes};

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
        /// The supports_wildcard checks if a given resolver supports the wildcard resolution by calling
    /// its `supportsInterface` function with the `resolve(bytes,bytes)` selector.
    ///
    /// # Arguments
    ///
    /// * `resolver_address`: The resolver's address.
    ///
    /// # Returns
    ///
    /// A `Result` with either a `bool` value indicating if the resolver supports wildcard
    /// resolution or a `ProviderError`.
    pub async fn supports_wildcard(
        &self,
        resolver_address: H160,
    ) -> Result<bool, CCIPMiddlewareError<M>> {
        // Prepare the data for the `supportsInterface` call, providing the selector for
        // the "resolve(bytes,bytes)" function
        let data = Some(
            "0x01ffc9a79061b92300000000000000000000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
        );

        let _tx_request = TransactionRequest {
            data,
            to: Some(NameOrAddress::Address(resolver_address)),
            ..Default::default()
        };

        let _tx_result: Result<Bytes, _> = self.call(&_tx_request.into(), None).await;
        let _tx = match _tx_result {
            Ok(_tx) => _tx,
            Err(_error) => {
                println!("Error calling: {:?}", _error);
                Bytes::from([])
            }
        };

        // If the response is empty, the resolver does not support wildcard resolution
        if _tx.0.is_empty() {
            return Ok(false);
        }

        let _result: U256 = decode_bytes(ParamType::Uint(256), _tx);

        // If the result is one, the resolver supports wildcard resolution; otherwise, it does not
        Ok(_result.eq(&U256::one()))
    }
}
