use actix_web::{web, HttpResponse};
use anyhow::Context;
use serde::Deserialize;
use steam_api_concurrent::SteamId;

use crate::api::session::AuthSession;
use crate::error::AppResponse;
use crate::State;

#[derive(Deserialize)]
pub(crate) struct Query {
    steam_id: SteamId,
}

pub(crate) async fn steam_level(
    session: actix_session::Session,
    data: web::Data<State>,
    query: web::Query<Query>,
) -> AppResponse {
    if session.authenticated().is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let resp = data.steam.api.get_player_steam_level(query.steam_id).await;
    let resp = resp.context("couldn't fetch from steam api")?;

    Ok(HttpResponse::Ok().json(resp.into_inner()))
}

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/steam-level").route(web::get().to(steam_level)));
}
