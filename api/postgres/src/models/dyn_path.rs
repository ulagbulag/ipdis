use ipis::core::chrono::{DateTime, Utc};

#[derive(Queryable)]
pub struct DynPaths {
    pub id: i32,
    pub account: String,
    pub signature: String,
    pub created_date: DateTime<Utc>,
    pub expiration_date: DateTime<Utc>,
    pub name: String,
    pub path: String,
    pub len: i64,
}
