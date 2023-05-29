use ethers_core::{
    abi::{ParamType, Detokenize},
    types::{Selector},
};
use ethers_providers::{Middleware};

use crate::{error::CCIPMiddlewareError, CCIPReadMiddleware};

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    pub async fn query_resolver<T: Detokenize>(
        &self,
        param: ParamType,
        ens_name: &str,
        selector: Selector,
    ) -> Result<T, CCIPMiddlewareError<M>> {
        self.query_resolver_parameters(param, ens_name, selector, None)
            .await
    }
}
