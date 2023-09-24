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

use actix_session::config::CookieContentSecurity;
use actix_session::storage::{CookieSessionStore, RedisActorSessionStore};
use actix_session::SessionMiddleware;
use actix_web::cookie::{self, Key, SameSite};
use actix_web::{middleware, web, App, HttpServer};
use anyhow::Context;
use openid::{make_auth_req_url, Provider};
use util::nonce::NonceSet;

use crate::error::error_handler;

const SOCKET: &str = "127.0.0.1:8080";

const STEAM_OPENID_LOGIN: &str = "https://steamcommunity.com/openid";
const REALM: &str = "http://localhost:8080";
const RETURN_TO: &str = "http://localhost:8080/api/auth/steam/callback";

struct SteamState {
    provider: Provider,
    nonces: NonceSet,
    api: steam_api_concurrent::Client,
}
impl SteamState {
    pub(crate) async fn new(client: &reqwest::Client) -> anyhow::Result<SteamState> {
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();
        let api = steam_api_concurrent::ClientOptions::new()
            .api_key(api_key)
            .build()
            .await
            .context("couldn't prepare steam api client")?;

        let resp = client.get(STEAM_OPENID_LOGIN).send().await;
        let resp = resp.context("couldn't fetch steam openid service")?;

        let xml = resp
            .text()
            .await
            .context("couldn't read response body as text")?;

        let provider =
            Provider::from_xml(&xml).context("couldn't parse response xml as service")?;

        let nonces = NonceSet::new();

        Ok(SteamState {
            provider,
            nonces,
            api,
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

fn load_cookie_key() -> anyhow::Result<cookie::Key> {
    use base64::engine::general_purpose::STANDARD as Base64;
    use base64::Engine;

    let key_b64 =
        dotenv::var("COOKIE_KEY_BASE64").context("missing COOKIE_KEY_BASE64 env variable")?;

    let mut key_data = vec![0u8; 128];
    let len = Base64
        .decode_slice(key_b64, &mut key_data)
        .context("couldn't decode COOKIE_KEY_BASE64")?;

    if len != 64 {
        anyhow::bail!("key in COOKIE_KEY_BASE64 is too small ({} < {})", 64, len);
    }

    cookie::Key::try_from(&key_data[..64])
        .context("couldn't construct cookie key from COOKIE_KEY_BASE64 data")
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

fn create_redis_session_mw(url: &str, key: Key) -> SessionMiddleware<RedisActorSessionStore> {
    SessionMiddleware::builder(RedisActorSessionStore::new(url), key)
        .cookie_http_only(false)
        .cookie_same_site(SameSite::Lax)
        .cookie_name("session-id".to_string())
        .cookie_content_security(CookieContentSecurity::Private)
        .build()
}

fn _create_cookie_session_mw(key: Key) -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), key)
        .cookie_http_only(false)
        .cookie_same_site(SameSite::Lax)
        .cookie_name("session-data".to_string())
        .cookie_content_security(CookieContentSecurity::Private)
        .build()
}

fn create_logger_mw() -> middleware::Logger {
    middleware::Logger::new(r#"%Ts %bB %{r}a [%r -> %s] "%{Referer}i" "%{User-Agent}i""#)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv().context("load .env environment")?;

    util::log::init_logger().context("couldn't initialize logger")?;
    log::info!("initialized logger");

    let cookie_key = load_cookie_key().context("couldn't load cookie key")?;
    let state = State::new().await.context("couldn't create app state")?;
    let data = web::Data::new(state);
    log::info!("created app state");

    let redis_url = dotenv::var("REDIS_URL").context("load REDIS_URL env variable")?;

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&data))
            .wrap(create_logger_mw())
            .wrap(error_handler())
            .wrap(create_redis_session_mw(&redis_url, cookie_key.clone()))
            .service(web::scope("/api").configure(api::configure))
    });

    server = server
        .bind(SOCKET)
        .with_context(|| format!("couldn't bind to socket `{}`", SOCKET))?;

    log::info!("server is listening on {}", SOCKET);

    log::info!("here is a list of endpoints:");
    for (endpoint, description) in [
        ("/api/auth/steam/login", "initiate login to steam"),
        ("/api/auth/steam/callback", "verify assertion from steam"),
        ("/api/auth/steam/logout", "logout from steam"),
        ("/api/auth/never/login", "initiate login to never"),
        ("/api/health/live", "health check"),
        ("/api/health/ready", "health check"),
        ("/api/health/error", "error example"),
        ("/api/health/cookies", "view cookies decrypted"),
    ] {
        log::info!("- http://{}{}: {}", SOCKET, endpoint, description);
    }

    server
        .workers(1)
        .run()
        .await
        .context("error while running server")?;

    Ok(())
}
