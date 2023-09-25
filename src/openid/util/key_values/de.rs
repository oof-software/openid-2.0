use std::fmt::Display;

use serde::de::{self, MapAccess};
use serde::{ser, Deserialize};
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("message: {0}")]
    Message(String),
    #[error("unexpected end of input")]
    Eof,
    #[error("trailing characters")]
    TrailingCharacters,
    #[error("{0} is not implemented")]
    NotImplemented(&'static str),
    #[error("invalid syntax")]
    Syntax,
    #[error("expected a colon")]
    ExpectedColon,
    #[error("expected a newline")]
    ExpectedNewline,
    #[error("couldn't parse integer: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("couldn't parse float: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error("couldn't parse bool: {0}")]
    ParseBool(#[from] std::str::ParseBoolError),
    #[error("expected a char, found a string")]
    ParseChar,
    #[error("expected something as a value but found nothing")]
    NoValue,
    #[error("expected nothing but found something")]
    ExpectedEmptyValue,
    #[error("expected to parse a key")]
    ExpectedKey,
    #[error("expected to parse a value")]
    ExpectedValue,
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
    /// The input string
    inner: &'de str,
    /// To know whether we are here
    ///
    /// ```text
    /// key:value\n
    /// ^
    /// ```
    ///
    /// or here
    ///
    /// ```text
    /// key:value\n
    ///     ^
    /// ```
    consumed_key: bool,
}

impl<'de> Deserializer<'de> {
    const fn from_str(input: &'de str) -> Self {
        Deserializer {
            inner: input,
            consumed_key: false,
        }
    }
}

impl<'de> Deserializer<'de> {
    fn consume_key(&mut self) -> Result<&'de str> {
        if self.consumed_key {
            return Err(Error::ExpectedValue);
        }
        let Some((key, remainder)) = self.inner.split_once(':') else {
            return Err(Error::ExpectedColon);
        };
        self.inner = remainder;
        self.consumed_key = true;
        Ok(key)
    }
    fn peek_key(&self) -> Result<&'de str> {
        if self.consumed_key {
            return Err(Error::ExpectedValue);
        }
        let Some((key, _)) = self.inner.split_once(':') else {
            return Err(Error::ExpectedColon);
        };
        Ok(key)
    }

    /// In the case of a HashMap, a value can be used as a 'key'
    fn consume_value(&mut self) -> Result<&'de str> {
        if !self.consumed_key {
            return self.consume_key();
        }
        let Some((value, remainder)) = self.inner.split_once('\n') else {
            return Err(Error::Eof);
        };
        self.inner = remainder;
        self.consumed_key = false;
        Ok(value)
    }
    fn peek_value(&self) -> Result<&'de str> {
        if !self.consumed_key {
            return self.peek_key();
        }
        let Some((value, _)) = self.inner.split_once('\n') else {
            return Err(Error::Eof);
        };
        Ok(value)
    }
}

/// By convention, the public API of a Serde deserializer is one or more
/// `from_xyz` methods such as `from_str`, `from_bytes`, or `from_reader`
/// depending on what Rust types the deserializer is able to consume as input.
///
/// This basic deserializer supports only `from_str`.
pub fn from_str<'de, T>(s: &'de str) -> Result<T>
where
    T: Deserialize<'de>,
{
    T::deserialize(&mut Deserializer::from_str(s))
}

macro_rules! deserialize_from_str {
    ($type:ty, $deserialize_method:ident, $visit_method:ident) => {
        fn $deserialize_method<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            let value: $type = self.consume_value()?.parse()?;
            visitor.$visit_method(value)
        }
    };
}
macro_rules! deserialize_not_implemented {
    ($deserialize_method:ident) => {
        fn $deserialize_method<V>(self, _visitor: V) -> std::result::Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            Err(Error::NotImplemented(stringify!($deserialize_method)))
        }
    };
}

/// ```text
/// key = { (anything except a colon)* }
/// value = { (anything except a newline)* }
/// key_value =  { key ~ ':' ~ value ~ \n }
/// document = { key_value* }
/// ```
impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    deserialize_from_str!(i8, deserialize_i8, visit_i8);
    deserialize_from_str!(i16, deserialize_i16, visit_i16);
    deserialize_from_str!(i32, deserialize_i32, visit_i32);
    deserialize_from_str!(i64, deserialize_i64, visit_i64);
    deserialize_from_str!(u8, deserialize_u8, visit_u8);
    deserialize_from_str!(u16, deserialize_u16, visit_u16);
    deserialize_from_str!(u32, deserialize_u32, visit_u32);
    deserialize_from_str!(u64, deserialize_u64, visit_u64);
    deserialize_from_str!(f32, deserialize_f32, visit_f32);
    deserialize_from_str!(f64, deserialize_f64, visit_f64);
    deserialize_from_str!(bool, deserialize_bool, visit_bool);

    deserialize_not_implemented!(deserialize_bytes);
    deserialize_not_implemented!(deserialize_byte_buf);
    deserialize_not_implemented!(deserialize_any);
    deserialize_not_implemented!(deserialize_seq);

    fn deserialize_str<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.consume_value()?;
        visitor.visit_borrowed_str(value)
    }

    fn deserialize_string<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // same as usual, get the value
        let value = self.consume_value()?;
        // iterate over chars because unicode stuff
        let mut chars = value.chars();

        // if there are no chars, that aint good
        let Some(char) = chars.next() else {
            return Err(Error::NoValue);
        };

        // if there are too many chars that aint good as well
        if chars.next().is_some() {
            return Err(Error::ParseChar);
        }

        // visit the var
        visitor.visit_char(char)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented("deserialize_unit_struct"))
    }

    fn deserialize_tuple<V>(
        self,
        _len: usize,
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented("deserialize_tuple"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented("deserialize_tuple_struct"))
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.peek_value()?;
        if value.is_empty() {
            let _ = self.consume_value()?;
            return visitor.visit_none();
        }
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.consume_value()?;
        if value.is_empty() {
            return visitor.visit_unit();
        }
        Err(Error::ExpectedEmptyValue)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(KeyValueMapAccess { de: self })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented("deserialize_enum"))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let key = self.consume_key()?;
        visitor.visit_str(key)
    }
}

struct KeyValueMapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> MapAccess<'de> for KeyValueMapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        // we've consumed the whole thingy
        if self.de.inner.is_empty() {
            return Ok(None);
        }

        // look for the key terminator
        if self.de.inner.find(':').is_none() {
            return Err(Error::ExpectedColon);
        };

        // deserialize the key
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        // a value always has to be terminated by a newline
        if self.de.inner.find('\n').is_none() {
            return Err(Error::ExpectedNewline);
        }

        // deserialize the value
        seed.deserialize(&mut *self.de)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use anyhow::Context;
    use serde::Deserialize;

    use super::from_str;

    macro_rules! assert_parse_error {
        ($input:literal) => {{
            let parsed = from_str::<HashMap<String, String>>($input);
            if !parsed.is_err() {
                println!("parsed: {:?}", parsed);
            }
            assert!(parsed.is_err());
        }};
    }

    macro_rules! assert_not_implemented {
        ($type:ty, $input:literal) => {
            let parsed = from_str::<$type>($input);
            assert!(parsed.is_err());
        };
    }

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
    fn deserialize_seq_of_pairs() -> anyhow::Result<()> {
        assert_not_implemented!(Vec<(String, String)>, "a:a\nb:b:b\n");
        Ok(())
    }

    #[test]
    fn deserialize_str_borrow_struct() -> anyhow::Result<()> {
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
    fn deserialize_excess_fields() -> anyhow::Result<()> {
        let input = "a:1\nb:-1\n";

        #[derive(Deserialize)]
        struct Test {
            a: i32,
        }

        let parsed = from_str::<Test>(input).context("parsing failed")?;
        assert_eq!(parsed.a, 1);

        Ok(())
    }

    #[test]
    fn deserialize_trailing_characters() -> anyhow::Result<()> {
        assert_parse_error!("a:1\nb:-1\ns");
        Ok(())
    }

    #[test]
    fn deserialize_missing_newline() -> anyhow::Result<()> {
        assert_parse_error!("a:1\nb:-1");
        Ok(())
    }

    #[test]
    fn deserialize_trailing_newline() -> anyhow::Result<()> {
        assert_parse_error!("a:1\nb:-1\n\n");
        Ok(())
    }

    #[test]
    fn deserialize_duplicate_identifier() -> anyhow::Result<()> {
        let input = "a:1\na:-1\n";

        let parsed = from_str::<HashMap<String, i32>>(input).context("parsing failed")?;

        assert_eq!(parsed.get("a"), Some(&-1));

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
    fn deserialize_duplicate_identifier_sturct() -> anyhow::Result<()> {
        let input = "a:true\na:false\n";

        #[derive(Deserialize)]
        struct Test {
            a: bool,
        }

        let parsed = from_str::<Test>(input);
        assert!(parsed.is_err());

        Ok(())
    }

    #[test]
    fn deserialize_option_struct() -> anyhow::Result<()> {
        let input = "a:\nb:42\nc:\n";

        #[derive(Deserialize)]
        struct Test {
            a: Option<String>,
            b: Option<i32>,
            c: Option<bool>,
        }

        let parsed = from_str::<Test>(input).context("parsing failed")?;
        assert_eq!(parsed.a, None);
        assert_eq!(parsed.b, Some(42));
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

    #[test]
    fn deserialize_hash_map_int_int() -> anyhow::Result<()> {
        let input = "1:1\n2:2\n-1:-1\n";

        let parsed = from_str::<HashMap<i32, i32>>(input).context("parsing failed")?;
        assert_eq!(parsed.get(&1), Some(&1));
        assert_eq!(parsed.get(&2), Some(&2));
        assert_eq!(parsed.get(&-1), Some(&-1));
        assert_eq!(parsed.get(&-2), None);

        Ok(())
    }

    #[test]
    fn deserialize_unit() -> anyhow::Result<()> {
        let input = "a:\n";

        #[derive(Deserialize)]
        struct Test {
            a: (),
        }

        let parsed = from_str::<Test>(input).context("parsing failed")?;
        assert_eq!(parsed.a, ());

        Ok(())
    }

    #[test]
    fn deserialize_unit_enum() -> anyhow::Result<()> {
        #[derive(Deserialize, Debug, PartialEq, Eq)]
        enum Test {
            A,
        }
        assert_not_implemented!(HashMap<String, Test>, "one:A\n");
        Ok(())
    }

    #[test]
    fn deserialize_null_key_map() -> anyhow::Result<()> {
        let input = ":-1\n";
        let parsed = from_str::<HashMap<(), i32>>(input).context("parsing failed")?;
        assert_eq!(parsed.get(&()), Some(&-1));
        Ok(())
    }

    #[test]
    fn deserialize_null_value_map() -> anyhow::Result<()> {
        let input = "-1:\n0:\n1:\n";
        let parsed = from_str::<HashMap<i32, ()>>(input).context("parsing failed")?;
        assert_eq!(parsed.get(&-1), Some(&()));
        assert_eq!(parsed.get(&0), Some(&()));
        assert_eq!(parsed.get(&1), Some(&()));
        assert_eq!(parsed.get(&2), None);
        Ok(())
    }

    #[test]
    fn deserialize_newtype_struct() -> anyhow::Result<()> {
        let input = "a:42\n";

        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct NewType(i32);

        #[derive(Deserialize)]
        struct Test {
            a: NewType,
        }

        let parsed = from_str::<Test>(input).context("parsing failed")?;
        assert_eq!(parsed.a, NewType(42));

        Ok(())
    }
}
