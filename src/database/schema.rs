table! {
    icons (owner, repo, path) {
        owner -> Text,
        repo -> Text,
        path -> Text,
    }
}

table! {
    repos (owner, repo, path) {
        owner -> Text,
        repo -> Text,
        path -> Text,
        icon_path -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    icons,
    repos,
);
