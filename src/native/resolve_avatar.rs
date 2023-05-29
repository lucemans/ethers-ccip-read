use std::str::FromStr;

use ethers_core::{
    abi::ParamType,
    types::{Address, NameOrAddress, TransactionRequest},
};
use ethers_providers::{erc, Middleware};
use futures_util::try_join;
use reqwest::Url;

use crate::{error::CCIPMiddlewareError, utils::decode_bytes::decode_bytes, CCIPReadMiddleware};

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    /// Resolve avatar field of an ENS name
    pub async fn resolve_avatar(&self, ens_name: &str) -> Result<Url, CCIPMiddlewareError<M>> {
        let (field, owner) = try_join!(
            self.resolve_field(ens_name, "avatar"),
            self.resolve_name(ens_name)
        )?;
        let url = Url::from_str(&field).map_err(|e| {
            CCIPMiddlewareError::TodoError(format!("URLParseError {}", e.to_string()))
        })?;
        match url.scheme() {
            "https" | "data" => Ok(url),
            "ipfs" => erc::http_link_ipfs(url).map_err(CCIPMiddlewareError::TodoError),
            "eip155" => {
                let token =
                    erc::ERCNFT::from_str(url.path()).map_err(CCIPMiddlewareError::TodoError)?;
                match token.type_ {
                    erc::ERCNFTType::ERC721 => {
                        let tx = TransactionRequest {
                            data: Some(
                                [&erc::ERC721_OWNER_SELECTOR[..], &token.id].concat().into(),
                            ),
                            to: Some(NameOrAddress::Address(token.contract)),
                            ..Default::default()
                        };
                        let data = self.inner().call(&tx.into(), None).await.map_err(|x| {
                            CCIPMiddlewareError::TodoError(format!(
                                "Error calling nft info: {}",
                                x.to_string()
                            ))
                        })?;

                        if decode_bytes::<Address>(ParamType::Address, data) != owner {
                            return Err(CCIPMiddlewareError::TodoError(
                                "NFTOwnerError".to_string(),
                            ));
                        }
                    }
                    erc::ERCNFTType::ERC1155 => {
                        let tx = TransactionRequest {
                            data: Some(
                                [
                                    &erc::ERC1155_BALANCE_SELECTOR[..],
                                    &[0x0; 12],
                                    &owner.0,
                                    &token.id,
                                ]
                                .concat()
                                .into(),
                            ),
                            to: Some(NameOrAddress::Address(token.contract)),
                            ..Default::default()
                        };
                        let data = self.inner().call(&tx.into(), None).await.map_err(|x| {
                            CCIPMiddlewareError::TodoError(format!(
                                "Error calling nft info: {}",
                                x.to_string()
                            ))
                        })?;
                        if decode_bytes::<u64>(ParamType::Uint(64), data) == 0 {
                            return Err(CCIPMiddlewareError::TodoError(
                                "Incorrect Balance".to_string(),
                            ));
                        }
                    }
                }

                let image_url = self.inner().resolve_nft(token).await.map_err(|x| {
                    CCIPMiddlewareError::TodoError(format!(
                        "Error resolving nft: {}",
                        x.to_string()
                    ))
                })?;
                match image_url.scheme() {
                    "https" | "data" => Ok(image_url),
                    "ipfs" => erc::http_link_ipfs(image_url).map_err(|x| {
                        CCIPMiddlewareError::TodoError(format!("URLParseError {}", x))
                    }),
                    _ => Err(CCIPMiddlewareError::TodoError(
                        "UnsupportedURLSchemeError".to_string(),
                    )),
                }
            }
            _ => Err(CCIPMiddlewareError::TodoError(
                "UnsupportedURLSchemeError".to_string(),
            )),
        }
    }
}
