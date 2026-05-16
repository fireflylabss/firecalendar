use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Event not found: {0}")]
    EventNotFound(String),

    #[error("Category not found: {0}")]
    CategoryNotFound(String),

    #[error("Event store not found. Run 'firecalendar init' first.")]
    StoreNotFound,

    #[error("Store already exists at {0}")]
    StoreAlreadyExists(String),

    #[error("Invalid event data: {0}")]
    InvalidEventData(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Notification error: {0}")]
    NotificationError(String),
}

pub type Result<T> = std::result::Result<T, Error>;