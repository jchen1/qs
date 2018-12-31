#![allow(proc_macro_derive_resolution_fallback)]

table! {
    steps (user_id, time) {
        time -> Timestamptz,
        user_id -> Uuid,
        source -> Text,
        count -> Nullable<Int4>,
    }
}

table! {
    tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        service -> Text,
        service_userid -> Text,
        access_token -> Text,
        access_token_expiry -> Timestamptz,
        refresh_token -> Text,
    }
}

table! {
    users (id) {
        id -> Uuid,
        email -> Text,
        g_sub -> Text,
    }
}

joinable!(steps -> users (user_id));
joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    steps,
    tokens,
    users,
);