pub use jwt::claims::{RegisteredClaims, SecondsSinceEpoch};

use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims<P> {
    #[serde(flatten)]
    pub registered: RegisteredClaims,
    #[serde(flatten)]
    pub private: P,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NoPrivate {}

impl<P> Claims<P> {
    pub fn new(private: P) -> Self {
        Self {
            registered: RegisteredClaims::default(),
            private,
        }
    }
}

fn to_unix_timestamp(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

macro_rules! impl_build_claims {
    ($($claim:tt: $claim_type:ty),*,) => {
        impl<P> Claims<P> {
            $(impl_build_claims!($claim: $claim_type);)*
        }
    };

    ($claim:ident: $claim_type:ty) => {
        pub fn $claim(mut self, $claim: $claim_type) -> Self {
            self.registered.$claim = Some($claim);
            self
        }
    };
}

impl_build_claims! {
    issuer: String,
    subject: String,
    audience: String,
    expiration: SecondsSinceEpoch,
    not_before: SecondsSinceEpoch,
    issued_at: SecondsSinceEpoch,
    json_web_token_id: String,
}

macro_rules! impl_valid_for {
    ($($func:ident($unit:ident): $base:expr),*,) => {
        impl<P> Claims<P> {
            $(impl_valid_for!($func($unit): $base);)*
        }
    };

    ($func:ident($unit:ident): $base:expr) => {
        pub fn $func(self, $unit: u64) -> Self {
            let exp = to_unix_timestamp(SystemTime::now() + Duration::from_secs($base * $unit));
            self.expiration(exp)
        }
    };
}

impl_valid_for! {
    valid_secs(seconds): 1,
    valid_mins(minutes): 60,
    valid_hs(hours): 60 * 60,
    valid_days(days): 60 * 60 * 24,
}
