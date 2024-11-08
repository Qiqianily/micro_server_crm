use crate::pb::User;
use prost_types::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};

impl User {
    pub fn new(id: u64, name: &str, email: &str) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Self {
            id,
            name: name.to_string(),
            email: email.to_string(),
            created_at: Some(Timestamp {
                seconds: now.as_secs() as i64,
                nanos: now.subsec_nanos() as i32,
            }),
        }
    }
}
