pub use sea_orm_migration::prelude::*;

mod m20231019_000001_create_table;

#[cfg(test)]
mod final_state;
pub mod migrator;

pub use migrator::Migrator;

pub trait MigratableTable {
    fn create() -> TableCreateStatement;
    #[must_use]
    fn drop() -> TableDropStatement {
        let table = Self::create();
        Table::drop()
            .table(table.get_table_name().unwrap().clone())
            .if_exists()
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrator::DieselMigrationState;
    use sea_orm::{Database, Statement, TryGetableMany};
    use serial_test::serial;
    use sqlx::{Postgres, Sqlite};
    use std::collections::HashMap;
    use test_helpers::util::build_test_set;
    use test_helpers::{PostgresqlDb, SqliteDb, TestDb};
    use test_log::test;

    build_test_set!(
        final_state::CsmlBotVersions,
        final_state::CsmlConversations,
        final_state::CsmlMessages,
        final_state::CsmlMemories,
        final_state::CsmlStates
    );

    #[test(tokio::test)]
    async fn test_sqlite() {
        let db = SqliteDb::new().unwrap();
        let options = sqlx::pool::PoolOptions::<Sqlite>::new();
        let db_uri = db.db_uri();
        let db_uri = db_uri.as_ref();
        tracing::info!(db_uri, "connecting to db");
        let conn = Database::connect(db_uri).await.unwrap();
        let pool = options.connect(db_uri).await.unwrap();
        // Count other tables because they might be created by other tests.
        // E.g., diesel migrations.
        let other_table_count = sea_schema::sqlite::discovery::SchemaDiscovery::new(pool.clone())
            .discover()
            .await
            .unwrap()
            .tables
            .iter()
            .filter(|t| {
                t.name.as_str()
                    != format!(
                        "select * from {}",
                        Migrator::migration_table_name().to_string()
                    )
            })
            .count();

        let ts: i64 = TryFrom::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
        .unwrap();

        Migrator::up(&conn, None).await.unwrap();
        let schema = sea_schema::sqlite::discovery::SchemaDiscovery::new(pool.clone())
            .discover()
            .await
            .unwrap();
        let tables: HashMap<_, _> = schema
            .tables
            .iter()
            .map(|tdef| (tdef.name.as_str(), tdef))
            .collect();
        assert_eq!(tables.len(), COUNT + other_table_count + 1);

        let res = conn
            .query_all(Statement::from_string(
                conn.get_database_backend(),
                format!(
                    "select * from {}",
                    Migrator::migration_table_name().to_string()
                ),
            ))
            .await
            .unwrap();
        assert_eq!(res.len(), 1);

        let (version, applied_at) = <(String, i64)>::try_get_many(
            res.first().unwrap(),
            "",
            &["version".to_owned(), "applied_at".to_owned()],
        )
        .unwrap();
        assert!(applied_at >= ts);
        assert_eq!(version, "m20231019_000001_create_table");

        validate_sqlite(&tables);

        Migrator::down(&conn, None).await.unwrap();

        let schema = sea_schema::sqlite::discovery::SchemaDiscovery::new(pool)
            .discover()
            .await
            .unwrap();
        assert_eq!(schema.tables.len(), other_table_count + 1);
    }

    #[test(tokio::test)]
    #[serial(postgres)]
    async fn test_postgres() {
        let db = PostgresqlDb::new().await.unwrap();
        let options = sqlx::pool::PoolOptions::<Postgres>::new();
        let db_uri = db.db_uri();
        let db_uri = db_uri.as_ref();

        tracing::info!(db_uri, "connecting to db");
        let conn = Database::connect(db_uri).await.unwrap();
        let pool = options.connect(db_uri).await.unwrap();
        // Count other tables because they might be created by other tests.
        // E.g., diesel migrations.
        let other_table_count =
            sea_schema::postgres::discovery::SchemaDiscovery::new(pool.clone(), "public")
                .discover()
                .await
                .unwrap()
                .tables
                .iter()
                .filter(|t| {
                    t.info.name.as_str()
                        != format!(
                            "select * from {}",
                            Migrator::migration_table_name().to_string()
                        )
                })
                .count();

        let ts: i64 = TryFrom::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
        .unwrap();

        Migrator::up(&conn, None).await.unwrap();
        let schema = sea_schema::postgres::discovery::SchemaDiscovery::new(pool.clone(), "public")
            .discover()
            .await
            .unwrap();
        let tables: HashMap<_, _> = schema
            .tables
            .iter()
            .map(|tdef| (tdef.info.name.as_str(), tdef))
            .collect();
        assert_eq!(tables.len(), COUNT + other_table_count + 1);

        let res = conn
            .query_all(Statement::from_string(
                conn.get_database_backend(),
                format!(
                    "select * from {}",
                    Migrator::migration_table_name().to_string()
                ),
            ))
            .await
            .unwrap();
        assert_eq!(res.len(), 1);

        let (version, applied_at) = <(String, i64)>::try_get_many(
            res.first().unwrap(),
            "",
            &["version".to_owned(), "applied_at".to_owned()],
        )
        .unwrap();
        assert!(applied_at >= ts);
        assert_eq!(version, "m20231019_000001_create_table");

        validate_pg(&tables);

        Migrator::down(&conn, None).await.unwrap();

        let schema = sea_schema::postgres::discovery::SchemaDiscovery::new(pool, "public")
            .discover()
            .await
            .unwrap();
        assert_eq!(schema.tables.len(), other_table_count + 1);
    }

    #[test(tokio::test)]
    async fn test_sqlite_diesel_migration() {
        let db = SqliteDb::new().unwrap();
        let conn = Database::connect(db.db_uri()).await.unwrap();

        conn.execute_unprepared("DROP TABLE IF EXISTS __diesel_schema_migrations;")
            .await
            .unwrap();
        conn.execute_unprepared(
            r"
create table __diesel_schema_migrations
(
    version VARCHAR(50) not null primary key,
    run_on TIMESTAMP default CURRENT_TIMESTAMP not null
);
",
        )
        .await
        .unwrap();
        conn.execute_unprepared(
            "insert into __diesel_schema_migrations (version) values ('20210830130425');",
        )
        .await
        .unwrap();

        let state = Migrator::migrate_from_diesel(&conn).await.unwrap();
        assert_eq!(state, DieselMigrationState::Applied);
        conn.execute_unprepared("DROP TABLE __diesel_schema_migrations;")
            .await
            .unwrap();
        conn.execute_unprepared(&format!(
            "DROP TABLE {};",
            Migrator::migration_table_name().to_string()
        ))
        .await
        .unwrap();
    }

    #[test(tokio::test)]
    #[serial(postgres)]
    async fn test_postgres_diesel_migration() {
        let db = PostgresqlDb::new().await.unwrap();
        let conn = Database::connect(db.db_uri()).await.unwrap();
        conn.execute_unprepared("DROP TABLE IF EXISTS __diesel_schema_migrations;")
            .await
            .unwrap();
        conn.execute_unprepared(
            r"
create table __diesel_schema_migrations
(
    version varchar(50) not null primary key,
    run_on timestamp default CURRENT_TIMESTAMP not null
);
",
        )
        .await
        .unwrap();
        conn.execute_unprepared("insert into __diesel_schema_migrations (version) values ('00000000000000'), ('20210830130425'), ('20210909130425'), ('20231020122950');")
            .await
            .unwrap();

        let state = Migrator::migrate_from_diesel(&conn).await.unwrap();
        assert_eq!(state, DieselMigrationState::Applied);
        conn.execute_unprepared("DROP TABLE __diesel_schema_migrations;")
            .await
            .unwrap();
        conn.execute_unprepared(&format!(
            "DROP TABLE {};",
            Migrator::migration_table_name().to_string()
        ))
        .await
        .unwrap();
    }
}
