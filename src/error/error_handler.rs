use actix_web::dev::ServiceResponse;
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{dev, ResponseError, Result};

use crate::error::error_json::ErrorJson;
use crate::error::AppError;

#[inline]
fn is_generated_from<E, B>(res: &dev::ServiceResponse<B>) -> bool
where
    E: actix_web::error::ResponseError + 'static,
{
    res.response()
        .error()
        .and_then(|err| err.as_error::<E>())
        .is_some()
}

/// If the error handler returns an `Err(InnerError)`, it must implement [`ResponseError`] by
/// the constraints on an error handler. The [`ResponseError::error_response`] method is then
/// invoked on the `InnerError` and that is returned.
#[allow(clippy::unnecessary_wraps)]
fn to_json_error<B: 'static>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    // Sanity check, this should never happend because API endpoints should
    // always return `AppError` and never directly `ErrorJson`.
    //
    // And we don't even expose ErrorJson, so this really shouldn't happen.
    if is_generated_from::<ErrorJson, _>(&res) {
        log::error!("Never return an ErrorJson, return an AppError instead!");
        // map_into_left_body means return the already generated response
        return Ok(ErrorHandlerResponse::Response(res.map_into_left_body()));
    }

    // App error is already good to go.
    if is_generated_from::<AppError, _>(&res) {
        err_trace!("Error cause is an app error, let it through 😏");
        // map_into_left_body means return the already generated response
        return Ok(ErrorHandlerResponse::Response(res.map_into_left_body()));
    };

    // Destructuring here is needed because we borrow res through err in the next line
    let (req, res) = res.into_parts();

    // If it's just a status-code without an error attached, we still need to do one more thing...
    let Some(err) = res.error() else {
        err_trace!("No error cause, just a status code... Add the cat! 🤡");
        let err_json_response = ErrorJson::from_status_code(res.status()).error_response();

        // map_into_right_body means return this newly generated response
        return Ok(ErrorHandlerResponse::Response(
            ServiceResponse::new(req, err_json_response).map_into_right_body(),
        ));
    };

    err_trace!("Error cause is some other error, convert it! 😈");
    let err_json_response = ErrorJson::from_actix_error(err).error_response();

    // map_into_right_body means return this newly generated response
    Ok(ErrorHandlerResponse::Response(
        ServiceResponse::new(req, err_json_response).map_into_right_body(),
    ))
}

pub(crate) fn error_handler<B: 'static>() -> ErrorHandlers<B> {
    ErrorHandlers::<B>::new().default_handler(to_json_error)
}
