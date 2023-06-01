use hmac::Hmac;
use sha2::{Sha256, Sha384, Sha512};

pub use jwt::SigningAlgorithm;
pub use jwt::VerifyingAlgorithm;

pub type Hs256 = Hmac<Sha256>;
pub type Hs384 = Hmac<Sha384>;
pub type Hs512 = Hmac<Sha512>;
