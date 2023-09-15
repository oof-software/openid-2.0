mod steam;

use actix_web::web;

pub(crate) fn init() -> actix_web::Scope {
    web::scope("/auth").service(steam::init())
}
