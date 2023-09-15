use actix_web::{http, web, HttpResponse};
use anyhow::Context;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::error::{AppResult, IntoAppError};
use crate::openid::{verify_against_provider, PositiveAssertion};
use crate::State;

#[actix_web::get("/login")]
pub(crate) async fn start_steam_auth(data: web::Data<State>) -> AppResult<HttpResponse> {
    let nonce = data.steam.nonces.insert_new().await;

    let url = data
        .steam
        .auth_url_with_nonce(&nonce)
        .context("couldn't create auth url with nonce")?;

    // Could just use redirect but here we can see how redirects work.
    // Just pray, that this is actually correct (●'◡'●)
    Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
        .insert_header((http::header::LOCATION.as_str(), url))
        .finish())
}

#[derive(Debug, Deserialize)]
pub(crate) struct CallbackQuery {
    /// We append this nonce to the auth request in [`start_steam_auth`]
    /// to [`PositiveAssertion::return_to`] and as per spec it must be preserved.
    custom_nonce: String,
    /// Regular fields expected when callback is called
    #[serde(flatten)]
    openid: PositiveAssertion,
}

#[actix_web::get("/callback")]
pub(crate) async fn return_steam_auth(
    data: web::Data<State>,
    query: web::Query<CallbackQuery>,
) -> AppResult<HttpResponse> {
    let nonces = &data.steam.nonces;
    let provider = &data.steam.provider;

    query
        .openid
        .validate(provider)
        .context("couldn't validate generic positive assertion")?;
    query
        .openid
        .validate_steam()
        .context("couldn't validate steam positive assertion")?;

    if !nonces.validate_and_remove(&query.custom_nonce).await {
        return Err(anyhow::anyhow!("invalid custom nonce")
            .into_app_error_with_status(StatusCode::BAD_REQUEST));
    }

    let result = verify_against_provider(&data.client, &data.steam.provider, &query.openid)
        .await
        .context("couldn't verify assertion against provider")?;

    use std::fmt::Write;
    let mut body = String::new();
    writeln!(&mut body, "Query: {:#?}", query.0).unwrap();
    writeln!(&mut body, "Response: {:#?}", result).unwrap();

    Ok(HttpResponse::Ok().content_type("text/plain").body(body))
}

pub(crate) fn init() -> actix_web::Scope {
    web::scope("/steam")
        .service(start_steam_auth)
        .service(return_steam_auth)
}
