#[cfg(any(feature = "postgresql", feature = "sea-orm"))]
pub fn get_expires_at(ttl: Option<chrono::Duration>) -> Option<chrono::NaiveDateTime> {
    match ttl {
        Some(ttl) => {
            let expires_at = chrono::Utc::now().naive_utc() + ttl;

            Some(expires_at)
        }
        None => None,
    }
}
