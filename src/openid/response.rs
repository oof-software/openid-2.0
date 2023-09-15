//! Example params
//! ```json
//! {
//!   "openid.ns": "http://specs.openid.net/auth/2.0",
//!   "openid.mode": "id_res",
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

use std::str::FromStr;

use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::openid::constants::{
    OPENID_ASSOCIATION_HANDLE, OPENID_CLAIMED_ID, OPENID_FIELD_PREFIX, OPENID_IDENTITY,
    OPENID_OP_ENDPOINT, OPENID_RESPONSE_NONCE, OPENID_RETURN_TO,
};

use super::{
    constants::{
        OPENID_AUTH_NAMESPACE, OPENID_MODE_IDENTIFIER_RESPONSE, OPENID_RESPONSE_NONCE_MAX_LEN,
    },
    Provider,
};

#[derive(Debug, Clone)]
pub(crate) struct Nonce {
    pub(crate) time: DateTime<Utc>,
    pub(crate) salt: String,
}
impl FromStr for Nonce {
    type Err = anyhow::Error;
    fn from_str(nonce: &str) -> Result<Self, Self::Err> {
        if nonce.len() > OPENID_RESPONSE_NONCE_MAX_LEN {
            anyhow::bail!("response nonce is too long");
        }

        let last_time_char = nonce.find('Z').context("nonce doesn't adhere to spec")?;
        let (time, salt) = nonce.split_at(last_time_char + 1);

        let salt = salt.to_string();
        let time: DateTime<Utc> = DateTime::from(
            DateTime::parse_from_rfc3339(time).context("couldn't parse date and time of nonce")?,
        );

        Ok(Nonce { time, salt })
    }
}
impl Nonce {
    pub(crate) fn age(&self, now: Option<&DateTime<Utc>>) -> Duration {
        match now {
            None => Utc::now().signed_duration_since(self.time),
            Some(now) => now.signed_duration_since(self.time),
        }
    }
}

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct PositiveAssertion {
    /// `openid.ns` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
    #[serde(rename = "openid.ns")]
    namespace: String,

    /// See [`crate::openid::constants::OPENID_MODE`]
    ///
    /// Value: `id_res`
    #[serde(rename = "openid.mode")]
    mode: String,

    /// See [`crate::openid::constants::OPENID_OP_ENDPOINT`]
    #[serde(rename = "openid.op_endpoint")]
    provider_endpoint: String,

    /// See [`crate::openid::constants::OPENID_CLAIMED_ID`]
    #[serde(rename = "openid.claimed_id")]
    claimed_id: String,

    /// See [`crate::openid::constants::OPENID_IDENTITY`]
    #[serde(rename = "openid.identity")]
    identity: String,

    /// See [`crate::openid::constants::OPENID_RETURN_TO`]
    ///
    /// Verbatim copy of the return_to URL parameter sent in the request.
    #[serde(rename = "openid.return_to")]
    return_to: String,

    /// See [`crate::openid::constants::OPENID_RESPONSE_NONCE`]
    #[serde(rename = "openid.response_nonce")]
    #[serde(serialize_with = "serialize_nonce")]
    #[serde(deserialize_with = "deserialize_nonce")]
    nonce: Nonce,

    /// See [`crate::openid::constants::OPENID_ASSOCIATION_HANDLE`]
    #[serde(rename = "openid.assoc_handle")]
    association_handle: String,

    /// See [`crate::openid::constants::OPENID_SIGNED_FIELDS`]
    #[serde(rename = "openid.signed")]
    #[serde(serialize_with = "serialize_comma_separated")]
    #[serde(deserialize_with = "deserialize_comma_separated")]
    signed_fields: Vec<String>,

    /// See [`crate::openid::constants::OPENID_SIGNATURE`]
    #[serde(rename = "openid.sig")]
    signature: String,
}

impl PositiveAssertion {
    /// Generic validation
    pub(crate) fn validate(&self, provider: &Provider) -> anyhow::Result<()> {
        /// Fields that must be signed as per spec
        const EXPECTED_SIGNED_FIELDS: [&str; 6] = [
            OPENID_OP_ENDPOINT,
            OPENID_RETURN_TO,
            OPENID_RESPONSE_NONCE,
            OPENID_ASSOCIATION_HANDLE,
            OPENID_CLAIMED_ID,
            OPENID_IDENTITY,
        ];

        /// - `actual`: A list of present field names _without_ the [prefix]
        /// - `expected`: A list of expected field names _with_ the [prefix]
        ///
        /// This is O(n^2) but the arrays shouldn't be too large so it's okay
        ///
        /// [prefix]: crate::openid::constants::OPENID_FIELD_PREFIX
        fn has_fields(actual: &[String], expected: &[&str]) -> bool {
            /// Check for equality without the prefix
            fn eq(actual: &str, expected: &str) -> bool {
                match expected.strip_prefix(OPENID_FIELD_PREFIX) {
                    None => false,
                    Some(expected_no_prefix) => actual == expected_no_prefix,
                }
            }
            expected
                .iter()
                .all(|expected| actual.iter().any(|actual| eq(actual, expected)))
        }

        if self.namespace != OPENID_AUTH_NAMESPACE {
            anyhow::bail!("invalid value for openid namespace");
        }
        if self.mode != OPENID_MODE_IDENTIFIER_RESPONSE {
            anyhow::bail!("invalid mode");
        }
        if self.provider_endpoint != provider.endpoint {
            anyhow::bail!("provider endpoint doesn't match");
        }
        if self.claimed_id != self.identity {
            anyhow::bail!("claimed identity doesn't match identity");
        }
        if !has_fields(&self.signed_fields, &EXPECTED_SIGNED_FIELDS) {
            anyhow::bail!("fields that should be signed aren't signed");
        }
        if self.signature.is_empty() {
            anyhow::bail!("signature field is empty");
        }

        Ok(())
    }
    /// Steam specific validation
    pub(crate) fn validate_steam(&self) -> anyhow::Result<()> {
        const STEAM_IDENTITY_PREFIX: &str = "https://steamcommunity.com/openid/id/";

        let claimed_id_id: u64 = self
            .claimed_id
            .strip_prefix(STEAM_IDENTITY_PREFIX)
            .context("claimed identity is not for a steam id")?
            .parse()
            .context("claimed identity cannot represent a steam id")?;

        let identity_id: u64 = self
            .identity
            .strip_prefix(STEAM_IDENTITY_PREFIX)
            .context("identity is not for a steam id")?
            .parse()
            .context("identity cannot represent a steam id")?;

        if claimed_id_id != identity_id {
            anyhow::bail!("claimed id doesn't match identity");
        }

        Ok(())
    }
    pub(crate) fn set_mode(&mut self, mode: &str) {
        self.mode.clear();
        self.mode.push_str(mode);
    }
}

/// Used to deserialize the comma-separated list of signed fields in [`PositiveAssertion::signed_fields`].
fn deserialize_comma_separated<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;
    let parts = str.split(',').map(|s| s.to_string());
    Ok(parts.collect())
}
fn serialize_comma_separated<S>(data: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let joined = data.join(",");
    serializer.serialize_str(&joined)
}

/// Used to deserialize the nonce in [`PositiveAssertion::nonce`]
fn deserialize_nonce<'de, D>(deserializer: D) -> Result<Nonce, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;
    let nonce: Nonce = str.parse().map_err(serde::de::Error::custom)?;
    Ok(nonce)
}
fn serialize_nonce<S>(data: &Nonce, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Make sure it matches the expected format of
    // `2001-02-03T04:05:06Z`
    use chrono::SecondsFormat::Secs;
    let mut buffer = data.time.to_rfc3339_opts(Secs, true);
    buffer.push_str(&data.salt);
    serializer.serialize_str(&buffer)
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Context;

    const TEST_URL: &str = "http://localhost:8080/auth/steam/callback/?openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0&openid.mode=id_res&openid.op_endpoint=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Flogin&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198181282063&openid.identity=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198181282063&openid.return_to=http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fsteam%2Fcallback%2F&openid.response_nonce=2023-09-15T11%3A23%3A46Z7RPb74voq1sqY2sKMcnOe%2FrxwQg%3D&openid.assoc_handle=1234567890&openid.signed=signed%2Cop_endpoint%2Cclaimed_id%2Cidentity%2Creturn_to%2Cresponse_nonce%2Cassoc_handle&openid.sig=SPaIMgwuYCQ2zVlgYmbSAKfD8Ps%3D";

    #[test]
    fn serialize_deserialize() -> anyhow::Result<()> {
        let provider = Provider::steam();

        let parsed = reqwest::Url::parse(TEST_URL).context("couldn't parse url")?;
        let query = parsed.query().context("url doesn't contain a query")?;

        let parsed: PositiveAssertion = serde_urlencoded::from_str(query)
            .context("couldn't parse positive assertion from query")?;

        parsed
            .validate(&provider)
            .context("couldn't validate response")?;
        parsed
            .validate_steam()
            .context("couldn't validate steam response")?;

        let as_query = serde_urlencoded::to_string(&parsed)
            .context("couldn't encode positive asstion back into a query")?;

        assert_eq!(query, as_query);

        Ok(())
    }
}
