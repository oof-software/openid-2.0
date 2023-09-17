use actix_web::web;

mod auth;
mod health;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::configure))
        .service(web::scope("/health").configure(health::configure));
}
