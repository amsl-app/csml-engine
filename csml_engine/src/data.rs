pub mod models;

pub mod filter;
#[cfg(feature = "async")]
pub mod future;

use crate::{
    Client,
    encrypt::{decrypt_data, encrypt_data},
};
use csml_interpreter::data::{Context, CsmlBot, CsmlFlow, Message, Module};

#[cfg(feature = "sea-orm")]
use sea_orm::{ConnectionTrait, TransactionTrait};
#[cfg(any(feature = "postgresql", feature = "sqlite"))]
use serde::de::StdError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

use std::marker::PhantomData;

use uuid::Uuid;

pub const DEBUG: &str = "DEBUG";
pub const DISABLE_SSL_VERIFY: &str = "DISABLE_SSL_VERIFY";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializeCsmlBot {
    pub id: String,
    pub name: String,
    pub flows: Vec<CsmlFlow>,
    pub native_components: Option<String>,
    // serde_json::Map<String, serde_json::Value>
    pub custom_components: Option<String>,
    // serde_json::Value
    pub default_flow: String,
    pub no_interruption_delay: Option<i32>,
    pub env: Option<String>,
    pub modules: Option<Vec<Module>>,
}

#[must_use]
pub fn to_serializable_bot(bot: &CsmlBot) -> SerializeCsmlBot {
    SerializeCsmlBot {
        id: bot.id.clone(),
        name: bot.name.clone(),
        flows: bot.flows.clone(),
        native_components: {
            bot.native_components
                .clone()
                .map(|value| Value::Object(value).to_string())
        },
        custom_components: { bot.custom_components.clone().map(|value| value.to_string()) },
        default_flow: bot.default_flow.clone(),
        no_interruption_delay: bot.no_interruption_delay,
        env: match &bot.env {
            Some(value) => encrypt_data(value).ok(),
            None => None,
        },
        modules: bot.modules.clone(),
    }
}

impl SerializeCsmlBot {
    #[must_use]
    pub fn to_bot(&self) -> CsmlBot {
        CsmlBot {
            id: self.id.clone(),
            name: self.name.clone(),
            apps_endpoint: None,
            flows: self.flows.clone(),
            native_components: self.native_components.as_ref().map(|value| {
                match serde_json::from_str(value.as_str()) {
                    Ok(Value::Object(map)) => map,
                    _ => unreachable!(),
                }
            }),
            custom_components: self.custom_components.as_ref().map(|value| {
                match serde_json::from_str(value.as_str()) {
                    Ok(value) => value,
                    _ => unreachable!(),
                }
            }),
            default_flow: self.default_flow.clone(),
            bot_ast: None,
            no_interruption_delay: self.no_interruption_delay,
            env: self.env.as_ref().and_then(|value| decrypt_data(value).ok()),
            modules: self.modules.clone(),
            multibot: None,
        }
    }
}

pub enum MutConnections<'a, E> {
    Direct(E),
    Reference(&'a mut E),
}

impl<E> AsMut<E> for MutConnections<'_, E> {
    fn as_mut(&mut self) -> &mut E {
        match self {
            MutConnections::Direct(e) => e,
            MutConnections::Reference(e) => e,
        }
    }
}

pub enum Connections<'a, E> {
    Direct(E),
    Reference(&'a E),
}

impl<T> AsRef<T> for Connections<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            Connections::Direct(e) => e,
            Connections::Reference(e) => e,
        }
    }
}

pub enum Database<'a> {
    #[cfg(feature = "postgresql")]
    Postgresql(PostgresqlClient<'a>),
    #[cfg(feature = "sqlite")]
    SqLite(SqliteClient<'a>),
    None(PhantomData<&'a ()>),
}

#[cfg(feature = "postgresql")]
impl<'a> From<&'a mut diesel::PgConnection> for Database<'a> {
    fn from(connection: &'a mut diesel::PgConnection) -> Self {
        Self::Postgresql(PostgresqlClient {
            client: MutConnections::Reference(connection),
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> From<&'a mut diesel::SqliteConnection> for Database<'a> {
    fn from(connection: &'a mut diesel::SqliteConnection) -> Self {
        Self::SqLite(SqliteClient {
            client: MutConnections::Reference(connection),
        })
    }
}

#[cfg(feature = "sea-orm")]
pub trait SeaOrmDbTraits: ConnectionTrait + TransactionTrait + Sync {}

#[cfg(feature = "sea-orm")]
impl<T: ConnectionTrait + TransactionTrait + Sync> SeaOrmDbTraits for T {}

#[cfg(not(feature = "sea-orm"))]
pub trait SeaOrmDbTraits: Sync + Send {}

#[cfg(not(feature = "sea-orm"))]
impl SeaOrmDbTraits for () {}

#[cfg(all(feature = "async", feature = "sea-orm"))]
type SeaOrmDbType = sea_orm::DatabaseConnection;

#[cfg(all(feature = "async", not(feature = "sea-orm")))]
type SeaOrmDbType = ();

#[cfg(feature = "async")]
pub enum AsyncDatabase<'a, T: SeaOrmDbTraits = SeaOrmDbType> {
    #[cfg(feature = "postgresql-async")]
    Postgresql(AsyncPostgresqlClient<'a>),
    #[cfg(feature = "sea-orm")]
    SeaOrm(SeaOrmClient<'a, T>),
    #[cfg(not(feature = "sea-orm"))]
    _Impossible(std::convert::Infallible, PhantomData<T>),
}

#[cfg(feature = "sea-orm")]
impl<'a, T: SeaOrmDbTraits> AsyncDatabase<'a, T> {
    pub fn sea_orm(connection: &'a T) -> Self {
        Self::SeaOrm(SeaOrmClient {
            client: Connections::Reference(connection),
        })
    }
}

#[cfg(feature = "sqlite")]
pub struct SqliteClient<'a> {
    pub client: MutConnections<'a, diesel::prelude::SqliteConnection>,
}

#[cfg(feature = "sqlite")]
impl SqliteClient<'static> {
    #[must_use]
    pub fn new(client: diesel::prelude::SqliteConnection) -> Self {
        Self {
            client: MutConnections::Direct(client),
        }
    }
}

#[cfg(feature = "postgresql")]
pub struct PostgresqlClient<'a> {
    pub client: MutConnections<'a, diesel::prelude::PgConnection>,
}

#[cfg(feature = "postgresql")]
impl PostgresqlClient<'static> {
    #[must_use]
    pub fn new(client: diesel::prelude::PgConnection) -> Self {
        Self {
            client: MutConnections::Direct(client),
        }
    }
}

#[cfg(feature = "sea-orm")]
pub struct SeaOrmClient<'a, T> {
    pub client: Connections<'a, T>,
}

#[cfg(feature = "sea-orm")]
impl<T> SeaOrmClient<'_, T> {
    #[must_use]
    pub fn new(client: T) -> Self {
        Self {
            client: Connections::Direct(client),
        }
    }

    #[must_use]
    pub fn db_ref(&self) -> &T {
        self.client.as_ref()
    }
}

#[cfg(feature = "postgresql-async")]
pub struct AsyncPostgresqlClient<'a> {
    pub client: MutConnections<'a, diesel_async::pg::AsyncPgConnection>,
}

#[cfg(feature = "postgresql-async")]
impl AsyncPostgresqlClient<'static> {
    #[must_use]
    pub fn new(client: diesel_async::pg::AsyncPgConnection) -> Self {
        Self {
            client: MutConnections::Direct(client),
        }
    }
}

pub struct ConversationInfo {
    pub request_id: String,
    pub conversation_id: Uuid,
    pub callback_url: Option<String>,
    pub client: Client,
    pub context: Context,
    pub metadata: Value,
    pub payloads: Vec<Message>,
    pub ttl: Option<chrono::Duration>,
    pub low_data: bool,
}

#[derive(Debug)]
pub enum Next {
    Flow(String),
    Step(String),
    Hold,
    //(i32)
    End,
    Error,
}

#[derive(Debug)]
pub enum EngineError {
    Serde(serde_json::Error),
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    Manager(String),
    Format(String),
    Interpreter(String),
    DateTimeError(String),
    Parring(String),
    Time(std::time::SystemTimeError),
    Internal(String),
    #[cfg(all(feature = "openssl", not(feature = "rustls")))]
    Openssl(openssl::error::ErrorStack),
    #[cfg(feature = "rustls")]
    Encryption(String),
    Base64(base64::DecodeError),
    Uuid(uuid::Error),

    #[cfg(any(feature = "postgresql", feature = "sqlite", feature = "sea-orm"))]
    SqlErrorCode(String),
    #[cfg(any(feature = "postgresql", feature = "sqlite"))]
    SqlMigrationsError(String),
}

impl Error for EngineError {}

impl Display for EngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serde(err) => {
                write!(f, "Serde error: {err}")
            }
            Self::Io(err) => {
                write!(f, "IO error: {err}")
            }
            Self::Utf8(err) => {
                write!(f, "Utf8 error: {err}")
            }
            Self::Manager(err) => {
                write!(f, "Manager error: {err}")
            }
            Self::Format(err) => {
                write!(f, "Format error: {err}")
            }
            Self::Interpreter(err) => {
                write!(f, "Interpreter error: {err}")
            }
            Self::DateTimeError(err) => {
                write!(f, "DateTime error: {err}")
            }
            Self::Parring(err) => {
                write!(f, "Parring error: {err}")
            }
            Self::Time(err) => {
                write!(f, "Time error: {err}")
            }
            Self::Internal(err) => {
                write!(f, "Internal error: {err}")
            }
            #[cfg(all(feature = "openssl", not(feature = "rustls")))]
            Self::Openssl(err) => {
                write!(f, "Openssl error: {err}")
            }
            #[cfg(feature = "rustls")]
            EngineError::Encryption(err) => {
                write!(f, "Encryption error: {}", err)
            }
            Self::Base64(err) => {
                write!(f, "Base64 error: {err}")
            }
            Self::Uuid(err) => {
                write!(f, "Uuid error: {err}")
            }
            #[cfg(any(feature = "postgresql", feature = "sqlite", feature = "sea-orm"))]
            Self::SqlErrorCode(err) => {
                write!(f, "Sql error: {err}")
            }
            #[cfg(any(feature = "postgresql", feature = "sqlite"))]
            Self::SqlMigrationsError(err) => {
                write!(f, "Sql migrations error: {err}")
            }
        }
    }
}

impl From<uuid::Error> for EngineError {
    fn from(value: uuid::Error) -> Self {
        Self::Uuid(value)
    }
}

impl From<serde_json::Error> for EngineError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

impl From<std::io::Error> for EngineError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::str::Utf8Error> for EngineError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8(e)
    }
}

impl From<std::time::SystemTimeError> for EngineError {
    fn from(e: std::time::SystemTimeError) -> Self {
        Self::Time(e)
    }
}

#[cfg(all(feature = "openssl", not(feature = "rustls")))]
impl From<openssl::error::ErrorStack> for EngineError {
    fn from(e: openssl::error::ErrorStack) -> Self {
        Self::Openssl(e)
    }
}

#[cfg(feature = "rustls")]
impl From<aes_gcm::Error> for EngineError {
    fn from(e: aes_gcm::Error) -> Self {
        EngineError::Encryption(e.to_string())
    }
}

impl From<base64::DecodeError> for EngineError {
    fn from(e: base64::DecodeError) -> Self {
        Self::Base64(e)
    }
}

#[cfg(any(feature = "postgresql", feature = "sqlite"))]
impl From<diesel::result::Error> for EngineError {
    fn from(e: diesel::result::Error) -> Self {
        Self::SqlErrorCode(e.to_string())
    }
}

#[cfg(any(feature = "postgresql", feature = "sqlite"))]
impl From<Box<dyn StdError + Send + Sync>> for EngineError {
    fn from(e: Box<dyn StdError + Send + Sync>) -> Self {
        Self::SqlErrorCode(e.to_string())
    }
}

#[cfg(any(feature = "postgresql", feature = "sqlite"))]
impl From<diesel_migrations::MigrationError> for EngineError {
    fn from(e: diesel_migrations::MigrationError) -> Self {
        Self::SqlMigrationsError(e.to_string())
    }
}

#[cfg(feature = "sea-orm")]
impl From<sea_orm::DbErr> for EngineError {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::SqlErrorCode(e.to_string())
    }
}
