#![allow(proc_macro_derive_resolution_fallback)]

table! {
    calories (user_id, time) {
        time -> Timestamptz,
        user_id -> Uuid,
        source -> Text,
        count -> Float8,
        level -> Int4,
        mets -> Int4,
    }
}

table! {
    distances (user_id, time) {
        time -> Timestamptz,
        user_id -> Uuid,
        source -> Text,
        count -> Float8,
    }
}

table! {
    elevations (user_id, time) {
        time -> Timestamptz,
        user_id -> Uuid,
        source -> Text,
        count -> Float8,
    }
}

table! {
    floors (user_id, time) {
        time -> Timestamptz,
        user_id -> Uuid,
        source -> Text,
        count -> Int4,
    }
}

table! {
    steps (user_id, time) {
        time -> Timestamptz,
        user_id -> Uuid,
        source -> Text,
        count -> Int4,
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

joinable!(calories -> users (user_id));
joinable!(distances -> users (user_id));
joinable!(elevations -> users (user_id));
joinable!(floors -> users (user_id));
joinable!(steps -> users (user_id));
joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    calories,
    distances,
    elevations,
    floors,
    steps,
    tokens,
    users,
);
