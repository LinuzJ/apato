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
        price -> Nullable<Text>,
        additional_costs -> Nullable<Int4>,
        rent -> Nullable<Int4>,
        watchlist_id -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    watchlists (id) {
        id -> Int4,
        location_id -> Nullable<Int4>,
        location_level -> Nullable<Int4>,
        location_name -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    apartments,
    watchlists,
);
