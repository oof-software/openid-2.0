use std::borrow::Cow;
use std::str::FromStr;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::constants::OPENID_RESPONSE_NONCE_MAX_LEN;

/// 30 seconds between the user authorizing us and us processing
/// the response seems reasonable.
const NONCE_MAX_AGE_MS: i64 = 30_000;

#[derive(Debug, Clone)]
pub(crate) struct Nonce {
    time: DateTime<Utc>,
    salt: String,
}

impl FromStr for Nonce {
    type Err = anyhow::Error;
    fn from_str(nonce: &str) -> Result<Self, Self::Err> {
        if nonce.len() > OPENID_RESPONSE_NONCE_MAX_LEN {
            anyhow::bail!("response nonce is too long");
        }

        let last_time_char = nonce.find('Z').context("nonce doesn't adhere to spec")?;
        let (time, salt) = nonce.split_at(last_time_char + 1);

        if salt.is_empty() {
            anyhow::bail!("response nonce doesn't contain a salt");
        }

        let salt = salt.to_string();
        let time: DateTime<Utc> = DateTime::from(
            DateTime::parse_from_rfc3339(time).context("couldn't parse date and time of nonce")?,
        );

        Ok(Nonce { time, salt })
    }
}

impl ToString for Nonce {
    fn to_string(&self) -> String {
        // Make sure it matches the expected format of
        // `2001-02-03T04:05:06Z`
        use chrono::SecondsFormat::Secs;
        let mut buffer = self.time.to_rfc3339_opts(Secs, true);
        buffer.push_str(&self.salt);
        buffer
    }
}

impl Nonce {
    /// # Important!
    ///
    /// Timestamp from steam doesn't contain subseconds
    /// therefore it can be in the future by up to a second.
    pub(crate) fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp_millis();
        let then = self.time.timestamp_millis();
        now - then > NONCE_MAX_AGE_MS
    }
    pub(crate) fn as_salt(&self) -> &str {
        &self.salt
    }
    pub(crate) const fn new(salt: String, time: DateTime<Utc>) -> Nonce {
        Nonce { time, salt }
    }
}

impl<'de> Deserialize<'de> for Nonce {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = Cow::<'de, str>::deserialize(deserializer)?;
        let nonce = Nonce::from_str(&str).map_err(serde::de::Error::custom)?;
        Ok(nonce)
    }
}

impl Serialize for Nonce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
