table! {
    auth (user) {
        user -> Text,
        password -> Binary,
        token -> Binary,
        salt -> Binary,
        comment -> Nullable<Text>,
        id -> Nullable<Text>,
        created_at -> Text,
        last_use -> Text,
    }
}

table! {
    instances (instance) {
        instance -> Integer,
        friendly_name -> Text,
        enabled -> Integer,
        last_use -> Text,
    }
}

table! {
    meta (uuid) {
        uuid -> Text,
        created_at -> Text,
    }
}

table! {
    settings (type_, hyperion_inst) {
        #[sql_name = "type"]
        type_ -> Text,
        config -> Text,
        hyperion_inst -> Nullable<Integer>,
        updated_at -> Text,
    }
}

joinable!(settings -> instances (hyperion_inst));

allow_tables_to_appear_in_same_query!(auth, instances, meta, settings,);
