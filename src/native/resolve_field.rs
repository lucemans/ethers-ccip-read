use ethers_core::abi::ParamType;
use ethers_providers::{parameterhash, Middleware, FIELD_SELECTOR};

use crate::{error::CCIPMiddlewareError, CCIPReadMiddleware};

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    /// Resolve a field of an ENS name
    pub async fn resolve_field(
        &self,
        ens_name: &str,
        field: &str,
    ) -> Result<String, CCIPMiddlewareError<M>> {
        let field: String = self
            .query_resolver_parameters(
                ParamType::String,
                ens_name,
                FIELD_SELECTOR,
                Some(&parameterhash(field)),
            )
            .await?;
        Ok(field)
    }
}
