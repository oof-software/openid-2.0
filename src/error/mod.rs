//! <https://github.com/actix/actix-web/blob/master/actix-web/MIGRATION-4.0.md#error-handlers>
//!
//! How error handling works
//!
//! TODO: Fix this
//!
//! 1) The endpoint hits the wall and returns `Err(...)`
//! 2) `Responder::respond_to` for `Result<R,E>` is invoked
//! 3) `HttpResponse::from_error` calls `error_response` on the `Error`
//!    and sets the underlying error to the error that caused the whole thing
//! 4) `AppError::error_response` is called if an `AppError` was the cause
//! 5) An error handler is selected from the list of registered error handlers and invoked
//!   1) If no error handler was found, the in (4) generated error response is returned
//!   2) If the error handler returns `Err(...)`, that implements
//! 6)
//!
//! # How it works
//!
//! ## `Err(AppError)`
//!
//! ```text
//! return Err(AppError)
//!   -> AppError::error_response
//!   -> ErrorJson::from_app_error
//!   -> ErrorJson::error_response
//!   -> to_json_error (error handler)
//!   -> Check the root cause -> AppError -> Return the generated response
//! ```
//!
//! ## Something Else
//!
//! ```text
//! Some Actix internal stuff
//!   -> to_json_error (error handler)
//!   -> Check the root casue -> Not an AppError -> Convert it
//!   ->
//! ```
//!

/// This macro is here to keep track of error conversion.
macro_rules! err_trace {
    ($arg:tt) => ({
        #[cfg(feature = "err-trace")]
        {
            ::log::info!(::std::concat!("[err-trace] ", $arg));
        }
    });
    ($arg:tt, $($args:tt)+) => ({
        #[cfg(feature = "err-trace")]
        {
            ::log::info!(::std::concat!("[err-trace] ", $arg), $($args)+);
        }
    });
}

mod app_error;
mod error_handler;
mod error_json;

pub(crate) use app_error::{AppError, AppResponse, AppResult, IntoAppError};
pub(crate) use error_handler::error_handler;
