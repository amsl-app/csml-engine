use sea_schema::postgres::def::ColumnInfo as PgColumnInfo;
use sea_schema::sqlite::def::ColumnInfo as SqliteColumnInfo;
use std::collections::HashMap;
use test_helpers::util::{TestableTable, ValidatableTable, postgres, sqlite};

pub struct CsmlBotVersions;

impl TestableTable for CsmlBotVersions {
    fn name() -> &'static str {
        "csml_bot_versions"
    }

    fn size() -> usize {
        6
    }
}

impl ValidatableTable<&PgColumnInfo> for CsmlBotVersions {
    fn validate(columns: &HashMap<&str, &PgColumnInfo>) {
        postgres::assert_uuid!(columns, "id");
        postgres::assert_string!(columns, "bot_id");
        postgres::assert_string!(columns, "bot");
        postgres::assert_string!(columns, "engine_version");
        postgres::assert_timestamp!(columns, "updated_at");
        postgres::assert_timestamp!(columns, "created_at");
    }
}

impl ValidatableTable<&SqliteColumnInfo> for CsmlBotVersions {
    fn validate(columns: &HashMap<&str, &SqliteColumnInfo>) {
        sqlite::assert_uuid!(columns, "id");
        sqlite::assert_text!(columns, "bot_id");
        sqlite::assert_text!(columns, "bot");
        sqlite::assert_text!(columns, "engine_version");
        sqlite::assert_timestamp!(columns, "updated_at");
        sqlite::assert_timestamp!(columns, "created_at");
    }
}
