use ethers_core::{
    abi::{self, Detokenize, ParamType},
    types::Bytes,
};

/// infallible conversion of Bytes to Address/String
///
/// # Panics
///
/// If the provided bytes were not an interpretation of an address
pub fn decode_bytes<T: Detokenize>(param: ParamType, bytes: Bytes) -> T {
    let tokens = abi::decode(&[param], bytes.as_ref())
        .expect("could not abi-decode bytes to address tokens");
    T::from_tokens(tokens).expect("could not parse tokens as address")
}
