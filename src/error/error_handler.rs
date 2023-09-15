use actix_web::{dev, error, middleware};

pub(crate) fn error_handler<B: 'static>() -> middleware::ErrorHandlers<B> {
    middleware::ErrorHandlers::default().default_handler(error_handler_inner)
}

fn error_handler_inner<B>(
    res: dev::ServiceResponse<B>,
) -> actix_web::Result<middleware::ErrorHandlerResponse<B>> {
    // https://github.com/actix/actix-web/discussions/2597#discussioncomment-2028389
    // https://docs.rs/actix-web/latest/actix_web/body/enum.EitherBody.html
    //
    // the inner service’s response body type maps to the `Left` variant
    // the middleware’s own error responses uses the `Right` variant

    if let Some(err) = res.response().error() {
        if let Some(err) = err.as_error::<error::InternalError<anyhow::Error>>() {
            let err_msg = format!("{err:?}");
            log::warn!("{}", err_msg.replace("\n\n", "\n"));
        } else {
            log::warn!("{err}");
        }
    }

    let status = res.status();
    let req = res.into_parts().0;

    // omit the body and only return the status code
    let res = actix_web::HttpResponseBuilder::new(status).finish();

    Ok(middleware::ErrorHandlerResponse::Response(
        dev::ServiceResponse::new(req, res).map_into_right_body(),
    ))
}
