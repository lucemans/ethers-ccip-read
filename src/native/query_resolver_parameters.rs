use ethers_core::{
    abi::{self, Detokenize, ParamType, Token},
    types::{transaction::eip2718::TypedTransaction, Bytes, Selector},
};
use ethers_providers::{resolve, Middleware};

use crate::{
    error::CCIPMiddlewareError,
    utils::{decode_bytes::decode_bytes, dns_encode::dns_encode},
    CCIPReadMiddleware,
};

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    pub async fn query_resolver_parameters<T: Detokenize>(
        &self,
        param: ParamType,
        ens_name: &str,
        selector: Selector,
        parameters: Option<&[u8]>,
    ) -> Result<T, CCIPMiddlewareError<M>> {
        let resolver_address = self.get_resolver(ens_name).await?;

        let mut tx: TypedTransaction =
            resolve(resolver_address, selector, ens_name, parameters).into();

        let mut parse_bytes = false;
        if self.supports_wildcard(resolver_address).await? {
            parse_bytes = true;

            let dns_encode_token = Token::Bytes(dns_encode(ens_name).unwrap());
            let tx_data_token = Token::Bytes(tx.data().unwrap().to_vec());

            let tokens = vec![dns_encode_token, tx_data_token];

            let encoded_data = abi::encode(&tokens);

            let resolve_selector = "9061b923";

            // selector("resolve(bytes,bytes)")
            tx.set_data(Bytes::from(
                [hex::decode(resolve_selector).unwrap(), encoded_data].concat(),
            ));
        }

        // resolve
        let mut data = self.inner().call(&tx, None).await.map_err(|e| {
            CCIPMiddlewareError::TodoError(format!("Error calling resolver: {}", e.to_string()))
        })?;
        if parse_bytes {
            data = decode_bytes(ParamType::Bytes, data);
        }

        Ok(decode_bytes(param, data))
    }
}
