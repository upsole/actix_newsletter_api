table! {
    account (id) {
        id -> Uuid,
        subscribed_at -> Timestamp,
        email -> Varchar,
        name -> Varchar,
        level -> Int4,
    }
}
