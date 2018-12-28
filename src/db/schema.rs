table! {
    tokens (id) {
        id -> Text,
        user_id -> Nullable<Text>,
        service -> Text,
        service_userid -> Text,
        access_token -> Text,
        access_token_expiry -> Timestamp,
        refresh_token -> Text,
    }
}

table! {
    users (id) {
        id -> Text,
        email -> Text,
    }
}

joinable!(tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    tokens,
    users,
);