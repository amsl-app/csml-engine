#[cfg(feature = "postgresql-async")]
use crate::future::db_connectors::postgresql_connector;

#[cfg(feature = "sea-orm")]
use crate::future::db_connectors::sea_orm_connector;

use crate::data::{AsyncDatabase, EngineError, SeaOrmDbTraits};

pub async fn delete_expired_data<T: SeaOrmDbTraits>(
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::expired_data::delete_expired_data(db).await?;
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::expired_data::delete_expired_data(db.db_ref()).await?;
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
    Ok(())
}
