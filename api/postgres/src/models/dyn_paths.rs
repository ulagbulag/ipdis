use ipis::core::chrono::NaiveDateTime;

use crate::schema::dyn_paths;

#[derive(Debug, Queryable)]
pub struct DynPath {
    pub id: i32,
    pub account: String,
    pub signature: String,
    pub created_date: NaiveDateTime,
    pub expiration_date: Option<NaiveDateTime>,
    pub kind: String,
    pub word: String,
    pub path: String,
    pub len: i64,
}

#[derive(Insertable)]
#[table_name = "dyn_paths"]
pub struct NewDynPath {
    pub account: String,
    pub signature: String,
    pub created_date: NaiveDateTime,
    pub expiration_date: Option<NaiveDateTime>,
    pub kind: String,
    pub word: String,
    pub path: String,
    pub len: i64,
}
