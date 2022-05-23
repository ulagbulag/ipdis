use ipis::core::{chrono::NaiveDateTime, uuid::Uuid};

#[derive(Debug, Queryable)]
pub struct DynPath {
    pub id: i32,
    // -- METADATA BEGIN --
    pub nonce: Uuid,
    pub guarantee: String,
    pub guarantor: String,
    pub guarantee_signature: String,
    pub guarantor_signature: String,
    pub created_date: NaiveDateTime,
    pub expiration_date: Option<NaiveDateTime>,
    // -- METADATA END --
    pub namespace: String,
    pub kind: String,
    pub word: String,
    pub path: String,
    pub len: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::dyn_paths)]
pub struct NewDynPath {
    // -- METADATA BEGIN --
    pub nonce: Uuid,
    pub guarantee: String,
    pub guarantor: String,
    pub guarantee_signature: String,
    pub guarantor_signature: String,
    pub created_date: NaiveDateTime,
    pub expiration_date: Option<NaiveDateTime>,
    // -- METADATA END --
    pub namespace: String,
    pub kind: String,
    pub word: String,
    pub path: String,
    pub len: i64,
}
