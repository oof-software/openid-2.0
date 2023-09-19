//! - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.4.1.1>
//! - <https://doc.rust-lang.org/std/primitive.str.html#method.parse>
//! - <https://serde.rs/data-format.html>
//! - <https://docs.rs/serde/1.0.188/serde/macro.forward_to_deserialize_any.html>
//! - <https://durch.github.io/rust-goauth/serde_urlencoded/index.html>
//! - <https://durch.github.io/rust-goauth/src/serde_urlencoded/de.rs.html>
//!
//! See test cases as the bottom of the file.

use std::fmt::Display;

use serde::de::value::MapDeserializer;
use serde::de::{self};
use serde::{forward_to_deserialize_any, ser, Deserialize};
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("message: {0}")]
    Message(String),
    #[error("unexpected end of input")]
    Eof,
    #[error("trailing characters")]
    TrailingCharacters,
    #[error("type not implemented")]
    NotImplemented,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

struct Deserializer<'de> {
    inner: &'de str,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { inner: input }
    }
}

// By convention, the public API of a Serde deserializer is one or more
// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
// depending on what Rust types the deserializer is able to consume as input.
//
// This basic deserializer supports only `from_str`.
pub(crate) fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    T::deserialize(Deserializer::from_str(s))
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented)
    }

    fn deserialize_i64<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
    }

    // bool u8 u16 u32 u64 i8 i16 i32 i64
    forward_to_deserialize_any! {
        f32 f64 char str string option
        bytes byte_buf unit_struct newtype_struct tuple_struct struct
        tuple enum ignored_any identifier
        unit seq map
    }
}

struct KeyValueIterator<'de> {
    input: &'de str,
}

impl<'de> Iterator for KeyValueIterator<'de> {
    type Item = (&'de str, &'de str);

    fn next(&mut self) -> Option<Self::Item> {
        let (line, remainder) = self.input.split_once('\n')?;
        let (key, value) = line.split_once(':')?;
        self.input = remainder;
        Some((key, value))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use anyhow::Context;
    use serde::Deserialize;

    use super::from_str;

    #[test]
    fn deserialize_string_struct() -> anyhow::Result<()> {
        let input = "a:a\nb:b : b \nc:\n🦀:🚀\n";

        #[derive(Deserialize)]
        struct Test {
            a: String,
            b: String,
            c: String,
            #[serde(rename = "🦀")]
            d: String,
        }

        let parsed = from_str::<Test>(input).context("parsing failed")?;
        assert_eq!(parsed.a, "a");
        assert_eq!(parsed.b, "b : b ");
        assert_eq!(parsed.c, "");
        assert_eq!(parsed.d, "🚀");

        Ok(())
    }

    #[test]
    fn deserialize_str_struct() -> anyhow::Result<()> {
        let input = "a:a\nb:b:b\n";

        #[derive(Deserialize)]
        struct Test<'a> {
            a: &'a str,
            b: &'a str,
        }

        let parsed = from_str::<Test>(input).context("parsing failed")?;
        assert_eq!(parsed.a, "a");
        assert_eq!(parsed.b, "b:b");

        Ok(())
    }

    #[test]
    fn deserialize_int_struct() -> anyhow::Result<()> {
        let input = "a:1\nb:-1\n";

        #[derive(Deserialize)]
        struct Test {
            a: i32,
            b: i32,
        }

        let parsed = from_str::<Test>(input).context("parsing failed")?;
        assert_eq!(parsed.a, 1);
        assert_eq!(parsed.b, -1);

        Ok(())
    }

    #[test]
    fn deserialize_bool_struct() -> anyhow::Result<()> {
        let input = "a:true\nb:false\n";

        #[derive(Deserialize)]
        struct Test {
            a: bool,
            b: bool,
        }

        let parsed = from_str::<Test>(input).context("parsing failed")?;
        assert_eq!(parsed.a, true);
        assert_eq!(parsed.b, false);

        Ok(())
    }

    #[test]
    fn deserialize_option_struct() -> anyhow::Result<()> {
        let input = "a:\nb:\nc:\n";

        #[derive(Deserialize)]
        struct Test {
            a: Option<String>,
            b: Option<i32>,
            c: Option<bool>,
        }

        let parsed = from_str::<Test>(input).context("parsing failed")?;
        assert_eq!(parsed.a, None);
        assert_eq!(parsed.b, None);
        assert_eq!(parsed.c, None);

        Ok(())
    }

    #[test]
    fn deserialize_hash_map_string() -> anyhow::Result<()> {
        let input = "a:a\nb:b\n";

        let parsed = from_str::<HashMap<String, String>>(input).context("parsing failed")?;
        assert_eq!(parsed.get("a").map(|s| s.as_str()), Some("a"));
        assert_eq!(parsed.get("b").map(|s| s.as_str()), Some("b"));
        assert_eq!(parsed.get("c").map(|s| s.as_str()), None);

        Ok(())
    }

    #[test]
    fn deserialize_hash_map_bool() -> anyhow::Result<()> {
        let input = "a:true\nb:false\n";

        let parsed = from_str::<HashMap<String, bool>>(input).context("parsing failed")?;
        assert_eq!(parsed.get("a"), Some(&true));
        assert_eq!(parsed.get("b"), Some(&false));
        assert_eq!(parsed.get("c"), None);

        Ok(())
    }
}
