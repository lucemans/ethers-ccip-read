use ethers_core::{abi::ParamType, types::{Address, H160}};
use ethers_providers::{Middleware, ENS_ADDRESS, get_resolver};

use crate::{error::CCIPMiddlewareError, CCIPReadMiddleware, utils::decode_bytes::decode_bytes};

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    pub async fn get_resolver(&self, ens_name: &str) -> Result<H160, CCIPMiddlewareError<M>> {
        let mut current_name: String = ens_name.to_string();

        let ens_addr = self.ens.unwrap_or(ENS_ADDRESS);

        loop {
            if current_name.eq("") || current_name.eq(".") {
                return Ok(H160::zero());
            }

            if !ens_name.eq("eth") && current_name.eq("eth") {
                return Ok(H160::zero());
            }

            let data = self
                .inner().call(
                    &get_resolver(ens_addr, &current_name.to_string()).into(),
                    None,
                )
                .await.map_err(|x| {
                    CCIPMiddlewareError::TodoError(format!("GR Error calling resolver: {}", x.to_string()))
                })?;

            if data.0.is_empty() {
                return Ok(H160::zero());
            }

            let resolver_address: Address = decode_bytes(ParamType::Address, data);

            if resolver_address != Address::zero() {
                if current_name != ens_name && !self.supports_wildcard(resolver_address).await? {
                    return Ok(H160::zero());
                }
                return Ok(resolver_address);
            }

            let mut splitted_name: Vec<&str> = current_name.split('.').collect();
            current_name = splitted_name.split_off(1).join(".").to_string();
        }
    }
}
