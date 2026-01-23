pub const ERROR_DB_SETUP: &str = "Database connector is not setup correctly";

#[cfg(any(feature = "sea-orm", feature = "postgresql-async"))]
pub const ERROR_DB_URI: &str = "Db uri is not set";
