#![forbid(unsafe_code)]
#![allow(dead_code)]
#![warn(
    clippy::copy_iterator,
    clippy::default_trait_access,
    clippy::doc_link_with_quotes,
    clippy::enum_glob_use,
    clippy::expl_impl_clone_on_copy,
    clippy::implicit_clone,
    clippy::inconsistent_struct_constructor,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::items_after_statements,
    clippy::iter_not_returning_iterator,
    clippy::large_digit_groups,
    clippy::large_futures,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::redundant_else,
    clippy::semicolon_if_nothing_returned,
    clippy::unnecessary_box_returns,
    clippy::unnecessary_join,
    clippy::unnecessary_wraps,
    clippy::unnested_or_patterns,
    clippy::unused_async,
    clippy::used_underscore_binding
)]
#![warn(clippy::wildcard_dependencies)]
#![warn(
    clippy::branches_sharing_code,
    clippy::clear_with_drain,
    clippy::cognitive_complexity,
    clippy::collection_is_never_read,
    clippy::debug_assert_with_mut_call,
    clippy::derive_partial_eq_without_eq,
    clippy::empty_line_after_doc_comments,
    clippy::empty_line_after_outer_attr,
    clippy::equatable_if_let,
    clippy::fallible_impl_from,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_with_drain,
    clippy::large_stack_frames,
    clippy::manual_clamp,
    clippy::missing_const_for_fn,
    clippy::mutex_integer,
    clippy::needless_collect,
    clippy::nonstandard_macro_braces,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::path_buf_push_overwrite,
    clippy::redundant_clone,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening,
    clippy::suspicious_operation_groupings,
    clippy::trait_duplication_in_bounds,
    clippy::type_repetition_in_bounds,
    clippy::unnecessary_struct_initialization,
    clippy::unused_rounding,
    clippy::useless_let_if_seq
)]

mod api;
mod error;
mod openid;
mod openid_next;
mod util;

use actix_web::{web, App, HttpServer};
use anyhow::Context;
use openid::{make_auth_req_url, Provider};
use util::nonce::NonceSet;

use crate::error::error_handler;

const SOCKET: &str = "127.0.0.1:8080";

const STEAM_OPENID_LOGIN: &str = "https://steamcommunity.com/openid";
const REALM: &str = "http://localhost:8080";
const RETURN_TO: &str = "http://localhost:8080/api/auth/steam/callback";
const LOGIN: &str = "http://localhost:8080/api/auth/steam/login";

struct SteamState {
    provider: Provider,
    auth_url: String,
    nonces: NonceSet,
}
impl SteamState {
    pub(crate) async fn new(client: &reqwest::Client) -> anyhow::Result<SteamState> {
        let resp = client.get(STEAM_OPENID_LOGIN).send().await;
        let resp = resp.context("couldn't fetch steam openid service")?;

        let xml = resp
            .text()
            .await
            .context("couldn't read response body as text")?;

        let provider =
            Provider::from_xml(&xml).context("couldn't parse response xml as service")?;

        let auth_url = make_auth_req_url(&provider, REALM, RETURN_TO)
            .context("couldn't create auth request url")?;

        let nonces = NonceSet::new();

        Ok(SteamState {
            provider,
            auth_url,
            nonces,
        })
    }
    pub(crate) fn auth_url_with_nonce(&self, nonce: &str) -> anyhow::Result<String> {
        let return_to = reqwest::Url::parse_with_params(RETURN_TO, [("custom_nonce", nonce)])
            .context("couldn't parse return_to url with custom nonce")?;
        let auth_url = make_auth_req_url(&self.provider, REALM, return_to.as_str())
            .context("couldn't create auth request url with custom nonce")?;
        Ok(auth_url)
    }
}

struct State {
    client: reqwest::Client,
    steam: SteamState,
}
impl State {
    pub async fn new() -> anyhow::Result<State> {
        let client = reqwest::Client::builder()
            .https_only(true)
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .context("couldn't build reqwest client")?;
        let steam = SteamState::new(&client)
            .await
            .context("couldn't create steam state")?;

        Ok(State { client, steam })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    util::log::init_logger().context("couldn't initialize logger")?;
    log::info!("initialized logger");

    let state = State::new().await.context("couldn't create app state")?;
    let data = web::Data::new(state);
    log::info!("created app state");

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&data))
            .wrap(error_handler())
            .service(web::scope("/api").configure(api::configure))
    });

    server = server
        .bind(SOCKET)
        .with_context(|| format!("couldn't bind to socket `{}`", SOCKET))?;

    log::info!("server is listening on {}", SOCKET);
    log::info!("visit {} and get you will get redirected to steam", LOGIN);
    log::info!("after authorization you get redirected to {}", RETURN_TO);

    server
        .workers(1)
        .run()
        .await
        .context("error while running server")?;

    Ok(())
}
