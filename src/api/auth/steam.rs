use std::collections::HashMap;

use actix_web::{web, HttpResponse, Responder};

use crate::State;

#[actix_web::get("/login")]
pub(crate) async fn start_steam_auth(data: web::Data<State>) -> impl Responder {
    web::Redirect::to(data.steam.auth_url.clone())
}

#[actix_web::get("/callback")]
pub(crate) async fn return_steam_auth(
    query: web::Query<HashMap<String, String>>,
) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(&query.0))
}

pub(crate) fn init() -> actix_web::Scope {
    web::scope("/steam")
        .service(start_steam_auth)
        .service(return_steam_auth)
}
