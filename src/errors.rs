use actix_web::{error::ResponseError, HttpResponse};

pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("SQL failed: {0:?}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Any error: {0:?}")]
    Anyhow(#[from] anyhow::Error),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::SqlxError(err) => HttpResponse::InternalServerError().json(err.to_string()),
            Self::Anyhow(err) => HttpResponse::InternalServerError().json(err.to_string()),
        }
    }
}
