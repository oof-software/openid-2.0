use actix_web::{web, HttpResponse};

#[actix_web::get("/live")]
pub(crate) async fn health_live() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("LIVE"))
}

#[actix_web::get("/ready")]
pub(crate) async fn health_ready() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("READY"))
}

pub(crate) fn init() -> actix_web::Scope {
    web::scope("/health")
        .service(health_live)
        .service(health_ready)
}
