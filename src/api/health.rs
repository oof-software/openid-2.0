use actix_web::{web, HttpResponse};
use reqwest::StatusCode;

use crate::error::{AppResult, IntoAppError};

#[actix_web::get("/live")]
pub(crate) async fn health_live() -> AppResult<HttpResponse> {
    Ok(HttpResponse::Ok().body("LIVE"))
}

#[actix_web::get("/ready")]
pub(crate) async fn health_ready() -> AppResult<HttpResponse> {
    Ok(HttpResponse::Ok().body("READY"))
}

/// Provide an example for an error response
#[actix_web::get("/error")]
pub(crate) async fn health_error() -> AppResult<HttpResponse> {
    Err(anyhow::anyhow!("stubbed toe 😖")
        .context("lost focus 😵")
        .context("fell over 🤾🏽‍♀️")
        .context("hit the floor 🤕")
        .into_app_error_with_status(StatusCode::IM_A_TEAPOT))
}

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health_live)
        .service(health_ready)
        .service(health_error);
}
