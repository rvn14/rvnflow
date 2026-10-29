use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("Unable to save or read your data: {0}")]
    Io(#[from] std::io::Error),
    #[error("The saved data could not be read: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Native(String),
}
impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
pub type Result<T> = std::result::Result<T, AppError>;
pub fn invalid(message: impl Into<String>) -> AppError {
    AppError::Validation(message.into())
}
