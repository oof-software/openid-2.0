use actix_web::dev::ServiceResponse;
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{dev, ResponseError, Result};

use super::{AppError, ErrorJson};

#[allow(clippy::unnecessary_wraps)]
fn to_json_error<B: 'static>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let (req, res) = res.into_parts();

    // App error is already good to go.
    let app_error = res.error().and_then(|err| err.as_error::<AppError>());
    if app_error.is_some() {
        log::info!("[trace] to_json_error: it's an app error, let it through");
        return Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
            req,
            res.map_into_left_body(),
        )));
    }

    // Sanity check, this should never happend because API endpoints should
    // always return `AppError` and never directly `ErrorJson`.
    let json_error = res.error().and_then(|err| err.as_error::<ErrorJson>());
    if json_error.is_some() {
        log::error!("[trace] to_json_error: it's already an json error?");
    }

    log::info!("[trace] to_json_error: it's some other error");
    let err_json: ErrorJson = res
        .error()
        .map_or_else(|| res.status().into(), |err| err.into());

    // Break the middleware chain and return our custom json response
    Ok(ErrorHandlerResponse::Response(
        ServiceResponse::new(req, err_json.error_response()).map_into_right_body(),
    ))
}

pub(crate) fn error_handler<B: 'static>() -> ErrorHandlers<B> {
    ErrorHandlers::<B>::new().default_handler(to_json_error)
}
