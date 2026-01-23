#[cfg(feature = "postgresql-async")]
use crate::future::db_connectors::postgresql_connector;

#[cfg(feature = "sea-orm")]
use crate::future::db_connectors::sea_orm_connector;

use crate::data::{AsyncDatabase, EngineError, SeaOrmDbTraits};

use crate::db_connectors::utils::get_expires_at;

use csml_interpreter::data::Client;

pub async fn delete_state_key<T: SeaOrmDbTraits>(
    client: &Client,
    type_: &str,
    key: &str,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(?client, r#type = %type_, %key, "db call delete state key");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::state::delete_state_key(client, type_, key, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::state::delete_state_key(client, type_, key, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn get_state_key<T: SeaOrmDbTraits>(
    client: &Client,
    type_: &str,
    key: &str,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<Option<serde_json::Value>, EngineError> {
    tracing::debug!(?client, r#type = %type_, %key, "db call get state key");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::state::get_state_key(client, type_, key, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::state::get_state_key(client, type_, key, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn set_state_items<T: SeaOrmDbTraits>(
    client: &Client,
    type_: &str,
    keys_values: Vec<(&str, &serde_json::Value)>,
    ttl: Option<chrono::Duration>,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(?client, r#type = %type_, ?keys_values, "db call set state items");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::state::set_state_items(
                client,
                type_,
                keys_values,
                get_expires_at(ttl),
                db,
            )
            .await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::state::set_state_items(
                client,
                type_,
                keys_values,
                get_expires_at(ttl),
                db.db_ref(),
            )
            .await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

#[cfg(all(test, feature = "_dbtest"))]
mod tests {
    use super::*;
    use crate::db_connectors::{make_migrations, revert_migrations};
    use crate::future::db_connectors::init_db;
    use core::panic;
    use csml_interpreter::data::{Hold, IndexInfo};
    use serial_test::serial;
    use test_log::test;

    #[test(tokio::test)]
    #[serial(postgres)]
    async fn ok_hold() {
        make_migrations().unwrap();
        let client = Client {
            bot_id: "bot_id".to_owned(),
            channel_id: "channel_id".to_owned(),
            user_id: "test".to_owned(),
        };
        let mut db = init_db().await.unwrap();

        let hash = "Hash".to_owned();
        let index_info = Hold {
            index: IndexInfo {
                command_index: 42,
                loop_index: vec![],
            },
            step_vars: serde_json::json!({}),
            step_name: "step_name".to_owned(),
            flow_name: "flow_name".to_owned(),
            previous: None,
            secure: false,
        };

        let state_hold: serde_json::Value = serde_json::json!({
            "index": index_info.index,
            "step_vars": index_info.step_vars,
            "hash": hash
        });

        set_state_items(
            &client,
            "hold",
            vec![("position", &state_hold)],
            None,
            &mut db,
        )
        .await
        .unwrap();

        let hold = get_state_key(&client, "hold", "position", &mut db)
            .await
            .unwrap()
            .unwrap();

        let Ok(index_result) = serde_json::from_value::<IndexInfo>(hold["index"].clone()) else {
            panic!("value not found in db");
        };

        if index_result.loop_index != index_info.index.loop_index
            && index_result.command_index != index_info.index.command_index
        {
            panic!("db get hold got the wrong value")
        }

        delete_state_key(&client, "hold", "position", &mut db)
            .await
            .unwrap();

        if get_state_key(&client, "hold", "position", &mut db)
            .await
            .unwrap()
            .is_some()
        {
            panic!("get_state_key should not have found a hold because it has deleted just before")
        }

        revert_migrations().unwrap();
    }
}
