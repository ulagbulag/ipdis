// @generated automatically by Diesel CLI.

diesel::table! {
    accounts_guarantees (id) {
        id -> Int4,
        nonce -> Uuid,
        guarantee -> Varchar,
        guarantor -> Varchar,
        guarantee_signature -> Varchar,
        guarantor_signature -> Varchar,
        created_date -> Timestamp,
        expiration_date -> Nullable<Timestamp>,
        hash -> Varchar,
    }
}

diesel::table! {
    dyn_paths (id) {
        id -> Int4,
        nonce -> Uuid,
        guarantee -> Varchar,
        guarantor -> Varchar,
        guarantee_signature -> Varchar,
        guarantor_signature -> Varchar,
        created_date -> Timestamp,
        expiration_date -> Nullable<Timestamp>,
        hash -> Varchar,
        namespace -> Varchar,
        kind -> Varchar,
        word -> Varchar,
        path -> Varchar,
        len -> Int8,
    }
}

diesel::table! {
    words (id) {
        id -> Int4,
        nonce -> Uuid,
        guarantee -> Varchar,
        guarantor -> Varchar,
        guarantee_signature -> Varchar,
        guarantor_signature -> Varchar,
        created_date -> Timestamp,
        expiration_date -> Nullable<Timestamp>,
        hash -> Varchar,
        namespace -> Varchar,
        kind -> Varchar,
        parent -> Varchar,
        lang -> Varchar,
        word -> Varchar,
        relpath -> Bool,
        path -> Varchar,
        len -> Int8,
    }
}

diesel::table! {
    words_counts (id) {
        id -> Int4,
        namespace -> Varchar,
        kind -> Varchar,
        parent -> Varchar,
        lang -> Varchar,
        word -> Varchar,
        count -> Int8,
    }
}

diesel::table! {
    words_counts_guarantees (id) {
        id -> Int4,
        guarantee -> Varchar,
        namespace -> Varchar,
        kind -> Varchar,
        parent -> Varchar,
        lang -> Varchar,
        word -> Varchar,
        count -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    accounts_guarantees,
    dyn_paths,
    words,
    words_counts,
    words_counts_guarantees,
);
