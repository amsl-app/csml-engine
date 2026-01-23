use sea_schema::postgres::def::ColumnInfo as PgColumnInfo;
use sea_schema::sqlite::def::ColumnInfo as SqliteColumnInfo;
use std::collections::HashMap;
use test_helpers::util::{TestableTable, ValidatableTable, postgres, sqlite};

pub struct CsmlMessages;

impl TestableTable for CsmlMessages {
    fn name() -> &'static str {
        "csml_messages"
    }

    fn size() -> usize {
        12
    }
}

impl ValidatableTable<&PgColumnInfo> for CsmlMessages {
    fn validate(columns: &HashMap<&str, &PgColumnInfo>) {
        postgres::assert_uuid!(columns, "id");
        postgres::assert_uuid!(columns, "conversation_id");
        postgres::assert_string!(columns, "flow_id");
        postgres::assert_string!(columns, "step_id");
        postgres::assert_string!(columns, "direction");
        postgres::assert_string!(columns, "payload");
        postgres::assert_string!(columns, "content_type");
        postgres::assert_integer!(columns, "message_order");
        postgres::assert_integer!(columns, "interaction_order");
        postgres::assert_timestamp!(columns, "updated_at");
        postgres::assert_timestamp!(columns, "created_at");
        postgres::assert_timestamp!(columns, "expires_at", false);
    }
}

impl ValidatableTable<&SqliteColumnInfo> for CsmlMessages {
    fn validate(columns: &HashMap<&str, &SqliteColumnInfo>) {
        sqlite::assert_uuid!(columns, "id");
        sqlite::assert_uuid!(columns, "conversation_id");
        sqlite::assert_text!(columns, "flow_id");
        sqlite::assert_text!(columns, "step_id");
        sqlite::assert_text!(columns, "direction");
        sqlite::assert_text!(columns, "payload");
        sqlite::assert_text!(columns, "content_type");
        sqlite::assert_integer!(columns, "message_order");
        sqlite::assert_integer!(columns, "interaction_order");
        sqlite::assert_timestamp!(columns, "updated_at");
        sqlite::assert_timestamp!(columns, "created_at");
        sqlite::assert_timestamp!(columns, "expires_at", false);
    }
}
