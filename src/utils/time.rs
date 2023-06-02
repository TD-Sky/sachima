use std::sync::OnceLock;
use std::time::SystemTime;
use time::{OffsetDateTime, UtcOffset};

static LOCAL_OFFSET: OnceLock<UtcOffset> = OnceLock::new();

pub fn init() {
    let local_offset = UtcOffset::current_local_offset().unwrap();
    LOCAL_OFFSET.set(local_offset).unwrap();
}

pub fn local_date_time(systime: SystemTime) -> OffsetDateTime {
    OffsetDateTime::from(systime).to_offset(*LOCAL_OFFSET.get().unwrap())
}
