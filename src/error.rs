use std::fmt;

#[derive(Debug)]
pub enum AppError {
    AccessDenied,
    Generic(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::AccessDenied => {
                write!(f, "insufficient permission to read process memory")
            }
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
