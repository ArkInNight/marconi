use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum AppError {
    NotFound,
    NotRunning,
    AccessDenied,
    CaptureFailed,
    Generic(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound => write!(f, "summon history URL not found"),
            AppError::NotRunning => write!(f, "Reverse:1999 is not running"),
            AppError::AccessDenied => write!(f, "insufficient permission to read process memory"),
            AppError::CaptureFailed => write!(f, "failed to capture summon history URL"),
            AppError::Generic(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Generic(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Generic(msg.to_string())
    }
}
