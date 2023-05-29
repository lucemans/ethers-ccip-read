use ethers_core::types::Address;
use ethers_providers::Middleware;

#[derive(Debug, Clone)]
pub struct CCIPReadMiddleware<M>
where
    M: Middleware,
{
    inner: M,
    pub ens: Option<Address>,
}

impl<M> Clone for CCIPReadMiddleware<M>
where
    M: Middleware,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            ens: self.ens,
        }
}

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    pub fn new(inner: M) -> Self {
        Self { inner, ens: None }
    }

    pub fn ens<T: Into<Address>>(mut self, ens: T) -> Self {
        self.ens = Some(ens.into());
        self
    }

    /// Get a reference to the inner middleware
    pub fn inner(&self) -> &M {
        &self.inner
    }
}
