pub mod claims;
pub use claims::Claims;

#[cfg(test)]
mod tests;

pub mod prelude;
use prelude::*;

pub use jwt::Error;
use jwt::Header;
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug)]
pub struct Codec<H> {
    hashed_key: H,
}

impl<H> Codec<H> {
    pub fn gen_token<C>(&self, claims: &C) -> Result<String, Error>
    where
        H: SigningAlgorithm,
        C: Serialize,
    {
        let header = Header {
            algorithm: self.hashed_key.algorithm_type(),
            ..Default::default()
        };

        Ok(jwt::Token::new(header, claims)
            .sign_with_key(&self.hashed_key)?
            .as_str()
            .to_owned())
    }

    #[inline]
    pub fn parse_token<C>(&self, token_str: &str) -> Result<C, Error>
    where
        H: VerifyingAlgorithm,
        C: DeserializeOwned,
    {
        token_str.verify_with_key(&self.hashed_key)
    }
}

macro_rules! impl_hs_new {
    ($($hs_type:ty => $hs_func:ident),*,) => {
        $(impl_hs_new!($hs_type => $hs_func);)*
    };

    ($hs_type:ty => $hs_func:ident) => {
        impl Codec<$hs_type> {
            pub fn $hs_func(key: &[u8]) -> Self {
                Self {
                    hashed_key: hmac::Mac::new_from_slice(key).unwrap(),
                }
            }
        }
    }
}

impl_hs_new! {
    Hs256 => hs256,
    Hs384 => hs384,
    Hs512 => hs512,
}
