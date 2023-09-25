use std::borrow::Cow;
use std::fmt::{Display, Write};
use std::ops::Deref;
use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

/// - Serialize `["a", "b", "c"]` into `"a,b,c"`
/// - Deserialize `"a,b,c"` into `["a", "b", "c"]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommaSeparated<T>(Vec<T>);

impl<T> CommaSeparated<T> {
    pub(crate) fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T> Deref for CommaSeparated<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> FromStr for CommaSeparated<T>
where
    T: FromStr,
{
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(CommaSeparated(Vec::new()));
        }

        let len = s.chars().filter(|&c| c == ',').count();
        let mut buffer: Vec<T> = Vec::with_capacity(len + 1);

        let parts = s.split(',');
        for part in parts {
            // TODO: Error conversion
            match part.parse() {
                Err(_) => return Err(anyhow!("couldn't parse the thingy ( ˘︹˘ )")),
                Ok(parsed) => buffer.push(parsed),
            };
        }

        Ok(CommaSeparated(buffer))
    }
}

impl<T> ToString for CommaSeparated<T>
where
    T: Display,
{
    fn to_string(&self) -> String {
        let mut buffer = String::new();
        let mut iter = self.0.iter();
        if let Some(first) = iter.next() {
            write!(&mut buffer, "{}", first).unwrap();
            for next in iter {
                write!(&mut buffer, ",{}", next).unwrap();
            }
        }
        buffer
    }
}

impl<'de, T> Deserialize<'de> for CommaSeparated<T>
where
    T: FromStr,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = Cow::<'de, str>::deserialize(deserializer)?;
        let cs = CommaSeparated::from_str(&str).map_err(serde::de::Error::custom)?;
        Ok(cs)
    }
}

impl<T> Serialize for CommaSeparated<T>
where
    T: Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use anyhow::Context;
    use serde::{Deserialize, Serialize};
    use steam_api_concurrent::SteamId;

    use super::CommaSeparated;

    const SERIALIZED: &str = "a,b,c,d,e";
    const DESERIALIZED: [&str; 5] = ["a", "b", "c", "d", "e"];

    #[test]
    fn from_str_works() -> anyhow::Result<()> {
        let parsed =
            CommaSeparated::<String>::from_str(SERIALIZED).context("deserialization failed")?;
        let parsed = parsed.into_inner();

        assert_eq!(parsed.len(), DESERIALIZED.len());
        for (actual, expected) in std::iter::zip(parsed, DESERIALIZED) {
            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[test]
    fn to_string_works() -> anyhow::Result<()> {
        let serialized: Vec<String> = DESERIALIZED.iter().map(|v| v.to_string()).collect();
        let serialized = CommaSeparated(serialized).to_string();

        assert_eq!(serialized, SERIALIZED);

        Ok(())
    }

    #[test]
    fn parses_steam_id_url() -> anyhow::Result<()> {
        #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
        struct Test {
            steam_ids: CommaSeparated<SteamId>,
        }

        let input = "&steam_ids=76561198181282063,76561198181282063,76561198181282063";
        let result = Test {
            steam_ids: CommaSeparated(vec![
                SteamId(76561198181282063),
                SteamId(76561198181282063),
                SteamId(76561198181282063),
            ]),
        };
        println!("{:?}", serde_urlencoded::to_string(&result));

        assert_eq!(Ok(result), serde_urlencoded::from_str(input));
        Ok(())
    }

    #[test]
    fn parses_steam_id_url_empty() -> anyhow::Result<()> {
        #[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
        struct Test {
            steam_ids: CommaSeparated<SteamId>,
        }

        let input = "&steam_ids=";
        let result = Test {
            steam_ids: CommaSeparated(vec![]),
        };
        println!("{:?}", serde_urlencoded::to_string(&result));

        assert_eq!(Ok(result), serde_urlencoded::from_str(input));
        Ok(())
    }
}
