use serde::{Deserialize, Serialize};

#[derive(
    PartialEq, Copy, Clone, Debug, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub enum LogLvl {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
