use actix_web::{HttpResponse, ResponseError};
use reqwest::StatusCode;
use serde::Serialize;

/// Json struct returned from the API on error
#[derive(Debug, Serialize)]
pub(crate) struct ErrorJson {
    error_chain: Vec<String>,
    #[serde(skip)]
    status_code: StatusCode,
    status_cat: String,
}

impl ErrorJson {
    /// Including a descriptive picture of a cat for each HTTP status code is crucial
    /// because it provides an intuitive visual cue, aiding developers in quickly identifying
    /// and troubleshooting issues, ultimately expediting debugging and maintenance processes.
    fn status_to_cat(status_code: StatusCode) -> String {
        format!("https://http.cat/{}", status_code.as_u16())
    }
    fn from_anyhow(err: &anyhow::Error, status_code: StatusCode) -> ErrorJson {
        log::info!("[trace] ErrorJson::from_anyhow");
        ErrorJson {
            error_chain: err.chain().map(|err| err.to_string()).collect(),
            status_cat: ErrorJson::status_to_cat(status_code),
            status_code,
        }
    }
}

impl From<StatusCode> for ErrorJson {
    fn from(status_code: StatusCode) -> ErrorJson {
        log::info!("[trace] ErrorJson::From<StatusCode>");
        ErrorJson {
            error_chain: Vec::new(),
            status_cat: ErrorJson::status_to_cat(status_code),
            status_code,
        }
    }
}

impl From<&AppError> for ErrorJson {
    fn from(err: &AppError) -> ErrorJson {
        log::info!("[trace] ErrorJson::From<AppError>");
        err.inner.as_ref().map_or_else(
            || err.status_code.into(),
            |anyhow| ErrorJson::from_anyhow(anyhow, err.status_code),
        )
    }
}

impl From<&actix_web::Error> for ErrorJson {
    fn from(err: &actix_web::Error) -> ErrorJson {
        log::info!("[trace] ErrorJson::From<actix_web::Error>");
        if let Some(err) = err.as_error::<AppError>() {
            return err.into();
        }

        let status_code = err.as_response_error().status_code();
        ErrorJson {
            error_chain: vec![err.to_string()],
            status_cat: ErrorJson::status_to_cat(status_code),
            status_code,
        }
    }
}

#[derive(Debug)]
pub(crate) struct AppError {
    status_code: StatusCode,
    inner: Option<anyhow::Error>,
}

/// Error type returned from endpoints
pub(crate) type AppResult<T, E = AppError> = std::result::Result<T, E>;

/// Shorthand methods to return an error with common status codes
macro_rules! impl_into_app_error {
    ($method_name:ident, $status_code:path) => {
        fn $method_name(self) -> AppError {
            self.into_app_error_with_status($status_code)
        }
    };
}

pub(crate) trait IntoAppError: Sized {
    fn into_app_error(self) -> AppError;
    fn into_app_error_with_status(self, status_code: StatusCode) -> AppError;

    impl_into_app_error!(into_app_error_bad_request, StatusCode::BAD_REQUEST);
    impl_into_app_error!(into_app_error_unauthorized, StatusCode::UNAUTHORIZED);
    impl_into_app_error!(
        into_app_error_temorary_redirect,
        StatusCode::TEMPORARY_REDIRECT
    );
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
        log::info!("[trace] into_app_error for anyhow::Error");
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            inner: Some(self),
        }
    }
    fn into_app_error_with_status(self, status_code: StatusCode) -> AppError {
        log::info!("[trace] into_app_error_with_status for anyhow::Error");
        AppError {
            status_code,
            inner: Some(self),
        }
    }
}
impl<T> IntoAppResult<T> for anyhow::Result<T> {
    fn into_app_result(self) -> std::result::Result<T, AppError> {
        log::info!("[trace] into_app_result for anyhow::Result");
        self.map_err(|err| err.into_app_error())
    }

    fn into_app_result_with_status(
        self,
        status_code: StatusCode,
    ) -> std::result::Result<T, AppError> {
        log::info!("[trace] into_app_result_with_status for anyhow::Result");
        self.map_err(|err| err.into_app_error_with_status(status_code))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> AppError {
        log::info!("[trace] AppError::From<anyhow::Error>");
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            inner: Some(err),
        }
    }
}

impl From<StatusCode> for AppError {
    fn from(status_code: StatusCode) -> AppError {
        log::info!("[trace] AppError::From<StatusCode>");
        AppError {
            status_code,
            inner: None,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.as_ref().map_or(Ok(()), |v| v.fmt(f))
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        log::info!("[trace] AppError::error_response");
        ErrorJson::from(self).error_response()
    }
}

impl std::fmt::Display for ErrorJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "status code: {}, ", self.status_code.as_u16())?;
        write!(f, "errors: {:?}", self.error_chain)
    }
}

impl ResponseError for ErrorJson {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        log::info!("[trace] ErrorJson::error_response");
        HttpResponse::build(self.status_code).json(self)
    }
}
