use ethers_core::{
    abi::ParamType,
    types::Address,
};
use ethers_providers::{Middleware, ADDR_SELECTOR};

use crate::{error::CCIPMiddlewareError, CCIPReadMiddleware};

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    /// Resolve an ENS name to an address
    pub async fn resolve_name(&self, ens_name: &str) -> Result<Address, CCIPMiddlewareError<M>> {
        self.query_resolver(ParamType::Address, ens_name, ADDR_SELECTOR)
            .await
    }
}
