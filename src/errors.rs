use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Couldn't convert {0} to enum")]
    FromStrError(String),
}