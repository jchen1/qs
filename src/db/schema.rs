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
    }
}

joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    tokens,
    users,
);