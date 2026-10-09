//! A serde deserializer over our JSONC tree (PLAN.md 4.6).
//!
//! Typed rules data is deserialized straight from [`Node`]s, so every error
//! keeps the source span of the value it refers to, and numbers never pass
//! through floating point: integer tokens are read from their text, and a
//! request for a floating-point value is an error.

use std::fmt;

use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};

use crate::jsonc::{Node, Span, Value};

/// A deserialization error and the span of the value it concerns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// What went wrong.
    pub message: String,
    /// Where, if known.
    pub span: Option<Span>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self {
            message: msg.to_string(),
            span: None,
        }
    }
}

impl Error {
    fn at(span: Span, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }

    /// Attaches `span` if the error has none yet (errors raised by derived
    /// code carry no span of their own).
    fn or_at(mut self, span: Span) -> Self {
        if self.span.is_none() {
            self.span = Some(span);
        }
        self
    }
}

/// Deserializes a `T` from `node`.
pub fn from_node<'de, T: de::Deserialize<'de>>(node: &'de Node) -> Result<T, Error> {
    T::deserialize(NodeDe(node)).map_err(|e| e.or_at(node.span))
}

/// The deserializer for one node.
#[derive(Clone, Copy)]
pub struct NodeDe<'a>(pub &'a Node);

impl NodeDe<'_> {
    fn kind(&self) -> &'static str {
        match self.0.value {
            Value::Null => "null",
            Value::Bool(_) => "a boolean",
            Value::Integer(_) => "an integer",
            Value::String(_) => "a string",
            Value::Array(_) => "an array",
            Value::Object(_) => "an object",
        }
    }

    fn wrong(&self, expected: &str) -> Error {
        Error::at(
            self.0.span,
            format!("expected {expected}, found {}", self.kind()),
        )
    }

    fn integer(&self) -> Result<i128, Error> {
        let Value::Integer(i) = &self.0.value else {
            return Err(self.wrong("an integer"));
        };
        let text = i.text();
        let (neg, digits) = match text.strip_prefix('-') {
            Some(d) => (true, d),
            None => (false, text),
        };
        let mut v: i128 = 0;
        for d in digits.bytes() {
            // nocite: decimal radix of the integer token
            v = v
                .checked_mul(10)
                .and_then(|v| v.checked_add(i128::from(d - b'0')))
                .ok_or_else(|| Error::at(self.0.span, "integer out of range"))?;
        }
        Ok(if neg { -v } else { v })
    }

    fn int<T: TryFrom<i128>>(&self, what: &str) -> Result<T, Error> {
        let v = self.integer()?;
        T::try_from(v).map_err(|_| Error::at(self.0.span, format!("{v} does not fit {what}")))
    }
}

macro_rules! int_method {
    ($method:ident, $visit:ident, $ty:ty) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            let v: $ty = self.int(stringify!($ty))?;
            visitor.$visit(v).map_err(|e: Error| e.or_at(self.0.span))
        }
    };
}

impl<'de> de::Deserializer<'de> for NodeDe<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let span = self.0.span;
        let r = match &self.0.value {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(*b),
            Value::Integer(_) => {
                let v = self.integer()?;
                match i64::try_from(v) {
                    Ok(v) => visitor.visit_i64(v),
                    Err(_) => visitor.visit_i128(v),
                }
            }
            Value::String(s) => visitor.visit_borrowed_str(s),
            Value::Array(items) => visitor.visit_seq(Seq { items, i: 0 }),
            Value::Object(members) => visitor.visit_map(Map {
                members,
                i: 0,
                value: None,
            }),
        };
        r.map_err(|e| e.or_at(span))
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0.value {
            Value::Bool(b) => visitor.visit_bool(b),
            _ => Err(self.wrong("a boolean")),
        }
    }

    int_method!(deserialize_i8, visit_i8, i8);
    int_method!(deserialize_i16, visit_i16, i16);
    int_method!(deserialize_i32, visit_i32, i32);
    int_method!(deserialize_i64, visit_i64, i64);
    int_method!(deserialize_i128, visit_i128, i128);
    int_method!(deserialize_u8, visit_u8, u8);
    int_method!(deserialize_u16, visit_u16, u16);
    int_method!(deserialize_u32, visit_u32, u32);
    int_method!(deserialize_u64, visit_u64, u64);

    fn deserialize_f32<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Error> {
        Err(Error::at(
            self.0.span,
            "floating-point fields are not allowed",
        ))
    }

    fn deserialize_f64<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Error> {
        Err(Error::at(
            self.0.span,
            "floating-point fields are not allowed",
        ))
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match &self.0.value {
            Value::String(s) => visitor.visit_borrowed_str(s),
            _ => Err(self.wrong("a string")),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0.value {
            Value::Null => visitor.visit_unit(),
            _ => Err(self.wrong("null")),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match &self.0.value {
            Value::Array(items) => visitor
                .visit_seq(Seq { items, i: 0 })
                .map_err(|e| e.or_at(self.0.span)),
            _ => Err(self.wrong("an array")),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Error> {
        match &self.0.value {
            Value::Array(items) if items.len() == len => self.deserialize_seq(visitor),
            Value::Array(items) => Err(Error::at(
                self.0.span,
                format!("expected {len} items, found {}", items.len()),
            )),
            _ => Err(self.wrong("an array")),
        }
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match &self.0.value {
            Value::Object(members) => visitor
                .visit_map(Map {
                    members,
                    i: 0,
                    value: None,
                })
                .map_err(|e| e.or_at(self.0.span)),
            _ => Err(self.wrong("an object")),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match &self.0.value {
            // A unit variant written as a string.
            Value::String(s) => visitor
                .visit_enum(s.as_str().into_deserializer())
                .map_err(|e: Error| e.or_at(self.0.span)),
            // An externally tagged variant: {"variant": value}.
            Value::Object(members) if members.len() == 1 => visitor
                .visit_enum(Enum {
                    tag: &members[0].key,
                    value: &members[0].value,
                })
                .map_err(|e| e.or_at(self.0.span)),
            _ => Err(self.wrong("a variant name or a one-member object")),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }
}

struct Seq<'a> {
    items: &'a [Node],
    i: usize,
}

impl<'de> SeqAccess<'de> for Seq<'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        let Some(node) = self.items.get(self.i) else {
            return Ok(None);
        };
        self.i += 1;
        seed.deserialize(NodeDe(node))
            .map(Some)
            .map_err(|e| e.or_at(node.span))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.items.len() - self.i)
    }
}

struct Map<'a> {
    members: &'a [crate::jsonc::Member],
    i: usize,
    value: Option<&'a Node>,
}

impl<'de> MapAccess<'de> for Map<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        let Some(m) = self.members.get(self.i) else {
            return Ok(None);
        };
        self.i += 1;
        self.value = Some(&m.value);
        let key: de::value::BorrowedStrDeserializer<'de, Error> =
            de::value::BorrowedStrDeserializer::new(&m.key);
        seed.deserialize(key)
            .map(Some)
            .map_err(|e| e.or_at(m.key_span))
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let node = self
            .value
            .take()
            .ok_or_else(|| <Error as de::Error>::custom("value without a key"))?;
        seed.deserialize(NodeDe(node))
            .map_err(|e| e.or_at(node.span))
    }
}

struct Enum<'a> {
    tag: &'a str,
    value: &'a Node,
}

impl<'de> de::EnumAccess<'de> for Enum<'de> {
    type Error = Error;
    type Variant = NodeDe<'de>;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error> {
        let tag: de::value::BorrowedStrDeserializer<'de, Error> =
            de::value::BorrowedStrDeserializer::new(self.tag);
        Ok((seed.deserialize(tag)?, NodeDe(self.value)))
    }
}

impl<'de> de::VariantAccess<'de> for NodeDe<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.0.value {
            Value::Null => Ok(()),
            _ => Err(self.wrong("null")),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Error> {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde::Deserialize;

    use super::*;
    use crate::jsonc;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Thing {
        name: String,
        count: u8,
        #[serde(default)]
        tags: Vec<String>,
        rgb: [u8; 3],
        mode: Mode,
        extra: Option<BTreeMap<String, i64>>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum Mode {
        Plain,
        Fancy { level: i32 },
    }

    #[test]
    fn typed_values_with_spans() {
        let src = r#"{"name": "x", "count": 7, "rgb": [1, 2, 3], "mode": {"fancy": {"level": -4}}, "extra": {"a": 9}}"#;
        let node = jsonc::parse(src).unwrap();
        let t: Thing = from_node(&node).unwrap();
        assert_eq!(t.count, 7);
        assert_eq!(t.mode, Mode::Fancy { level: -4 });
        assert_eq!(t.extra.unwrap()["a"], 9);
        let plain =
            jsonc::parse(r#"{"name": "x", "count": 1, "rgb": [1,2,3], "mode": "plain"}"#).unwrap();
        assert_eq!(from_node::<Thing>(&plain).unwrap().mode, Mode::Plain);
    }

    #[test]
    fn errors_point_at_the_value() {
        let src = r#"{"name": "x", "count": 300, "rgb": [1, 2, 3], "mode": "plain"}"#;
        let node = jsonc::parse(src).unwrap();
        let e = from_node::<Thing>(&node).unwrap_err();
        let span = e.span.unwrap();
        assert_eq!(&src[span.start..span.end], "300");
        let src = r#"{"name": "x", "count": 3, "rgb": [1, 2, 3], "mode": "plain", "bogus": 1}"#;
        let e = from_node::<Thing>(&jsonc::parse(src).unwrap()).unwrap_err();
        assert!(e.message.contains("bogus"), "{e}");
        let src = r#"{"name": "x", "count": 3, "rgb": [1, 2], "mode": "plain"}"#;
        assert!(from_node::<Thing>(&jsonc::parse(src).unwrap()).is_err());
    }
}
