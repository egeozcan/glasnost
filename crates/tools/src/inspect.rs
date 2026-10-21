//! A field-by-field view of any serializable value (M9 N4, the desync
//! tool): a value is turned into a [`Value`] tree that keeps every struct
//! field's name, and two trees are compared into the paths that differ
//! ([`diff`]), each with both values as text.
//!
//! It reads only what `serde::Serialize` gives, so it needs nothing from
//! the simulation but its derived serializers, and it changes nothing. Our
//! own tool; no spec applies.

use serde::ser::{self, Serialize};

/// A serialized value with its names.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// `()`, a unit struct.
    Unit,
    /// A boolean.
    Bool(bool),
    /// An integer of any width.
    Int(i128),
    /// A float (the simulation has none; other callers might).
    Float(String),
    /// A string or a character.
    Text(String),
    /// Bytes.
    Bytes(Vec<u8>),
    /// `None`.
    Nothing,
    /// `Some(value)`.
    Just(Box<Value>),
    /// A sequence, a tuple or a tuple struct.
    Seq(Vec<Value>),
    /// A map, in its own order.
    Map(Vec<(Value, Value)>),
    /// A struct, its fields in declaration order.
    Struct(Vec<(&'static str, Value)>),
    /// An enum variant and what it holds ([`Value::Unit`] for none).
    Variant(&'static str, Box<Value>),
}

/// Why a value could not be viewed.
#[derive(Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

/// The view of a value.
pub fn view<T: Serialize + ?Sized>(v: &T) -> Result<Value, Error> {
    v.serialize(Viewer)
}

/// The longest text one value is shown as.
// nocite: our own layout
const SHOWN: usize = 160;

impl Value {
    fn write(&self, out: &mut String) {
        use std::fmt::Write as _;
        if out.len() > SHOWN {
            return;
        }
        match self {
            Value::Unit => out.push_str("()"),
            Value::Bool(b) => {
                let _ = write!(out, "{b}");
            }
            Value::Int(i) => {
                let _ = write!(out, "{i}");
            }
            Value::Float(f) => out.push_str(f),
            Value::Text(t) => {
                let _ = write!(out, "{t:?}");
            }
            Value::Bytes(b) => {
                let _ = write!(out, "<{} bytes>", b.len());
            }
            Value::Nothing => out.push_str("None"),
            Value::Just(v) => {
                out.push_str("Some(");
                v.write(out);
                out.push(')');
            }
            Value::Seq(items) => {
                out.push('[');
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    v.write(out);
                    if out.len() > SHOWN {
                        break;
                    }
                }
                out.push(']');
            }
            Value::Map(items) => {
                out.push('{');
                for (i, (k, v)) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    k.write(out);
                    out.push_str(": ");
                    v.write(out);
                    if out.len() > SHOWN {
                        break;
                    }
                }
                out.push('}');
            }
            Value::Struct(fields) => {
                out.push('{');
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(k);
                    out.push_str(": ");
                    v.write(out);
                    if out.len() > SHOWN {
                        break;
                    }
                }
                out.push('}');
            }
            Value::Variant(name, inner) => {
                out.push_str(name);
                match inner.as_ref() {
                    Value::Unit => {}
                    Value::Struct(_) => {
                        out.push(' ');
                        inner.write(out);
                    }
                    other => {
                        out.push('(');
                        other.write(out);
                        out.push(')');
                    }
                }
            }
        }
    }

    /// The value as one line of text, shortened past a length.
    pub fn show(&self) -> String {
        let mut s = String::new();
        self.write(&mut s);
        if s.len() > SHOWN {
            let mut cut = SHOWN;
            while !s.is_char_boundary(cut) {
                cut -= 1;
            }
            s.truncate(cut);
            s.push('…');
        }
        s
    }

    /// The field `name` of a struct.
    pub fn field(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Struct(fields) => fields.iter().find(|(k, _)| *k == name).map(|(_, v)| v),
            _ => None,
        }
    }
}

/// One differing field: its path and both values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldDiff {
    /// The path from the compared value, such as `pos.x` or `queue[2]`.
    pub path: String,
    /// The first value.
    pub a: String,
    /// The second value.
    pub b: String,
}

/// The paths at which `a` and `b` differ, at most `limit` of them, under
/// the path `at`. A struct is compared field by field, a sequence of the
/// same length item by item, a map key by key; anything else, or a
/// sequence whose length changed, as a whole.
pub fn diff(at: &str, a: &Value, b: &Value, limit: usize, out: &mut Vec<FieldDiff>) {
    if a == b || out.len() >= limit {
        return;
    }
    let join = |name: &str| {
        if at.is_empty() {
            name.to_owned()
        } else {
            format!("{at}.{name}")
        }
    };
    match (a, b) {
        (Value::Struct(fa), Value::Struct(fb))
            if fa.len() == fb.len() && fa.iter().zip(fb).all(|(x, y)| x.0 == y.0) =>
        {
            for ((k, x), (_, y)) in fa.iter().zip(fb) {
                diff(&join(k), x, y, limit, out);
            }
        }
        (Value::Seq(xa), Value::Seq(xb)) if xa.len() == xb.len() => {
            for (i, (x, y)) in xa.iter().zip(xb).enumerate() {
                diff(&format!("{at}[{i}]"), x, y, limit, out);
            }
        }
        (Value::Map(ma), Value::Map(mb)) => {
            let keyed = |m: &[(Value, Value)]| -> std::collections::BTreeMap<String, Value> {
                m.iter().map(|(k, v)| (k.show(), v.clone())).collect()
            };
            let (ka, kb) = (keyed(ma), keyed(mb));
            let keys: std::collections::BTreeSet<&String> = ka.keys().chain(kb.keys()).collect();
            for k in keys {
                let p = format!("{at}[{k}]");
                match (ka.get(k), kb.get(k)) {
                    (Some(x), Some(y)) => diff(&p, x, y, limit, out),
                    (x, y) => {
                        if out.len() < limit {
                            out.push(FieldDiff {
                                path: p,
                                a: x.map_or_else(|| "(none)".into(), Value::show),
                                b: y.map_or_else(|| "(none)".into(), Value::show),
                            });
                        }
                    }
                }
            }
        }
        (Value::Just(x), Value::Just(y)) => diff(at, x, y, limit, out),
        (Value::Variant(na, x), Value::Variant(nb, y)) if na == nb => diff(at, x, y, limit, out),
        _ => out.push(FieldDiff {
            path: if at.is_empty() {
                "(value)".into()
            } else {
                at.into()
            },
            a: a.show(),
            b: b.show(),
        }),
    }
}

struct Viewer;

/// Collects a sequence, a tuple or a tuple struct.
struct SeqOf(Vec<Value>);

/// Collects a tuple or struct variant.
struct VariantOf(&'static str, Vec<Value>, Vec<(&'static str, Value)>);

/// Collects a map.
struct MapOf(Vec<(Value, Value)>, Option<Value>);

/// Collects a struct.
struct StructOf(Vec<(&'static str, Value)>);

impl ser::Serializer for Viewer {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SeqOf;
    type SerializeTuple = SeqOf;
    type SerializeTupleStruct = SeqOf;
    type SerializeTupleVariant = VariantOf;
    type SerializeMap = MapOf;
    type SerializeStruct = StructOf;
    type SerializeStructVariant = VariantOf;

    fn serialize_bool(self, v: bool) -> Result<Value, Error> {
        Ok(Value::Bool(v))
    }
    fn serialize_i8(self, v: i8) -> Result<Value, Error> {
        Ok(Value::Int(v.into()))
    }
    fn serialize_i16(self, v: i16) -> Result<Value, Error> {
        Ok(Value::Int(v.into()))
    }
    fn serialize_i32(self, v: i32) -> Result<Value, Error> {
        Ok(Value::Int(v.into()))
    }
    fn serialize_i64(self, v: i64) -> Result<Value, Error> {
        Ok(Value::Int(v.into()))
    }
    fn serialize_i128(self, v: i128) -> Result<Value, Error> {
        Ok(Value::Int(v))
    }
    fn serialize_u8(self, v: u8) -> Result<Value, Error> {
        Ok(Value::Int(v.into()))
    }
    fn serialize_u16(self, v: u16) -> Result<Value, Error> {
        Ok(Value::Int(v.into()))
    }
    fn serialize_u32(self, v: u32) -> Result<Value, Error> {
        Ok(Value::Int(v.into()))
    }
    fn serialize_u64(self, v: u64) -> Result<Value, Error> {
        Ok(Value::Int(v.into()))
    }
    fn serialize_u128(self, v: u128) -> Result<Value, Error> {
        i128::try_from(v)
            .map(Value::Int)
            .or_else(|_| Ok(Value::Text(v.to_string())))
    }
    fn serialize_f32(self, v: f32) -> Result<Value, Error> {
        Ok(Value::Float(v.to_string()))
    }
    fn serialize_f64(self, v: f64) -> Result<Value, Error> {
        Ok(Value::Float(v.to_string()))
    }
    fn serialize_char(self, v: char) -> Result<Value, Error> {
        Ok(Value::Text(v.to_string()))
    }
    fn serialize_str(self, v: &str) -> Result<Value, Error> {
        Ok(Value::Text(v.to_owned()))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Value, Error> {
        Ok(Value::Bytes(v.to_vec()))
    }
    fn serialize_none(self) -> Result<Value, Error> {
        Ok(Value::Nothing)
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<Value, Error> {
        Ok(Value::Just(Box::new(v.serialize(Viewer)?)))
    }
    fn serialize_unit(self) -> Result<Value, Error> {
        Ok(Value::Unit)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Value, Error> {
        Ok(Value::Unit)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Value, Error> {
        Ok(Value::Variant(variant, Box::new(Value::Unit)))
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<Value, Error> {
        v.serialize(Viewer)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        v: &T,
    ) -> Result<Value, Error> {
        Ok(Value::Variant(variant, Box::new(v.serialize(Viewer)?)))
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<SeqOf, Error> {
        Ok(SeqOf(Vec::with_capacity(len.unwrap_or(0))))
    }
    fn serialize_tuple(self, len: usize) -> Result<SeqOf, Error> {
        Ok(SeqOf(Vec::with_capacity(len)))
    }
    fn serialize_tuple_struct(self, _: &'static str, len: usize) -> Result<SeqOf, Error> {
        Ok(SeqOf(Vec::with_capacity(len)))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<VariantOf, Error> {
        Ok(VariantOf(variant, Vec::new(), Vec::new()))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<MapOf, Error> {
        Ok(MapOf(Vec::new(), None))
    }
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<StructOf, Error> {
        Ok(StructOf(Vec::with_capacity(len)))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<VariantOf, Error> {
        Ok(VariantOf(variant, Vec::new(), Vec::new()))
    }
}

impl ser::SerializeSeq for SeqOf {
    type Ok = Value;
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        self.0.push(v.serialize(Viewer)?);
        Ok(())
    }
    fn end(self) -> Result<Value, Error> {
        Ok(Value::Seq(self.0))
    }
}

impl ser::SerializeTuple for SeqOf {
    type Ok = Value;
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        ser::SerializeSeq::serialize_element(self, v)
    }
    fn end(self) -> Result<Value, Error> {
        Ok(Value::Seq(self.0))
    }
}

impl ser::SerializeTupleStruct for SeqOf {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        ser::SerializeSeq::serialize_element(self, v)
    }
    fn end(self) -> Result<Value, Error> {
        Ok(Value::Seq(self.0))
    }
}

impl ser::SerializeTupleVariant for VariantOf {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        self.1.push(v.serialize(Viewer)?);
        Ok(())
    }
    fn end(self) -> Result<Value, Error> {
        Ok(Value::Variant(self.0, Box::new(Value::Seq(self.1))))
    }
}

impl ser::SerializeStructVariant for VariantOf {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        v: &T,
    ) -> Result<(), Error> {
        self.2.push((key, v.serialize(Viewer)?));
        Ok(())
    }
    fn end(self) -> Result<Value, Error> {
        Ok(Value::Variant(self.0, Box::new(Value::Struct(self.2))))
    }
}

impl ser::SerializeMap for MapOf {
    type Ok = Value;
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, k: &T) -> Result<(), Error> {
        self.1 = Some(k.serialize(Viewer)?);
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), Error> {
        let k = self
            .1
            .take()
            .ok_or_else(|| Error("a map value without its key".into()))?;
        self.0.push((k, v.serialize(Viewer)?));
        Ok(())
    }
    fn end(self) -> Result<Value, Error> {
        Ok(Value::Map(self.0))
    }
}

impl ser::SerializeStruct for StructOf {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        v: &T,
    ) -> Result<(), Error> {
        self.0.push((key, v.serialize(Viewer)?));
        Ok(())
    }
    fn skip_field(&mut self, _: &'static str) -> Result<(), Error> {
        Ok(())
    }
    fn end(self) -> Result<Value, Error> {
        Ok(Value::Struct(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(serde::Serialize)]
    struct Cell {
        x: i32,
        y: i32,
    }

    #[derive(serde::Serialize)]
    enum Goal {
        Idle,
        Go(Cell),
        Hit { at: u32, by: Option<u16> },
    }

    #[derive(serde::Serialize)]
    struct Thing {
        health: i32,
        pos: Cell,
        goal: Goal,
        queue: Vec<u8>,
        slots: BTreeMap<(i32, i32), u32>,
    }

    fn thing(health: i32, y: i32, goal: Goal, queue: Vec<u8>) -> Thing {
        Thing {
            health,
            pos: Cell { x: 3, y },
            goal,
            queue,
            slots: BTreeMap::from([((1, 2), 5)]),
        }
    }

    #[test]
    fn differences_are_named_by_path() {
        let a = view(&thing(10, 4, Goal::Hit { at: 3, by: Some(2) }, vec![1, 2])).unwrap();
        let b = view(&thing(11, 5, Goal::Hit { at: 3, by: Some(9) }, vec![1, 7])).unwrap();
        let mut out = Vec::new();
        diff("", &a, &b, 10, &mut out);
        let paths: Vec<(&str, &str, &str)> = out
            .iter()
            .map(|d| (d.path.as_str(), d.a.as_str(), d.b.as_str()))
            .collect();
        assert_eq!(
            paths,
            [
                ("health", "10", "11"),
                ("pos.y", "4", "5"),
                ("goal.by", "2", "9"),
                ("queue[1]", "2", "7"),
            ]
        );
        let c = view(&thing(10, 4, Goal::Idle, vec![1])).unwrap();
        let d = view(&thing(10, 4, Goal::Go(Cell { x: 1, y: 1 }), vec![])).unwrap();
        out.clear();
        diff("", &c, &d, 10, &mut out);
        assert_eq!(out[0].path, "goal");
        assert_eq!(out[0].a, "Idle");
        assert_eq!(out[0].b, "Go {x: 1, y: 1}");
        assert_eq!(out[1].path, "queue");
        assert_eq!(out[1].a, "[1]");
        assert_eq!(out[1].b, "[]");
        // A struct key of a map is shown as text.
        assert_eq!(
            view(&BTreeMap::from([(Cell { x: 1, y: 2 }.x, 1)]))
                .unwrap()
                .show(),
            "{1: 1}"
        );
        out.clear();
        diff("", &a, &a, 10, &mut out);
        assert!(out.is_empty());
    }
}
