use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.4.1.1>
///
/// # Example
///
/// The trailing newline is mandatory!
///
/// ```text
/// keyOne:valueOne\nkeyTwo:value:Two\n
/// ```
///
/// Is parsed as
///
/// ```json
/// { "keyOne": "valueOne", "keyTwo": "value:Two" }
/// ```
pub(crate) struct KeyValues(HashMap<String, String>);

impl KeyValues {
    pub(crate) fn into_inner(self) -> HashMap<String, String> {
        self.0
    }
}

impl FromStr for KeyValues {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.chars().filter(|c| *c == '\n').count();
        let mut map = HashMap::with_capacity(len);

        for line in s.split_terminator('\n') {
            let Some((key, value)) = line.split_once(':') else {
                anyhow::bail!("encountered line without colon (':')");
            };
            if map.insert(key.to_string(), value.to_string()).is_some() {
                anyhow::bail!("key `{}` is definied more than once", key);
            }
        }

        Ok(KeyValues(map))
    }
}

impl ToString for KeyValues {
    fn to_string(&self) -> String {
        let map = &self.0;
        let len = map.iter().fold(0, |acc, (k, v)| {
            // key + value + (':' + '\n')
            acc + k.len() + v.len() + 2
        });

        let mut buffer = String::with_capacity(len);
        for (k, v) in map {
            buffer.push_str(k);
            buffer.push(':');
            buffer.push_str(v);
            buffer.push('\n');
        }
        buffer
    }
}

impl<'de> Deserialize<'de> for KeyValues {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = Cow::<'de, str>::deserialize(deserializer)?;
        let cs = KeyValues::from_str(&str).map_err(serde::de::Error::custom)?;
        Ok(cs)
    }
}

impl Serialize for KeyValues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::str::FromStr;

    use anyhow::Context;

    use super::KeyValues;

    const SERIALIZED_1: &str = "url:https://forsen.forsen\nfors:forsen\n";
    const SERIALIZED_2: &str = "fors:forsen\nurl:https://forsen.forsen\n";
    const DESERIALIZED: [(&str, &str); 2] = [("url", "https://forsen.forsen"), ("fors", "forsen")];

    #[test]
    fn from_str_works() -> anyhow::Result<()> {
        let parsed = KeyValues::from_str(SERIALIZED_1).context("deserialization failed")?;
        let parsed = parsed.into_inner();

        assert_eq!(parsed.len(), DESERIALIZED.len());
        for (k, v) in DESERIALIZED {
            assert_eq!(Some(v), parsed.get(k).map(|k| k.as_str()));
        }

        Ok(())
    }

    #[test]
    fn to_string_works() -> anyhow::Result<()> {
        let serialized: HashMap<String, String> = DESERIALIZED
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let serialized = KeyValues(serialized).to_string();

        // The `HashMap` loses order so we have to check both possibilities
        if serialized != SERIALIZED_1 && serialized != SERIALIZED_2 {
            assert!(
                false,
                "serialized ({}) matches neither ({}) nor ({})",
                serialized, SERIALIZED_1, SERIALIZED_2
            );
        }

        Ok(())
    }
}
