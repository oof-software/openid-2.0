use std::borrow::Cow;
use std::str::FromStr;

use actix_web::{http, web, HttpResponse};
use anyhow::Context;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use steam_api_concurrent::api::PlayerSummary;
use steam_api_concurrent::SteamId;

use crate::api::session::{AuthSession, SteamAuthState};
use crate::error::{AppResponse, AppResult, IntoAppError};
use crate::openid::{
    verify_against_provider, PositiveAssertion, VerifyResponse, STEAM_IDENTITY_PREFIX,
};
use crate::State;

/// Initiate OpenID 2.0 authentication with Steam
pub(crate) async fn start_steam_auth(
    session: actix_session::Session,
    data: web::Data<State>,
) -> AppResponse {
    let state = session.steam_auth_state()?;

    let nonce = match state.as_ref() {
        Some(SteamAuthState::Redirected { nonce }) => {
            // the user should've been redirected to steam and not be on this page
            // give him a new nonce, remove the old one and move on.
            session
                .validate_replace_nonce(&data, nonce.as_str())
                .context("couldn't refresh nonce")?
        }
        Some(SteamAuthState::Authenticated { .. }) => {
            // the user is already authenticated, send him back to the home page
            return Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
                .insert_header((http::header::LOCATION.as_str(), "/api/health/cookies"))
                .finish());
        }
        None => {
            // the expected case, the user visists this page for the first time
            session
                .insert_new_nonce(&data)
                .context("couldn't create nonce")?
        }
    };

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

pub(crate) async fn logout_steam_auth(session: actix_session::Session) -> AppResult<HttpResponse> {
    session.logout().context("couldn't logout")?;
    Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
        .insert_header((http::header::LOCATION.as_str(), "/api/health/cookies"))
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

#[derive(Debug, Serialize)]
struct CallbackResponse<'a> {
    response: &'a VerifyResponse,
    custom_nonce: &'a str,
    assertion: &'a PositiveAssertion,
    profile: Option<&'a PlayerSummary>,
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
        verify_against_provider(&state.client, &state.steam.provider, assertion)
            .await
            .context("couldn't verify assertion against provider")?;

    Ok(validation_result)
}

/// Process a possible OpenID 2.0 Positive Assertion
/// after the user has granted **authentication**.
pub(crate) async fn return_steam_auth(
    session: actix_session::Session,
    data: web::Data<State>,
    query: web::Query<CallbackQuery>,
) -> AppResponse {
    let state = session.steam_auth_state()?;

    let state_nonce = match state.as_ref() {
        Some(SteamAuthState::Redirected { nonce }) => {
            // we expect to see this nonce in the return_to for the open id response
            // and in the query parameters.
            nonce
        }
        Some(SteamAuthState::Authenticated { .. }) => {
            // the user is already authenticated...?
            return Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
                .insert_header((http::header::LOCATION.as_str(), "/api/health/cookies"))
                .finish());
        }
        None => {
            // the user should visit the login page first
            return Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
                .insert_header((http::header::LOCATION.as_str(), "/api/auth/steam/login"))
                .finish());
        }
    };

    // check that the nonces in the query parameters and in the cookie state match
    if query.custom_nonce != state_nonce.as_str() {
        return Err(
            anyhow::anyhow!("query param nonce doesn't match state nonce",)
                .into_app_error_bad_request(),
        );
    }

    // validate and remove the nonce as it is now used
    let nonces = &data.steam.nonces;
    nonces
        .validate_and_remove(&query.custom_nonce)
        .context("couldn't validate the supplied nonce")
        .map_err(|err| err.into_app_error_bad_request())?;

    // extract the steam id from the positive asstion from steam
    let steam_id_str = query
        .assertion
        .claimed_id()
        .strip_prefix(STEAM_IDENTITY_PREFIX)
        .context("assertion claimed id has invalid prefix")
        .map_err(|err| err.into_app_error_bad_request())?;
    let steam_id = SteamId::from_str(steam_id_str)
        .context("couldn't parse steam id")
        .map_err(|err| err.into_app_error_bad_request())?;

    // make another request to validate the positive assertion
    //
    // without this, another user could spoof a valid
    // openid endpoint and impersonate other users!
    let validation_result = validate_positive_assertion(&query.assertion, &data)
        .await
        .map_err(|err| err.into_app_error_bad_request())?;

    // the positive assertion was not genuine but has been forged
    if !validation_result.is_valid() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    // everything has been checked, the user is good to go!
    session
        .authenticate(steam_id)
        .context("couldn't update session to authenticate")?;

    // TODO: This should be exposed by other endpoints and then
    //       they should be called by the frontent.
    let steam_api = &data.steam.api;
    let resp = steam_api
        .get_player_summaries(Cow::from(&[steam_id][..]))
        .await;

    if let Err(err) = resp.as_ref() {
        log::warn!("couldn't fetch steam profile for {}: {:?}", steam_id, err);
    }

    let profile = resp.as_ref().ok().and_then(|map| map.get(&steam_id));
    let response = CallbackResponse {
        response: &validation_result,
        custom_nonce: &query.custom_nonce,
        assertion: &query.assertion,
        profile,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/callback").route(web::get().to(return_steam_auth)))
        .service(web::resource("/login").route(web::get().to(start_steam_auth)))
        .service(web::resource("/logout").route(web::get().to(logout_steam_auth)));
}
