use ethers_core::{
    abi::ParamType,
    types::{Bytes, U256},
};
use ethers_providers::{reverse_address, Middleware, NAME_SELECTOR};

use crate::{
    error::CCIPMiddlewareError, utils::selectors::ADDR_MULTI_SELECTOR2, CCIPReadMiddleware,
};

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    pub async fn resolve_addresses(
        &self,
        ens_name: &str,
        coin_type: &str,
    ) -> Result<String, CCIPMiddlewareError<M>> {
        let x = U256::from_dec_str(coin_type).map_err(|x| {
            CCIPMiddlewareError::TodoError("FetchError(Invalid Cointype)".to_owned())
        })?;

        let field: Bytes = self
            .query_resolver_parameters(
                ParamType::Bytes,
                ens_name,
                ADDR_MULTI_SELECTOR2,
                Some(&[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 60,
                ]),
            )
            .await?;
        Ok(format!("{:?}", field))
    }
}
