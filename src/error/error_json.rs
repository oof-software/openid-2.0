use actix_web::{HttpResponse, ResponseError};
use reqwest::StatusCode;
use serde::Serialize;

use crate::error::AppError;

/// Json struct returned from the API on error
#[derive(Debug, Serialize)]
pub(super) struct ErrorJson {
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

    /// This is not implemented as a trait because it should not be exposed.
    pub(super) fn from_anyhow(err: &anyhow::Error, status_code: StatusCode) -> ErrorJson {
        log::info!("[err-trace] ErrorJson::from_anyhow");
        ErrorJson {
            error_chain: err.chain().map(|err| err.to_string()).collect(),
            status_cat: ErrorJson::status_to_cat(status_code),
            status_code,
        }
    }

    /// This is not implemented as a trait because it should not be exposed.
    pub(super) fn from_status_code(status_code: StatusCode) -> ErrorJson {
        log::info!("[err-trace] ErrorJson::from_status_code");
        ErrorJson {
            error_chain: Vec::new(),
            status_cat: ErrorJson::status_to_cat(status_code),
            status_code,
        }
    }

    /// This is not implemented as a trait because it should not be exposed.
    pub(super) fn from_app_error(err: &AppError) -> ErrorJson {
        log::info!("[err-trace] ErrorJson::from_app_error");
        err.inner.as_ref().map_or_else(
            || ErrorJson::from_status_code(err.status_code),
            |anyhow| ErrorJson::from_anyhow(anyhow, err.status_code),
        )
    }

    /// This is not implemented as a trait because it should not be exposed.
    fn from_actix_error(err: &actix_web::Error) -> ErrorJson {
        if let Some(err) = err.as_error::<AppError>() {
            return ErrorJson::from_app_error(err);
        }

        let status_code = err.as_response_error().status_code();
        ErrorJson {
            error_chain: vec![err.to_string()],
            status_cat: ErrorJson::status_to_cat(status_code),
            status_code,
        }
    }

    /// This is not implemented as a trait because it should not be exposed.
    ///
    /// `pub(crate)` because it is needed in [`crate::error::error_handler`]
    ///
    /// This consumes the [`actix_web::HttpResponse`] because it replaces it.
    #[inline]
    pub(crate) fn from_response<B>(res: HttpResponse<B>) -> ErrorJson {
        res.error().map_or_else(
            || ErrorJson::from_status_code(res.status()),
            ErrorJson::from_actix_error,
        )
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
        log::info!("[err-trace] ErrorJson::error_response");
        HttpResponse::build(self.status_code).json(self)
    }
}
