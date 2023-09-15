mod api;
mod error;
mod openid;
mod util;

use actix_web::{web, App, HttpServer};
use anyhow::Context;
use openid::Provider;

const SOCKET: &str = "127.0.0.1:8080";

const STEAM_OPENID_LOGIN: &str = "https://steamcommunity.com/openid";
const REALM: &str = "http://localhost:8080";
const RETURN_TO: &str = "http://localhost:8080/api/auth/steam/callback";

struct SteamState {
    provider: Provider,
    auth_url: String,
}
impl SteamState {
    pub async fn new(client: &reqwest::Client) -> anyhow::Result<SteamState> {
        use openid::make_auth_req_url;

        let resp = client.get(STEAM_OPENID_LOGIN).send().await;
        let resp = resp.context("couldn't fetch steam openid service")?;

        let xml = resp
            .text()
            .await
            .context("couldn't read response body as text")?;

        let provider = openid::parse_xml(&xml).context("couldn't parse response xml as service")?;

        let auth_url = make_auth_req_url(&provider, REALM, RETURN_TO)
            .context("couldn't create auth request url")?;

        Ok(SteamState { provider, auth_url })
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
    util::init_logger().context("couldn't initialize logger")?;
    log::info!("initialized logger");

    let state = State::new().await.context("couldn't create app state")?;
    let data = web::Data::new(state);
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(error::error_handler())
            .app_data(web::Data::clone(&data))
            .service(api::init())
    });

    server = server
        .bind(SOCKET)
        .with_context(|| format!("couldn't bind to socket `{}`", SOCKET))?;
    log::info!("server is listening on {}", SOCKET);

    server
        .workers(1)
        .run()
        .await
        .context("error while running server")?;

    Ok(())
}
