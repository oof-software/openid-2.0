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

use std::borrow::Borrow;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use super::comma_separated::CommaSeparated;
use super::constants::*;
use super::nonce::Nonce;
use super::Provider;

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
    service_endpoint: String,

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
    nonce: Nonce,

    /// See [`crate::openid::constants::OPENID_ASSOCIATION_HANDLE`]
    #[serde(rename = "openid.assoc_handle")]
    association_handle: String,

    /// See [`crate::openid::constants::OPENID_SIGNED_FIELDS`]
    #[serde(rename = "openid.signed")]
    signed_fields: CommaSeparated,

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
                expected
                    .strip_prefix(OPENID_FIELD_PREFIX)
                    .map_or(false, |expected_no_prefix| actual == expected_no_prefix)
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
        if self.service_endpoint != provider.service.endpoint {
            anyhow::bail!("provider endpoint doesn't match");
        }
        if self.claimed_id != self.identity {
            anyhow::bail!("claimed identity doesn't match identity");
        }
        if !has_fields(self.signed_fields.borrow(), &EXPECTED_SIGNED_FIELDS) {
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

        if self.nonce.is_expired() {
            anyhow::bail!("too old");
        }

        Ok(())
    }
    pub(crate) fn set_mode(&mut self, mode: &str) {
        self.mode.clear();
        self.mode.push_str(mode);
    }
}

#[cfg(test)]
mod test {
    use anyhow::Context;
    use chrono::Utc;

    use super::*;

    const TEST_URL: &str = "http://localhost:8080/auth/steam/callback/?openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0&openid.mode=id_res&openid.op_endpoint=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Flogin&openid.claimed_id=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198181282063&openid.identity=https%3A%2F%2Fsteamcommunity.com%2Fopenid%2Fid%2F76561198181282063&openid.return_to=http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fsteam%2Fcallback%2F&openid.response_nonce=2023-09-15T11%3A23%3A46Z7RPb74voq1sqY2sKMcnOe%2FrxwQg%3D&openid.assoc_handle=1234567890&openid.signed=signed%2Cop_endpoint%2Cclaimed_id%2Cidentity%2Creturn_to%2Cresponse_nonce%2Cassoc_handle&openid.sig=SPaIMgwuYCQ2zVlgYmbSAKfD8Ps%3D";

    const TEST_PARAMS_NONCE_SALT: &str = "7RPb74voq1sqY2sKMcnOe/rxwQg=";
    const TEST_PARAMS_ENDPOINT: &str = "https://steamcommunity.com/openid/login";
    const TEST_PARAMS_ID: &str = "https://steamcommunity.com/openid/id/76561198181282063";
    const TEST_PARAMS_RETURN_TO: &str = "http://localhost:3000/auth/steam/callback/";
    const TEST_PARAMS_ASSOC_HANDLE: &str = "1234567890";
    const TEST_PARAMS_SIGNED_FIELDS: &str =
        "signed,op_endpoint,claimed_id,identity,return_to,response_nonce,assoc_handle";
    const TEST_PARAMS_SIGNATURE: &str = "SPaIMgwuYCQ2zVlgYmbSAKfD8Ps=";
    const TEST_PARAMS_NONCE: &str = "2023-09-15T11:23:46Z7RPb74voq1sqY2sKMcnOe/rxwQg=";
    const TEST_PARAMS_BASE_URL: &str = "http://localhost:8080";

    const TEST_PARAMS_WITHOUT_NONCE: [(&str, &str); 10] = [
        (OPENID_NAMESPACE, OPENID_AUTH_NAMESPACE),
        (OPENID_MODE, OPENID_MODE_IDENTIFIER_RESPONSE),
        (OPENID_OP_ENDPOINT, TEST_PARAMS_ENDPOINT),
        (OPENID_CLAIMED_ID, TEST_PARAMS_ID),
        (OPENID_IDENTITY, TEST_PARAMS_ID),
        (OPENID_RETURN_TO, TEST_PARAMS_RETURN_TO),
        (OPENID_RESPONSE_NONCE, ""),
        (OPENID_ASSOCIATION_HANDLE, TEST_PARAMS_ASSOC_HANDLE),
        (OPENID_SIGNED_FIELDS, TEST_PARAMS_SIGNED_FIELDS),
        (OPENID_SIGNATURE, TEST_PARAMS_SIGNATURE),
    ];

    fn make_test_url() -> anyhow::Result<String> {
        let mut params = TEST_PARAMS_WITHOUT_NONCE.map(|(k, v)| (k.to_string(), v.to_string()));
        let nonce_url = params
            .iter_mut()
            .find(|(k, _)| k == OPENID_RESPONSE_NONCE)
            .context("nonce field not found")?;

        let nonce = Nonce::new(TEST_PARAMS_NONCE_SALT.to_string(), Utc::now());

        // This relies on `to_string` resulting in the same
        // representation as serialized with serde_urlencoded!
        nonce_url.1 = nonce.to_string();

        let url = reqwest::Url::parse_with_params(TEST_PARAMS_BASE_URL, &params)
            .context("couldn't serialize whole quert into url")?;

        Ok(url.into())
    }

    #[test]
    fn validate_steam() -> anyhow::Result<()> {
        let provider = Provider::steam();

        let test_url = make_test_url().context("couldn't make test url")?;
        let parsed = reqwest::Url::parse(&test_url).context("couldn't parse url")?;
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

    #[test]
    fn serialize_deserialize() -> anyhow::Result<()> {
        let parsed = reqwest::Url::parse(TEST_URL).context("couldn't parse url")?;
        let query = parsed.query().context("url doesn't contain a query")?;

        let parsed: PositiveAssertion = serde_urlencoded::from_str(query)
            .context("couldn't parse positive assertion from query")?;

        assert_eq!(parsed.namespace, OPENID_AUTH_NAMESPACE);
        assert_eq!(parsed.mode, OPENID_MODE_IDENTIFIER_RESPONSE);
        assert_eq!(parsed.service_endpoint, TEST_PARAMS_ENDPOINT);
        assert_eq!(parsed.claimed_id, TEST_PARAMS_ID);
        assert_eq!(parsed.identity, TEST_PARAMS_ID);
        assert_eq!(parsed.return_to, TEST_PARAMS_RETURN_TO);
        assert_eq!(parsed.nonce.to_string(), TEST_PARAMS_NONCE);
        assert_eq!(parsed.association_handle, TEST_PARAMS_ASSOC_HANDLE);
        assert_eq!(parsed.signed_fields.to_string(), TEST_PARAMS_SIGNED_FIELDS);
        assert_eq!(parsed.signature, TEST_PARAMS_SIGNATURE);

        let as_query = serde_urlencoded::to_string(&parsed)
            .context("couldn't encode positive asstion back into a query")?;
        assert_eq!(query, as_query);

        Ok(())
    }
}
