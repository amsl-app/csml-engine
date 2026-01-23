use sea_schema::postgres::def::ColumnInfo as PgColumnInfo;
use sea_schema::sqlite::def::ColumnInfo as SqliteColumnInfo;
use std::collections::HashMap;
use test_helpers::util::{TestableTable, ValidatableTable, postgres, sqlite};

pub struct CsmlMemories;

impl TestableTable for CsmlMemories {
    fn name() -> &'static str {
        "csml_memories"
    }

    fn size() -> usize {
        9
    }
}

impl ValidatableTable<&PgColumnInfo> for CsmlMemories {
    fn validate(columns: &HashMap<&str, &PgColumnInfo>) {
        postgres::assert_uuid!(columns, "id");
        postgres::assert_string!(columns, "bot_id");
        postgres::assert_string!(columns, "channel_id");
        postgres::assert_string!(columns, "user_id");
        postgres::assert_string!(columns, "key");
        postgres::assert_string!(columns, "value");
        postgres::assert_timestamp!(columns, "expires_at", false);
        postgres::assert_timestamp!(columns, "updated_at");
        postgres::assert_timestamp!(columns, "created_at");
    }
}

impl ValidatableTable<&SqliteColumnInfo> for CsmlMemories {
    fn validate(columns: &HashMap<&str, &SqliteColumnInfo>) {
        sqlite::assert_uuid!(columns, "id");
        sqlite::assert_text!(columns, "bot_id");
        sqlite::assert_text!(columns, "channel_id");
        sqlite::assert_text!(columns, "user_id");
        sqlite::assert_text!(columns, "key");
        sqlite::assert_text!(columns, "value");
        sqlite::assert_timestamp!(columns, "expires_at", false);
        sqlite::assert_timestamp!(columns, "updated_at");
        sqlite::assert_timestamp!(columns, "created_at");
    }
}
