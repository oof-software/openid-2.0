use actix_web::{HttpResponse, ResponseError};
use reqwest::StatusCode;
use serde::Serialize;

#[derive(Debug)]
pub(crate) struct AppError {
    status_code: StatusCode,
    inner: anyhow::Error,
}

pub(crate) type AppResult<T, E = AppError> = std::result::Result<T, E>;

pub(crate) trait IntoAppError: Sized {
    fn into_app_error(self) -> AppError;
    fn into_app_error_with_status(self, status_code: StatusCode) -> AppError;
}
pub(crate) trait IntoAppResult<T>: Sized {
    fn into_app_result(self) -> std::result::Result<T, AppError>;
    fn into_app_result_with_status(
        self,
        status_code: StatusCode,
    ) -> std::result::Result<T, AppError>;
}

impl IntoAppError for anyhow::Error {
    fn into_app_error(self) -> AppError {
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            inner: self,
        }
    }
    fn into_app_error_with_status(self, status_code: StatusCode) -> AppError {
        AppError {
            status_code,
            inner: self,
        }
    }
}
impl<T> IntoAppResult<T> for anyhow::Result<T> {
    fn into_app_result(self) -> std::result::Result<T, AppError> {
        self.map_err(|err| err.into_app_error())
    }

    fn into_app_result_with_status(
        self,
        status_code: StatusCode,
    ) -> std::result::Result<T, AppError> {
        self.map_err(|err| err.into_app_error_with_status(status_code))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            inner: value,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        #[derive(Serialize)]
        struct ErrorJson {
            error_chain: Vec<String>,
            status_code: String,
        }
        impl ErrorJson {
            fn new(err: &anyhow::Error, status_code: StatusCode) -> ErrorJson {
                ErrorJson {
                    error_chain: err.chain().map(|err| err.to_string()).collect(),
                    status_code: format!("https://http.cat/{}", status_code.as_u16()),
                }
            }
        }

        let error_json = ErrorJson::new(&self.inner, self.status_code);
        HttpResponse::build(self.status_code).json(&error_json)
    }
}
