use actix_web::web;

mod auth;
mod health;

pub(crate) fn init() -> actix_web::Scope {
    web::scope("/api")
        .service(auth::init())
        .service(health::init())
}
