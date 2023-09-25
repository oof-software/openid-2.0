use actix_web::web;

mod player_bans;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(player_bans::configure);
}
