use std::borrow::Cow;

use actix_web::{web, HttpResponse};
use anyhow::Context;
use serde::Deserialize;
use steam_api_concurrent::SteamId;

use crate::api::session::AuthSession;
use crate::error::AppResponse;
use crate::openid::comma_separated::CommaSeparated;
use crate::State;

#[derive(Deserialize)]
pub(crate) struct Query {
    steam_ids: CommaSeparated<SteamId>,
}

pub(crate) async fn player_summaries(
    session: actix_session::Session,
    data: web::Data<State>,
    query: web::Query<Query>,
) -> AppResponse {
    if session.authenticated().is_none() {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let steam_ids = query.into_inner().steam_ids.into_inner();
    if steam_ids.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let steam_ids = Cow::Owned(steam_ids);
    let resp = data.steam.api.get_player_summaries(steam_ids).await;
    let resp = resp.context("couldn't fetch from steam api")?;

    Ok(HttpResponse::Ok().json(resp.into_inner()))
}

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/player-summaries").route(web::get().to(player_summaries)));
}
