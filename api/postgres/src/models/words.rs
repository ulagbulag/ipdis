use ipis::core::{chrono::NaiveDateTime, uuid::Uuid};

#[derive(Debug, Queryable)]
pub struct Word {
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
    pub parent: String,
    pub lang: String,
    pub word: String,
    pub relpath: bool,
    pub path: String,
    pub len: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::words)]
pub struct NewWord {
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
    pub parent: String,
    pub lang: String,
    pub word: String,
    pub relpath: bool,
    pub path: String,
    pub len: i64,
}

#[derive(Debug, Queryable)]
pub struct WordCount {
    pub id: i32,
    pub namespace: String,
    pub kind: String,
    pub parent: String,
    pub lang: String,
    pub word: String,
    pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::words_counts)]
pub struct NewWordCount {
    pub namespace: String,
    pub kind: String,
    pub parent: String,
    pub lang: String,
    pub word: String,
    pub count: i64,
}

#[derive(Debug, Queryable)]
pub struct WordCountGuarantee {
    pub id: i32,
    pub guarantee: String,
    pub namespace: String,
    pub kind: String,
    pub parent: String,
    pub lang: String,
    pub word: String,
    pub count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::words_counts_guarantees)]
pub struct NewWordCountGuarantee {
    pub guarantee: String,
    pub namespace: String,
    pub kind: String,
    pub parent: String,
    pub lang: String,
    pub word: String,
    pub count: i64,
}
