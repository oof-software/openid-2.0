use actix_web::web;

mod auth;
mod health;
mod session;
mod steam;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::configure))
        .service(web::scope("/health").configure(health::configure))
        .service(web::scope("/steam").configure(steam::configure));
}
