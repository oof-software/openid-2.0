use actix_web::{HttpResponse, ResponseError};
use reqwest::StatusCode;

use crate::error::error_json::ErrorJson;

#[derive(Debug)]
pub(crate) struct AppError {
    pub(super) status_code: StatusCode,
    pub(super) inner: anyhow::Error,
}

/// Error type returned from endpoints
pub(crate) type AppResult<T, E = AppError> = std::result::Result<T, E>;

/// Response returned from Endpoints
pub(crate) type AppResponse<E = AppError> = AppResult<HttpResponse, E>;

/// Shorthand methods to return an error with common status codes
macro_rules! impl_into_app_error {
    ($method_name:ident, $status_code:path) => {
        fn $method_name(self) -> AppError {
            self.into_app_error_with_status($status_code)
        }
    };
}

/// Easy conversion into an AppError
///
/// Currently only implemented for [`anyhow::Error`]
pub(crate) trait IntoAppError: Sized {
    fn into_app_error_with_status(self, status_code: StatusCode) -> AppError;

    impl_into_app_error!(into_app_error_im_a_teapot, StatusCode::IM_A_TEAPOT);
    impl_into_app_error!(into_app_error_bad_request, StatusCode::BAD_REQUEST);
    impl_into_app_error!(into_app_error_unauthorized, StatusCode::UNAUTHORIZED);
    impl_into_app_error!(
        into_app_error_temorary_redirect,
        StatusCode::TEMPORARY_REDIRECT
    );
}

impl IntoAppError for anyhow::Error {
    fn into_app_error_with_status(self, status_code: StatusCode) -> AppError {
        err_trace!("IntoAppError for anyhow::Error");
        AppError {
            status_code,
            inner: self,
        }
    }
}

/// If no status code is given, use INTERNAL_SERVER_ERROR
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> AppError {
        err_trace!("AppError::From<anyhow::Error>");
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            inner: err,
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

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        err_trace!("AppError::error_response");
        ErrorJson::from_app_error(self).error_response()
    }
}
