// @generated automatically by Diesel CLI.

diesel::table! {
    apartments (id) {
        id -> Text,
        location_id -> Nullable<Int4>,
        location_level -> Nullable<Int4>,
        location_name -> Nullable<Text>,
        size -> Nullable<Float8>,
        rooms -> Nullable<Int4>,
        price -> Nullable<Text>,
        additional_costs -> Nullable<Int4>,
    }
}
