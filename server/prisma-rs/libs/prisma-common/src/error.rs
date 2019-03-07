use std::error::Error as StdError;

type Cause = Box<dyn StdError>;

#[derive(Debug)]
pub enum Error {
    /// Error forming a connection. Check the connection options and network
    /// status.
    ConnectionError(&'static str, Option<Cause>),
    /// Error querying the database. Check the query format and parameters.
    QueryError(&'static str, Option<Cause>),
    /// Couldn't read the protobuf from Scala
    ProtobufDecodeError(&'static str, Option<Cause>),
    /// Couldn't read the JSON from Scala
    JsonDecodeError(&'static str, Option<Cause>),
    /// Input from Scala was not good
    InvalidInputError(String),
    /// Invalid connection arguments, e.g. first and last were both defined in a query
    InvalidConnectionArguments(&'static str),
    /// No result returned from query
    NoResultError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

impl Error {
    fn fetch_cause(opt_cause: &Option<Cause>) -> Option<&(dyn std::error::Error + 'static)> {
        opt_cause.as_ref().map(|cause| &**cause)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::ConnectionError(message, _) => message,
            Error::QueryError(message, _) => message,
            Error::ProtobufDecodeError(message, _) => message,
            Error::JsonDecodeError(message, _) => message,
            Error::InvalidInputError(message) => message,
            Error::InvalidConnectionArguments(message) => message,
            Error::NoResultError => "Query returned no results",
        }
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ConnectionError(_, cause) => Self::fetch_cause(&cause),
            Error::QueryError(_, cause) => Self::fetch_cause(&cause),
            Error::ProtobufDecodeError(_, cause) => Self::fetch_cause(&cause),
            Error::JsonDecodeError(_, cause) => Self::fetch_cause(&cause),
            _ => None,
        }
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Error {
        Error::ConnectionError("Error creating database connection", Some(Box::new(e)))
    }
}

impl From<sqlite::Error> for Error {
    fn from(e: sqlite::Error) -> Error {
        // TODO(katharina): Properly handle sqlite errors
        // This can't currently be done because the sqlite
        // crate doesn't properly expose these errors as variants.
        Error::QueryError("Error querying SQLite database", Some(Box::new(e)))
    }
}

impl From<prost::DecodeError> for Error {
    fn from(e: prost::DecodeError) -> Error {
        Error::ProtobufDecodeError("Error decoding protobuf message", Some(Box::new(e)))
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Error {
        Error::JsonDecodeError("Error decoding JSON message", Some(Box::new(e)))
    }
}
