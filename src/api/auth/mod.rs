mod never;
mod steam;

use actix_web::web;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/steam").configure(steam::configure))
        .service(web::scope("/never").configure(never::configure));
}
