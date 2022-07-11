table! {
    account (id) {
        id -> Uuid,
        subscribed_at -> Timestamp,
        email -> Varchar,
        name -> Varchar,
        status -> Bool,
        auth_token -> Uuid,
    }
}
