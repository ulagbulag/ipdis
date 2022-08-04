use ipis::core::{chrono::NaiveDateTime, uuid::Uuid};

#[derive(Debug, Queryable)]
pub struct AccountsGuarantee {
    pub id: i32,
    // -- METADATA BEGIN --
    pub nonce: Uuid,
    pub guarantee: String,
    pub guarantor: String,
    pub guarantee_signature: String,
    pub guarantor_signature: String,
    pub created_date: NaiveDateTime,
    pub expiration_date: Option<NaiveDateTime>,
    pub hash: String,
    // -- METADATA END --
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::accounts_guarantees)]
pub struct NewAccountsGuarantee {
    // -- METADATA BEGIN --
    pub nonce: Uuid,
    pub guarantee: String,
    pub guarantor: String,
    pub guarantee_signature: String,
    pub guarantor_signature: String,
    pub created_date: NaiveDateTime,
    pub expiration_date: Option<NaiveDateTime>,
    pub hash: String,
    // -- METADATA END --
}
