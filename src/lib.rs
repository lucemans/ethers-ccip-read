//! # Ethers CCIP-Read
//!
//! Provides an [ethers](https://docs.rs/ethers) compatible middleware for submitting
mod middleware;
pub use middleware::CCIPReadMiddleware;

pub mod utils;

pub mod native;

pub mod error;
