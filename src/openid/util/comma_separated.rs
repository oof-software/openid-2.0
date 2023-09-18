use std::borrow::Cow;
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// - Serialize `["a", "b", "c"]` into `"a,b,c"`
/// - Deserialize `"a,b,c"` into `["a", "b", "c"]`
#[derive(Debug, Clone)]
pub(crate) struct CommaSeparated(Vec<String>);

impl CommaSeparated {
    pub(crate) fn into_inner(self) -> Vec<String> {
        self.0
    }
}

impl Deref for CommaSeparated {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for CommaSeparated {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(',').map(|s| s.to_string());
        Ok(CommaSeparated(parts.collect()))
    }
}

impl ToString for CommaSeparated {
    fn to_string(&self) -> String {
        self.0.join(",")
    }
}

impl<'de> Deserialize<'de> for CommaSeparated {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = Cow::<'de, str>::deserialize(deserializer)?;
        let cs = CommaSeparated::from_str(&str).map_err(serde::de::Error::custom)?;
        Ok(cs)
    }
}

impl Serialize for CommaSeparated {
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

    use super::CommaSeparated;

    const SERIALIZED: &str = "a,b,c,d,e";
    const DESERIALIZED: [&str; 5] = ["a", "b", "c", "d", "e"];

    #[test]
    fn from_str_works() -> anyhow::Result<()> {
        let parsed = CommaSeparated::from_str(SERIALIZED).context("deserialization failed")?;
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
}
