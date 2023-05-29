use ethers_core::abi::{Address, ParamType};
use ethers_providers::{reverse_address, Middleware, NAME_SELECTOR};

use crate::{error::CCIPMiddlewareError, CCIPReadMiddleware};

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    /// Look up an address to find its primary ENS name
    pub async fn lookup_address(&self, address: Address) -> Result<String, CCIPMiddlewareError<M>> {
        let ens_name = reverse_address(address);
        let domain: String = self
            .query_resolver(ParamType::String, &ens_name, NAME_SELECTOR)
            .await?;
        let reverse_address = self.resolve_name(&domain).await?;
        if address != reverse_address {
            Err(CCIPMiddlewareError::TodoError(format!(
                "User does not own domain: {}",
                domain
            )))
        } else {
            Ok(domain)
        }
    }
}
