use ipis::core::{chrono::NaiveDateTime, uuid::Uuid};

#[derive(Debug, Queryable)]
pub struct IdfWord {
    pub id: i32,
    pub kind: String,
    pub lang: String,
    pub word: String,
    pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::idf_words)]
pub struct NewIdfWord {
    pub kind: String,
    pub lang: String,
    pub word: String,
    pub count: i64,
}

#[derive(Debug, Queryable)]
pub struct IdfWordGuarantee {
    pub id: i32,
    pub guarantee: String,
    pub kind: String,
    pub lang: String,
    pub word: String,
    pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::idf_words_guarantees)]
pub struct NewIdfWordGuarantee {
    pub guarantee: String,
    pub kind: String,
    pub lang: String,
    pub word: String,
    pub count: i64,
}

#[derive(Debug, Queryable)]
pub struct IdfLog {
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
    pub kind: String,
    pub lang: String,
    pub word: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::idf_logs)]
pub struct NewIdfLog {
    // -- METADATA BEGIN --
    pub nonce: Uuid,
    pub guarantee: String,
    pub guarantor: String,
    pub guarantee_signature: String,
    pub guarantor_signature: String,
    pub created_date: NaiveDateTime,
    pub expiration_date: Option<NaiveDateTime>,
    // -- METADATA END --
    pub kind: String,
    pub lang: String,
    pub word: String,
}
