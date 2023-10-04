use anyhow::Context;
use serde::{Deserialize, Serialize};
use steam_api_concurrent::SteamId;

use crate::util::nonce::Nonce;
use crate::State;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub(crate) enum SteamAuthState {
    Redirected { nonce: Nonce },
    Authenticated { id: SteamId },
}

pub(crate) trait AuthSession {
    fn steam_auth_state(&self) -> anyhow::Result<Option<SteamAuthState>>;
    fn redirected(&self) -> Option<Nonce>;
    fn replace_session(&self, state: &State) -> anyhow::Result<Nonce>;
    fn authenticated(&self) -> Option<SteamId>;
    fn validate_replace_nonce(&self, state: &State, old: &str) -> anyhow::Result<Nonce>;
    fn insert_new_nonce(&self, state: &State) -> anyhow::Result<Nonce>;
    fn authenticate(&self, steam_id: SteamId) -> anyhow::Result<()>;
    fn logout(&self) -> anyhow::Result<SteamId>;
}

// TODO: Clean this up
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
    fn replace_session(&self, state: &State) -> anyhow::Result<Nonce> {
        let nonces = &state.steam.nonces;
        let nonce = nonces.insert_new();
        let state = SteamAuthState::Redirected {
            nonce: nonce.clone(),
        };
        self.insert("steam-auth-state", state)
            .context("couldn't serialize nonce to json")?;
        Ok(nonce)
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
