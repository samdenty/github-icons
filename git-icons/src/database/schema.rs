table! {
    icons (url) {
        url -> Text,
        kind -> Text,
        sizes -> Text,
        #[sql_name = "type"]
        type_ -> Text,
    }
}

table! {
    repos (owner, name, path) {
        owner -> Text,
        name -> Text,
        path -> Text,
        icon -> Text,
    }
}

allow_tables_to_appear_in_same_query!(icons, repos,);
