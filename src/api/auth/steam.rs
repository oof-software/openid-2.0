use actix_web::{http, web, HttpResponse};
use anyhow::Context;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::error::{AppResult, IntoAppError};
use crate::openid::{verify_against_provider, PositiveAssertion, VerifyResponse};
use crate::State;

#[actix_web::get("/login")]
pub(crate) async fn start_steam_auth(data: web::Data<State>) -> AppResult<HttpResponse> {
    let nonce = data.steam.nonces.insert_new();

    let url = data
        .steam
        .auth_url_with_nonce(nonce.as_str())
        .context("couldn't create auth url with nonce")?;

    // Could just use redirect but here we can see how redirects work.
    // Just pray, that this is actually correct (●'◡'●)
    Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
        .insert_header((http::header::LOCATION.as_str(), url))
        .finish())
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CallbackQuery {
    /// We append this nonce to the auth request in [`start_steam_auth`]
    /// to [`PositiveAssertion::return_to`] and as per spec it must be preserved.
    custom_nonce: String,
    /// Regular fields expected when callback is called
    #[serde(flatten)]
    assertion: PositiveAssertion,
}

#[derive(Serialize)]
struct CallbackResponse<'a> {
    response: &'a VerifyResponse,
    custom_nonce: &'a str,
    assertion: &'a PositiveAssertion,
}

async fn validate_positive_assertion(
    assertion: &PositiveAssertion,
    state: &State,
) -> anyhow::Result<VerifyResponse> {
    assertion
        .validate(&state.steam.provider)
        .context("invalid positive assertion (generic)")?;
    assertion
        .validate_steam()
        .context("invalid positive assertion (steam)")?;

    let validation_result =
        verify_against_provider(&state.client, &state.steam.provider, &assertion)
            .await
            .context("couldn't verify assertion against provider")?;

    Ok(validation_result)
}

#[actix_web::get("/callback")]
pub(crate) async fn return_steam_auth(
    data: web::Data<State>,
    query: web::Query<CallbackQuery>,
) -> AppResult<HttpResponse> {
    let nonces = &data.steam.nonces;

    nonces
        .validate_and_remove(&query.custom_nonce)
        .context("couldn't validate the supplied nonce")
        .map_err(|err| err.into_app_error_bad_request())?;

    let validation_result = validate_positive_assertion(&query.assertion, &data)
        .await
        .map_err(|err| err.into_app_error_bad_request())?;

    let response = CallbackResponse {
        response: &validation_result,
        custom_nonce: &query.custom_nonce,
        assertion: &query.assertion,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub(crate) fn init() -> actix_web::Scope {
    web::scope("/steam")
        .service(start_steam_auth)
        .service(return_steam_auth)
}
