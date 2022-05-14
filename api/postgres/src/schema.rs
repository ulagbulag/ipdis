table! {
    dyn_paths (id) {
        id -> Int4,
        nonce -> Uuid,
        guarantee -> Varchar,
        guarantor -> Varchar,
        signature -> Varchar,
        created_date -> Timestamp,
        expiration_date -> Nullable<Timestamp>,
        kind -> Varchar,
        word -> Varchar,
        path -> Varchar,
        len -> Int8,
    }
}

table! {
    idf_logs (id) {
        id -> Int4,
        nonce -> Uuid,
        guarantee -> Varchar,
        guarantor -> Varchar,
        signature -> Varchar,
        created_date -> Timestamp,
        expiration_date -> Nullable<Timestamp>,
        kind -> Varchar,
        word -> Varchar,
    }
}

table! {
    idf_words (id) {
        id -> Int4,
        kind -> Varchar,
        word -> Varchar,
        count -> Int8,
    }
}

allow_tables_to_appear_in_same_query!(
    dyn_paths,
    idf_logs,
    idf_words,
);
