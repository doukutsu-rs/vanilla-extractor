//! Error types and conversion functions.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// An enum containing all kinds of game framework errors.
#[derive(Debug, Clone)]
pub enum GameError {
    /// An error in the filesystem layout
    FilesystemError(String),
    /// An error trying to load a resource, such as getting an invalid image file.
    ResourceLoadError(String),
    /// Unable to find a resource; the `Vec` is the paths it searched for and associated errors
    ResourceNotFound(String, Vec<(std::path::PathBuf, GameError)>),
    /// Something went wrong trying to read from a file
    IOError(Arc<std::io::Error>),
    /// Something went wrong while parsing something.
    ParseError(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameError::ResourceLoadError(ref s) => write!(f, "Error loading resource: {}", s),
            GameError::ResourceNotFound(ref s, ref paths) => {
                write!(f, "Resource not found: {}, searched in paths {:?}", s, paths)
            }
            _ => write!(f, "GameError {:?}", self),
        }
    }
}

impl Error for GameError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            GameError::IOError(e) => Some(e as &dyn Error),
            _ => None,
        }
    }
}

/// A convenient result type consisting of a return type and a `GameError`
pub type GameResult<T = ()> = Result<T, GameError>;

impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> GameError {
        GameError::IOError(Arc::new(e))
    }
}
