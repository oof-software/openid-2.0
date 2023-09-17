use std::borrow::{Borrow, Cow};
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

impl Borrow<[String]> for CommaSeparated {
    fn borrow(&self) -> &[String] {
        self.0.as_slice()
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
