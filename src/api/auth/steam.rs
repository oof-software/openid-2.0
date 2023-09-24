use std::borrow::Cow;
use std::str::FromStr;

use actix_web::{http, web, HttpResponse};
use anyhow::Context;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use steam_api_concurrent::api::PlayerSummary;
use steam_api_concurrent::SteamId;

use crate::error::{AppResponse, AppResult, IntoAppError};
use crate::openid::{
    verify_against_provider, PositiveAssertion, VerifyResponse, STEAM_IDENTITY_PREFIX,
};
use crate::util::nonce::Nonce;
use crate::State;

#[derive(Serialize, Deserialize, Debug)]
enum SteamAuthState {
    Redirected { nonce: Nonce },
    Authenticated { id: SteamId },
}

trait AuthSession {
    fn steam_auth_state(&self) -> anyhow::Result<Option<SteamAuthState>>;
    fn redirected(&self) -> Option<Nonce>;
    fn authenticated(&self) -> Option<SteamId>;
    fn validate_replace_nonce(&self, state: &State, old: &str) -> anyhow::Result<Nonce>;
    fn insert_new_nonce(&self, state: &State) -> anyhow::Result<Nonce>;
    fn authenticate(&self, steam_id: SteamId) -> anyhow::Result<()>;
    fn logout(&self) -> anyhow::Result<SteamId>;
}

impl AuthSession for actix_session::Session {
    fn authenticated(&self) -> Option<SteamId> {
        let state = self.steam_auth_state().ok().flatten()?;
        match state {
            SteamAuthState::Redirected { .. } => None,
            SteamAuthState::Authenticated { id } => Some(id),
        }
    }
    fn redirected(&self) -> Option<Nonce> {
        let state = self.steam_auth_state().ok().flatten()?;
        match state {
            SteamAuthState::Redirected { nonce } => Some(nonce),
            SteamAuthState::Authenticated { .. } => None,
        }
    }
    fn logout(&self) -> anyhow::Result<SteamId> {
        let id = self.authenticated().context("not logged in")?;
        self.clear();
        Ok(id)
    }
    fn validate_replace_nonce(&self, state: &State, old: &str) -> anyhow::Result<Nonce> {
        let nonces = &state.steam.nonces;
        let nonce = nonces.replace(old).context("couldn't replace old nonce")?;
        let state = SteamAuthState::Redirected {
            nonce: nonce.clone(),
        };
        self.insert("steam-auth-state", state)
            .context("couldn't serialize nonce to json")?;
        Ok(nonce)
    }
    fn insert_new_nonce(&self, state: &State) -> anyhow::Result<Nonce> {
        let nonces = &state.steam.nonces;
        let nonce = nonces.insert_new();
        let state = SteamAuthState::Redirected {
            nonce: nonce.clone(),
        };
        self.insert("steam-auth-state", state)
            .context("couldn't serialize nonce to json")?;
        Ok(nonce)
    }
    fn steam_auth_state(&self) -> anyhow::Result<Option<SteamAuthState>> {
        self.get::<SteamAuthState>("steam-auth-state")
            .context("couldn't deserialize steam-auth-state")
    }
    fn authenticate(&self, steam_id: SteamId) -> anyhow::Result<()> {
        let state = SteamAuthState::Authenticated { id: steam_id };
        self.insert("steam-auth-state", state)
            .context("couldn't serialize steam id to json")
    }
}

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
            // the user is already authenticated, send him back to the home page
            return Ok(HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
                .insert_header((http::header::LOCATION.as_str(), "/api/health/cookies"))
                .finish());
        }
        None => {
            // The user should visit the login page first
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
