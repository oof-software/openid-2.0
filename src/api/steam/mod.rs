use actix_web::web;

mod player_bans;
mod player_summaries;
mod steam_level;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(player_bans::configure)
        .configure(steam_level::configure)
        .configure(player_summaries::configure);
}
