use actix_web::{web, HttpResponse};

use super::session::AuthSession;
use crate::error::{AppResult, IntoAppError};

pub(crate) async fn health_live() -> AppResult<HttpResponse> {
    Ok(HttpResponse::Ok().body("LIVE"))
}

pub(crate) async fn health_ready() -> AppResult<HttpResponse> {
    Ok(HttpResponse::Ok().body("READY"))
}

/// Provide an example for an error response
pub(crate) async fn health_error() -> AppResult<HttpResponse> {
    Err(anyhow::anyhow!("stubbed toe 😖")
        .context("lost focus 😵")
        .context("fell over 🤾🏽‍♀️")
        .context("hit the floor 🤕")
        .into_app_error_im_a_teapot())
}

/// Let the user view the encrypted cookies
pub(crate) async fn health_cookies(session: actix_session::Session) -> AppResult<HttpResponse> {
    let auth_state = session.steam_auth_state()?;
    Ok(HttpResponse::Ok().json(&auth_state))
}

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/live").route(web::get().to(health_live)))
        .service(web::resource("/ready").route(web::get().to(health_ready)))
        .service(web::resource("/error").route(web::get().to(health_error)))
        .service(web::resource("/cookies").route(web::get().to(health_cookies)));
}
