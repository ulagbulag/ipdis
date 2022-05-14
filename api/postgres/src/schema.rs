table! {
    accounts_guarantees (id) {
        id -> Int4,
        nonce -> Uuid,
        guarantee -> Varchar,
        guarantor -> Varchar,
        guarantee_signature -> Varchar,
        guarantor_signature -> Varchar,
        created_date -> Timestamp,
        expiration_date -> Nullable<Timestamp>,
    }
}

table! {
    dyn_paths (id) {
        id -> Int4,
        nonce -> Uuid,
        guarantee -> Varchar,
        guarantor -> Varchar,
        guarantee_signature -> Varchar,
        guarantor_signature -> Varchar,
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
        guarantee_signature -> Varchar,
        guarantor_signature -> Varchar,
        created_date -> Timestamp,
        expiration_date -> Nullable<Timestamp>,
        kind -> Varchar,
        lang -> Varchar,
        word -> Varchar,
    }
}

table! {
    idf_words (id) {
        id -> Int4,
        kind -> Varchar,
        lang -> Varchar,
        word -> Varchar,
        count -> Int8,
    }
}

allow_tables_to_appear_in_same_query!(
    accounts_guarantees,
    dyn_paths,
    idf_logs,
    idf_words,
);
