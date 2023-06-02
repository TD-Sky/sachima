use sha2::Digest;
use sha2::Sha256;
use std::sync::OnceLock;

static HASHER: OnceLock<Sha256> = OnceLock::new();

pub fn init(secret_key: &str) {
    let hasher = Sha256::new_with_prefix(secret_key);
    HASHER.set(hasher).unwrap();
}

pub fn hash(plaintext: &str) -> String {
    format!(
        "{:x}",
        HASHER
            .get()
            .unwrap()
            .clone()
            .chain_update(plaintext)
            .finalize()
    )
}

#[inline]
pub fn verify(plaintext: &str, password: &str) -> bool {
    hash(plaintext) == password
}
