use sea_schema::postgres::def::ColumnInfo as PgColumnInfo;
use sea_schema::sqlite::def::ColumnInfo as SqliteColumnInfo;
use std::collections::HashMap;
use test_helpers::util::{TestableTable, ValidatableTable, postgres, sqlite};

pub struct CsmlConversations;

impl TestableTable for CsmlConversations {
    fn name() -> &'static str {
        "csml_conversations"
    }

    fn size() -> usize {
        11
    }
}

impl ValidatableTable<&PgColumnInfo> for CsmlConversations {
    fn validate(columns: &HashMap<&str, &PgColumnInfo>) {
        postgres::assert_uuid!(columns, "id");
        postgres::assert_string!(columns, "bot_id");
        postgres::assert_string!(columns, "channel_id");
        postgres::assert_string!(columns, "user_id");
        postgres::assert_string!(columns, "flow_id");
        postgres::assert_string!(columns, "step_id");
        postgres::assert_string!(columns, "status");
        postgres::assert_timestamp!(columns, "last_interaction_at");
        postgres::assert_timestamp!(columns, "updated_at");
        postgres::assert_timestamp!(columns, "created_at");
        postgres::assert_timestamp!(columns, "expires_at", false);
    }
}

impl ValidatableTable<&SqliteColumnInfo> for CsmlConversations {
    fn validate(columns: &HashMap<&str, &SqliteColumnInfo>) {
        sqlite::assert_uuid!(columns, "id");
        sqlite::assert_text!(columns, "bot_id");
        sqlite::assert_text!(columns, "channel_id");
        sqlite::assert_text!(columns, "user_id");
        sqlite::assert_text!(columns, "flow_id");
        sqlite::assert_text!(columns, "step_id");
        sqlite::assert_text!(columns, "status");
        sqlite::assert_timestamp!(columns, "last_interaction_at");
        sqlite::assert_timestamp!(columns, "updated_at");
        sqlite::assert_timestamp!(columns, "created_at");
        sqlite::assert_timestamp!(columns, "expires_at", false);
    }
}
