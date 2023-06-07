use std::time::{SystemTime, UNIX_EPOCH};

#[inline]
pub fn to_unix_timestamp(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap().as_secs()
}
