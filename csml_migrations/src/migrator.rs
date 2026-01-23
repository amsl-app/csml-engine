use crate::{DynIden, m20231019_000001_create_table};
use itertools::Itertools;
use sea_orm::prelude::*;
use sea_orm::sea_query::{Alias, IntoIden};
use sea_orm::{Statement, TransactionError, TransactionTrait, TryGetableMany};
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use sqlx::types::chrono;
use std::error::Error;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20231019_000001_create_table::Migration)]
    }

    fn migration_table_name() -> DynIden {
        Alias::new("csml_seaql_migrations").into_iden()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DieselMigrationState {
    // Migration was unnecessary
    Skipped,
    // Migration was applied
    Applied,
}

impl Migrator {
    pub async fn migrate_from_diesel<C: ConnectionTrait + TransactionTrait>(
        conn: &C,
    ) -> Result<DieselMigrationState, DbErr> {
        let backend = conn.get_database_backend();
        #[allow(clippy::match_wildcard_for_single_variants)]
        let versions: &[&str] = match backend {
            sea_orm::DatabaseBackend::Postgres => &[
                "00000000000000",
                "20210830130425",
                "20210909130425",
                "20231020122950",
            ],
            sea_orm::DatabaseBackend::Sqlite => &["20210830130425"],
            _ => {
                unimplemented!("Unsupported database backend")
            }
        };
        let migration_table = Self::migration_table_name().to_string();
        let applied_migrations = Self::get_applied_migrations(conn).await?;
        if !applied_migrations.is_empty() {
            tracing::info!("some sea orm migrations already applied, skipping diesel migration");
            return Ok(DieselMigrationState::Skipped);
        }
        let res = conn
            .transaction(|txn| {
                Box::pin(async move {
                    let res = txn
                        .query_all(Statement::from_string(
                            backend,
                            format!("SELECT * FROM __diesel_schema_migrations WHERE version in ({}) ORDER BY run_on DESC", versions.iter().map(|version| format!("'{version}'")).join(", ")),
                        ))
                        .await;
                    let res = match res {
                        Ok(res) => res,
                        Err(err) => {
                            #[allow(clippy::match_wildcard_for_single_variants)]
                            match backend {
                                sea_orm::DatabaseBackend::Postgres => {
                                    if let DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(err))) = &err {
                                        let code = err.code();
                                        if let Some(code) = code {
                                            // The Table does not exist, which is Ok
                                            if code == "42P01" {
                                                tracing::info!("__diesel_schema_migrations table does not exist, skipping diesel migration");
                                                return Ok(DieselMigrationState::Skipped);
                                            }
                                        }
                                    }
                                }
                                sea_orm::DatabaseBackend::Sqlite => {
                                    if let DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(err))) = &err
                                        && err.message() == "no such table: __diesel_schema_migrations" {
                                            tracing::info!("__diesel_schema_migrations table does not exist, skipping diesel migration");
                                            return Ok(DieselMigrationState::Skipped);
                                        }
                                }
                                _ => {}
                            }

                            tracing::error!(error = &err as &dyn Error, "error while checking diesel migration state");
                            return Err(err);
                        }
                    };
                    if res.is_empty() {
                        tracing::info!("migrations not found in __diesel_schema_migrations. skipping diesel migration");
                        return Ok(DieselMigrationState::Skipped);
                    }
                    if res.len() != versions.len() {
                        return Err(DbErr::Custom(format!("Migrations found in __diesel_schema_migrations do not match expected versions. Found {} of {} migrations", res.len(), versions.len())));
                    }
                    // Unwrap is safe because we checked the length
                    let res= res.first().unwrap();
                    let (_, applied_at) =
                        <(String, chrono::NaiveDateTime)>::try_get_many(res, "", &[String::from("version"), String::from("run_on")])?;
                    let res = txn
                        .execute(Statement::from_string(
                            backend,
                            format!("DELETE FROM __diesel_schema_migrations WHERE version in ({})", versions.iter().map(|version| format!("'{version}'")).join(", ")),
                        ))
                        .await?;
                    if res.rows_affected() != u64::try_from(versions.len()).map_err(|_| DbErr::Custom("Failed to convert u32 to u64".to_string()))? {
                        return Err(DbErr::RecordNotFound(
                            "Migration could not be deleted because it could not be found".to_string(),
                        ));
                    }
                    let applied_at = applied_at.and_utc().timestamp();
                    txn.execute(Statement::from_sql_and_values(
                        backend,
                        format!("INSERT INTO {migration_table} (version, applied_at) VALUES ($1, $2)"),
                        vec!["m20231019_000001_create_table".into(), applied_at.into()],
                    ))
                    .await?;
                    tracing::info!("migrated diesel migration {versions:?} to sea orm");
                    Ok(DieselMigrationState::Applied)
                })
            })
            .await;
        match res {
            Ok(state) => Ok(state),
            Err(TransactionError::Transaction(err) | TransactionError::Connection(err)) => Err(err),
        }
    }
}
