use std::fmt::Display;

use serde::de::{self, SeqAccess};
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
    #[error("expected a comma")]
    ExpectedComma,
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
}

impl<'de> Deserializer<'de> {
    const fn from_str(input: &'de str) -> Self {
        Deserializer { inner: input }
    }
}

impl<'de> Deserializer<'de> {
    /// In the case of a HashMap, a value can be used as a 'key'
    fn consume_value(&mut self) -> Result<&'de str> {
        // a value is supposed to be here but theres nothing...
        if self.inner.is_empty() {
            return Err(Error::Eof);
        }

        match self.inner.find(',') {
            None => {
                // there is no comma after this, so the whole
                // thing has to be the value
                Ok(std::mem::take(&mut self.inner))
            }
            Some(index) => {
                // there is a comma after this, take the part
                // that belongs to this value and leave the comma
                // after it
                let (value, remainder) = self.inner.split_at(index);
                self.inner = remainder;
                Ok(value)
            }
        }
    }
    fn peek_value(&self) -> Result<&'de str> {
        if self.inner.is_empty() {
            return Err(Error::Eof);
        }

        self.inner
            .find(',')
            .map_or(Ok(self.inner), |index| (Ok(&self.inner[..index])))
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
/// value = { (anything except a comma)* }
/// follow_value = { ',' ~ value }
/// list = { (value ~ follow_value*)? }
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
    deserialize_not_implemented!(deserialize_char);
    deserialize_not_implemented!(deserialize_map);
    deserialize_not_implemented!(deserialize_identifier);

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

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let value = self.peek_value()?;
        if value.is_empty() {
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

    fn deserialize_seq<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(CommaSeparatedSeqAccess {
            de: self,
            is_first: true,
        })
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

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
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
        todo!()
    }
}

struct CommaSeparatedSeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    is_first: bool,
}

impl<'de, 'a> SeqAccess<'de> for CommaSeparatedSeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> std::result::Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.de.inner.is_empty() {
            return Ok(None);
        }

        if self.is_first {
            self.is_first = false;
        } else {
            self.de.inner = &self.de.inner[1..];
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}

#[cfg(test)]
mod test {
    use steam_api_concurrent::SteamId;

    use super::from_str;

    #[test]
    fn it_works() -> anyhow::Result<()> {
        let input = "";
        let result: Vec<i32> = vec![];

        assert_eq!(Ok(result), from_str(input));
        Ok(())
    }

    #[test]
    fn parses_steam_id() -> anyhow::Result<()> {
        let input = "76561198181282063,76561198181282063,76561198181282063";
        let result = vec![
            SteamId(76561198181282063),
            SteamId(76561198181282063),
            SteamId(76561198181282063),
        ];

        assert_eq!(Ok(result), from_str(input));
        Ok(())
    }

    #[test]
    fn parses_steam_id_optional() -> anyhow::Result<()> {
        let input = "76561198181282063,,76561198181282063";
        let result = vec![
            Some(SteamId(76561198181282063)),
            None,
            Some(SteamId(76561198181282063)),
        ];

        assert_eq!(Ok(result), from_str(input));
        Ok(())
    }
}
