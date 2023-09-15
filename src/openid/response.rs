//! Example
//! ```json
//! {
//!   "openid.mode": "id_res"
//!   "openid.ns": "http://specs.openid.net/auth/2.0",
//!   "openid.op_endpoint": "https://steamcommunity.com/openid/login",
//!   "openid.claimed_id": "https://steamcommunity.com/openid/id/<STEAMID>",
//!   "openid.identity": "https://steamcommunity.com/openid/id/<STEAMID>",
//!   "openid.return_to": "http://localhost:8080/api/auth/steam/callback",
//!   "openid.response_nonce": "<REDACTED>",
//!   "openid.assoc_handle": "<REDACTED>",
//!   "openid.signed": "signed,op_endpoint,claimed_id,identity,return_to,response_nonce,assoc_handle",
//!   "openid.sig": "<REDACTED>",
//! }
//! ```
