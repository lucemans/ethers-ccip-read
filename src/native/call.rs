use std::str::FromStr;

use async_recursion::async_recursion;
use ethers_core::{
    abi::{self, Address, ParamType, Token},
    types::{transaction::eip2718::TypedTransaction, BlockId, BlockNumber, Bytes, NameOrAddress},
    utils::serialize,
};
use ethers_providers::{Middleware, MiddlewareError};
use hex::FromHex;

use crate::{error::CCIPMiddlewareError, CCIPReadMiddleware};

static MAX_CCIP_REDIRECT_ATTEMPT: u8 = 10;

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    pub async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, CCIPMiddlewareError<M>> {
        self._call(tx, block, 0).await
    }

    #[async_recursion]
    pub async fn _call(
        &self,
        transaction: &TypedTransaction,
        block_id: Option<BlockId>,
        attempt: u8,
    ) -> Result<Bytes, CCIPMiddlewareError<M>> {
        if attempt >= MAX_CCIP_REDIRECT_ATTEMPT {
            // may need more info
            return Err(CCIPMiddlewareError::MaxRedirectionError);
        }

        let tx_sender = match transaction.to().unwrap() {
            NameOrAddress::Name(ens_name) => self.resolve_name(ens_name).await?,
            NameOrAddress::Address(addr) => *addr,
        };

        // let tx_value: Value = utils::serialize(transaction);
        let block_value = serialize(&block_id.unwrap_or_else(|| BlockNumber::Latest.into()));
        let result = match self.inner().call(transaction, block_id).await {
            Ok(response) => response.to_string(),
            Err(provider_error) => {
                if !provider_error.is_error_response() {
                    return Err(CCIPMiddlewareError::MiddlewareError(provider_error));
                }

                let content = provider_error
                    .as_error_response().unwrap();
                let data = content.data.as_ref().unwrap_or(&serde_json::Value::Null);
                if data.is_null() {
                    return Err(CCIPMiddlewareError::TodoError("Data is null".to_string()));
                }
                data.to_string()
                    .trim_matches('"')
                    .trim_start_matches("0x")
                    .to_string()
            }
        };

        if block_value.eq("latest")
            && !tx_sender.is_zero()
            && result.starts_with("556f1830")
            && hex::decode(result.clone()).unwrap().len() % 32 == 4
        {
            let output_types = vec![
                ParamType::Address,                            // 'address'
                ParamType::Array(Box::new(ParamType::String)), // 'string[]'
                ParamType::Bytes,                              // 'bytes'
                ParamType::FixedBytes(4),                      // 'bytes4'
                ParamType::Bytes,                              // 'bytes'
            ];

            let decoded_data: Vec<abi::Token> =
                abi::decode(&output_types, &Vec::from_hex(&result.clone()[8..]).unwrap()).unwrap();

            if let (
                Token::Address(addr),
                Token::Array(strings),
                Token::Bytes(bytes),
                Token::FixedBytes(bytes4),
                Token::Bytes(bytes2),
            ) = (
                decoded_data.get(0).unwrap(),
                decoded_data.get(1).unwrap(),
                decoded_data.get(2).unwrap(),
                decoded_data.get(3).unwrap(),
                decoded_data.get(4).unwrap(),
            ) {
                let sender: Address = *addr;
                let urls: Vec<&str> = strings
                    .iter()
                    .map(|t| match t {
                        Token::String(s) => s.as_str(),
                        _ => panic!("CCIP Read contained corrupt URL string"),
                    })
                    .collect();

                let call_data: &[u8] = bytes;
                let callback_selector: Vec<u8> = bytes4.clone();
                let extra_data: &[u8] = bytes2;

                if !sender.eq(&tx_sender) {
                    return Err(CCIPMiddlewareError::TodoError("SenderError".to_string()));
                }

                let ccip_result = self
                    ._ccip_request(sender, transaction, call_data, urls)
                    .await?;
                if ccip_result.is_empty() {
                    return Err(CCIPMiddlewareError::TodoError(
                        "GatewayNotFoundError".to_string(),
                    ));
                }

                let ccip_result_token = Token::Bytes(ccip_result.as_ref().to_vec());
                let extra_data_token = Token::Bytes(extra_data.into());

                let tokens = vec![ccip_result_token, extra_data_token];

                let encoded_data = abi::encode(&tokens);
                let mut new_transaction = transaction.clone();
                new_transaction.set_data(Bytes::from(
                    [callback_selector.clone(), encoded_data.clone()].concat(),
                ));

                return self._call(&new_transaction, block_id, attempt + 1).await;
            }
        }

        let result = match Bytes::from_str(&result) {
            Ok(bytes) => bytes,
            Err(error) => {
                println!("error: {:?}", error);
                return Err(CCIPMiddlewareError::TodoError("GatewayError".to_string()));
            }
        };

        Ok(result)
    }
}
