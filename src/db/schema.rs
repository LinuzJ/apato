// @generated automatically by Diesel CLI.

diesel::table! {
    apartments (id) {
        id -> Int4,
        card_id -> Nullable<Text>,
        location_id -> Nullable<Int4>,
        location_level -> Nullable<Int4>,
        location_name -> Nullable<Text>,
        size -> Nullable<Float8>,
        rooms -> Nullable<Int4>,
        price -> Nullable<Int4>,
        additional_costs -> Nullable<Int4>,
        rent -> Nullable<Int4>,
        estimated_yield -> Nullable<Float8>,
        url -> Nullable<Text>,
        watchlist_id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    watchlists (id) {
        id -> Int4,
        location_id -> Int4,
        location_level -> Int4,
        location_name -> Text,
        chat_id -> Int4,
        goal_yield -> Nullable<Float8>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(apartments -> watchlists (watchlist_id));

diesel::allow_tables_to_appear_in_same_query!(
    apartments,
    watchlists,
);
