//! JSONC reader for our rules format (PLAN.md 4.6): JSON plus `//` and
//! `/* */` comments and trailing commas, parsed into our own tree with byte
//! spans.
//!
//! Numbers never pass through floating point. Number tokens keep their text;
//! only integers are accepted, and a bare decimal (`1.5`, `2e3`) is an error.
//! Decimals in rules data are quoted strings with a unit (`"3.75c"`) and are
//! parsed straight into fixed point by the schema layer (M1).
//!
//! **Milestone M0**: the tree and parser, used to validate the synthetic
//! ruleset. **M1** adds the serde deserializer over this tree, the schema,
//! merge patches, inheritance and the resolve pipeline.

/// Byte range `start..end` in the source text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    /// First byte.
    pub start: usize,
    /// One past the last byte.
    pub end: usize,
}

/// An integer token, kept as its source text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Integer {
    text: String,
}

impl Integer {
    /// The token exactly as written, e.g. `-42`.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The value as `i64`, or `None` if it does not fit.
    pub fn to_i64(&self) -> Option<i64> {
        let (negative, digits) = match self.text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, self.text.as_str()),
        };
        let mut value: i64 = 0;
        for d in digits.bytes() {
            let d = i64::from(d - b'0');
            value = value.checked_mul(10)?;
            value = if negative {
                value.checked_sub(d)?
            } else {
                value.checked_add(d)?
            };
        }
        Some(value)
    }
}

/// A JSON value with spans on every node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// An integer number token.
    Integer(Integer),
    /// A string, escapes resolved.
    String(String),
    /// An array.
    Array(Vec<Node>),
    /// An object; members in source order, keys unique.
    Object(Vec<Member>),
}

/// A value and where it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// The value.
    pub value: Value,
    /// Its source range.
    pub span: Span,
}

/// One `"key": value` pair of an object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    /// The key, escapes resolved.
    pub key: String,
    /// Source range of the key string, quotes included.
    pub key_span: Span,
    /// The value.
    pub value: Node,
}

impl Node {
    /// The member named `key`, if this is an object that has one.
    pub fn get(&self, key: &str) -> Option<&Node> {
        match &self.value {
            Value::Object(members) => members.iter().find(|m| m.key == key).map(|m| &m.value),
            _ => None,
        }
    }

    /// The members, if this is an object.
    pub fn members(&self) -> Option<&[Member]> {
        match &self.value {
            Value::Object(m) => Some(m),
            _ => None,
        }
    }

    /// The string, if this is a string.
    pub fn as_str(&self) -> Option<&str> {
        match &self.value {
            Value::String(s) => Some(s),
            _ => None,
        }
    }
}

/// What went wrong while parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The input ended inside a value, string or comment.
    UnexpectedEnd,
    /// A character that cannot start or continue the current construct.
    UnexpectedChar(char),
    /// A number with a fraction or exponent. Quote decimals with a unit.
    BareDecimal,
    /// A malformed number such as `01` or `-`.
    InvalidNumber,
    /// An unknown or malformed escape sequence.
    InvalidEscape,
    /// A raw control character inside a string.
    ControlCharInString,
    /// An object repeats a key.
    DuplicateKey(String),
    /// Something other than whitespace or comments after the root value.
    TrailingContent,
    /// Nesting deeper than [`MAX_DEPTH`].
    TooDeep,
}

/// A parse error with the source range it refers to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    /// What went wrong.
    pub kind: ErrorKind,
    /// Where.
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::UnexpectedEnd => write!(f, "unexpected end of input"),
            ErrorKind::UnexpectedChar(c) => write!(f, "unexpected character {c:?}"),
            ErrorKind::BareDecimal => write!(
                f,
                "bare decimal number; write decimals as quoted strings with a unit, e.g. \"1.5c\""
            ),
            ErrorKind::InvalidNumber => write!(f, "malformed number"),
            ErrorKind::InvalidEscape => write!(f, "invalid escape sequence"),
            ErrorKind::ControlCharInString => write!(f, "control character in string"),
            ErrorKind::DuplicateKey(k) => write!(f, "duplicate key {k:?}"),
            ErrorKind::TrailingContent => write!(f, "content after the root value"),
            ErrorKind::TooDeep => write!(f, "nesting deeper than {MAX_DEPTH} levels"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Deepest array/object nesting accepted, to bound recursion on hostile input.
pub const MAX_DEPTH: usize = 128;

/// Parses one JSONC document.
pub fn parse(src: &str) -> Result<Node, ParseError> {
    let mut p = Parser { src, pos: 0 };
    p.skip_trivia()?;
    let root = p.value(0)?;
    p.skip_trivia()?;
    if p.pos < src.len() {
        return Err(p.error_here(ErrorKind::TrailingContent));
    }
    Ok(root)
}

/// 1-based line and column (in characters) of a byte offset.
pub fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let before = &src[..offset.min(src.len())];
    let line = before.matches('\n').count() + 1;
    let col = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, col)
}

/// Writes `node` as JSONC text: two-space indentation, members in their
/// order, strings escaped as JSON requires, a trailing newline. Parsing
/// the result gives back an equal tree (spans aside).
pub fn write(node: &Node) -> String {
    let mut out = String::new();
    write_node(node, 0, &mut out);
    out.push('\n');
    out
}

fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            // nocite: JSON escapes every control character below U+0020
            c if u32::from(c) < 0x20 => {
                let _ = std::fmt::Write::write_fmt(out, format_args!("\\u{:04x}", u32::from(c)));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_node(node: &Node, depth: usize, out: &mut String) {
    let indent = |d: usize, out: &mut String| out.extend(std::iter::repeat_n("  ", d));
    match &node.value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Integer(i) => out.push_str(i.text()),
        Value::String(s) => write_string(s, out),
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Array(items) => {
            let simple = items
                .iter()
                .all(|i| !matches!(i.value, Value::Array(_) | Value::Object(_)));
            if simple {
                out.push('[');
                for (n, item) in items.iter().enumerate() {
                    if n > 0 {
                        out.push_str(", ");
                    }
                    write_node(item, depth + 1, out);
                }
                out.push(']');
            } else {
                out.push_str("[\n");
                for item in items {
                    indent(depth + 1, out);
                    write_node(item, depth + 1, out);
                    out.push_str(",\n");
                }
                indent(depth, out);
                out.push(']');
            }
        }
        Value::Object(members) if members.is_empty() => out.push_str("{}"),
        Value::Object(members) => {
            out.push_str("{\n");
            for m in members {
                indent(depth + 1, out);
                write_string(&m.key, out);
                out.push_str(": ");
                write_node(&m.value, depth + 1, out);
                out.push_str(",\n");
            }
            indent(depth, out);
            out.push('}');
        }
    }
}

/// A string node.
pub fn string(s: impl Into<String>) -> Node {
    Node {
        value: Value::String(s.into()),
        span: Span::default(),
    }
}

/// An integer node.
pub fn integer(v: i128) -> Node {
    Node {
        value: Value::Integer(Integer {
            text: v.to_string(),
        }),
        span: Span::default(),
    }
}

/// A boolean node.
pub fn boolean(v: bool) -> Node {
    Node {
        value: Value::Bool(v),
        span: Span::default(),
    }
}

/// An array node.
pub fn array(items: Vec<Node>) -> Node {
    Node {
        value: Value::Array(items),
        span: Span::default(),
    }
}

/// An object node; members keep the given order.
pub fn object(members: impl IntoIterator<Item = (String, Node)>) -> Node {
    Node {
        value: Value::Object(
            members
                .into_iter()
                .map(|(key, value)| Member {
                    key,
                    key_span: Span::default(),
                    value,
                })
                .collect(),
        ),
        span: Span::default(),
    }
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.pos).copied()
    }

    fn error_here(&self, kind: ErrorKind) -> ParseError {
        let end = self.src[self.pos..]
            .chars()
            .next()
            .map_or(self.pos, |c| self.pos + c.len_utf8());
        ParseError {
            kind,
            span: Span {
                start: self.pos,
                end,
            },
        }
    }

    fn unexpected(&self) -> ParseError {
        match self.src[self.pos..].chars().next() {
            Some(c) => self.error_here(ErrorKind::UnexpectedChar(c)),
            None => self.error_here(ErrorKind::UnexpectedEnd),
        }
    }

    fn skip_trivia(&mut self) -> Result<(), ParseError> {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\n' | b'\r') => self.pos += 1,
                Some(b'/') => match self.src.as_bytes().get(self.pos + 1) {
                    Some(b'/') => {
                        self.pos = self.src[self.pos..]
                            .find('\n')
                            .map_or(self.src.len(), |i| self.pos + i);
                    }
                    Some(b'*') => {
                        let start = self.pos;
                        match self.src[self.pos + 2..].find("*/") {
                            Some(i) => self.pos += 2 + i + 2,
                            None => {
                                return Err(ParseError {
                                    kind: ErrorKind::UnexpectedEnd,
                                    span: Span {
                                        start,
                                        end: self.src.len(),
                                    },
                                });
                            }
                        }
                    }
                    _ => return Err(self.unexpected()),
                },
                _ => return Ok(()),
            }
        }
    }

    fn expect_word(&mut self, word: &str) -> Result<(), ParseError> {
        if self.src[self.pos..].starts_with(word) {
            self.pos += word.len();
            Ok(())
        } else {
            Err(self.unexpected())
        }
    }

    fn value(&mut self, depth: usize) -> Result<Node, ParseError> {
        let start = self.pos;
        let value = match self.peek() {
            None => return Err(self.error_here(ErrorKind::UnexpectedEnd)),
            Some(b'{') => self.object(depth)?,
            Some(b'[') => self.array(depth)?,
            Some(b'"') => Value::String(self.string()?),
            Some(b't') => {
                self.expect_word("true")?;
                Value::Bool(true)
            }
            Some(b'f') => {
                self.expect_word("false")?;
                Value::Bool(false)
            }
            Some(b'n') => {
                self.expect_word("null")?;
                Value::Null
            }
            Some(b'-' | b'0'..=b'9') => Value::Integer(self.integer()?),
            Some(_) => return Err(self.unexpected()),
        };
        Ok(Node {
            value,
            span: Span {
                start,
                end: self.pos,
            },
        })
    }

    fn integer(&mut self) -> Result<Integer, ParseError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        let digits_start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        let digits = &self.src[digits_start..self.pos];
        let span = Span {
            start,
            end: self.pos,
        };
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(ParseError {
                kind: ErrorKind::BareDecimal,
                span,
            });
        }
        if digits.is_empty() || (digits.len() > 1 && digits.starts_with('0')) {
            return Err(ParseError {
                kind: ErrorKind::InvalidNumber,
                span,
            });
        }
        Ok(Integer {
            text: self.src[start..self.pos].to_owned(),
        })
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.pos += 1; // opening quote
        let mut out = String::new();
        loop {
            let Some(c) = self.src[self.pos..].chars().next() else {
                return Err(self.error_here(ErrorKind::UnexpectedEnd));
            };
            match c {
                '"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                '\\' => {
                    self.pos += 1;
                    out.push(self.escape()?);
                }
                c if u32::from(c) < 0x20 => {
                    return Err(self.error_here(ErrorKind::ControlCharInString));
                }
                c => {
                    out.push(c);
                    self.pos += c.len_utf8();
                }
            }
        }
    }

    fn escape(&mut self) -> Result<char, ParseError> {
        let Some(c) = self.peek() else {
            return Err(self.error_here(ErrorKind::UnexpectedEnd));
        };
        self.pos += 1;
        Ok(match c {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => {
                let first = self.hex4()?;
                if (0xD800..0xDC00).contains(&first) {
                    // High surrogate: a low one must follow.
                    if !self.src[self.pos..].starts_with("\\u") {
                        return Err(self.error_here(ErrorKind::InvalidEscape));
                    }
                    self.pos += 2;
                    let second = self.hex4()?;
                    if !(0xDC00..0xE000).contains(&second) {
                        return Err(self.error_here(ErrorKind::InvalidEscape));
                    }
                    let combined = 0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00);
                    char::from_u32(combined)
                        .ok_or_else(|| self.error_here(ErrorKind::InvalidEscape))?
                } else {
                    char::from_u32(first)
                        .ok_or_else(|| self.error_here(ErrorKind::InvalidEscape))?
                }
            }
            _ => {
                self.pos -= 1;
                return Err(self.error_here(ErrorKind::InvalidEscape));
            }
        })
    }

    fn hex4(&mut self) -> Result<u32, ParseError> {
        let digits = self
            .src
            .get(self.pos..self.pos + 4)
            .ok_or_else(|| self.error_here(ErrorKind::UnexpectedEnd))?;
        if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(self.error_here(ErrorKind::InvalidEscape));
        }
        let value = u32::from_str_radix(digits, 16)
            .map_err(|_| self.error_here(ErrorKind::InvalidEscape))?;
        self.pos += 4;
        Ok(value)
    }

    fn array(&mut self, depth: usize) -> Result<Value, ParseError> {
        if depth >= MAX_DEPTH {
            return Err(self.error_here(ErrorKind::TooDeep));
        }
        self.pos += 1; // [
        let mut items = Vec::new();
        loop {
            self.skip_trivia()?;
            if self.peek() == Some(b']') {
                self.pos += 1;
                return Ok(Value::Array(items));
            }
            items.push(self.value(depth + 1)?);
            self.skip_trivia()?;
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {}
                _ => return Err(self.unexpected()),
            }
        }
    }

    fn object(&mut self, depth: usize) -> Result<Value, ParseError> {
        if depth >= MAX_DEPTH {
            return Err(self.error_here(ErrorKind::TooDeep));
        }
        self.pos += 1; // {
        let mut members: Vec<Member> = Vec::new();
        loop {
            self.skip_trivia()?;
            match self.peek() {
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Value::Object(members));
                }
                Some(b'"') => {}
                _ => return Err(self.unexpected()),
            }
            let key_start = self.pos;
            let key = self.string()?;
            let key_span = Span {
                start: key_start,
                end: self.pos,
            };
            if members.iter().any(|m| m.key == key) {
                return Err(ParseError {
                    kind: ErrorKind::DuplicateKey(key),
                    span: key_span,
                });
            }
            self.skip_trivia()?;
            if self.peek() != Some(b':') {
                return Err(self.unexpected());
            }
            self.pos += 1;
            self.skip_trivia()?;
            let value = self.value(depth + 1)?;
            members.push(Member {
                key,
                key_span,
                value,
            });
            self.skip_trivia()?;
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {}
                _ => return Err(self.unexpected()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(src: &str) -> ErrorKind {
        parse(src).unwrap_err().kind
    }

    #[test]
    fn parses_comments_and_trailing_commas() {
        let doc = parse("// head\n{ /* a */ \"a\": [1, -2, ], \"b\": {\"c\": null,}, }").unwrap();
        let a = doc.get("a").unwrap();
        let Value::Array(items) = &a.value else {
            panic!("array expected")
        };
        assert_eq!(items.len(), 2);
        assert_eq!(doc.get("b").unwrap().get("c").unwrap().value, Value::Null);
    }

    #[test]
    fn integers_keep_their_text() {
        let doc = parse("[-0, 9223372036854775807, 9223372036854775808]").unwrap();
        let Value::Array(items) = doc.value else {
            panic!()
        };
        let ints: Vec<_> = items
            .iter()
            .map(|n| match &n.value {
                Value::Integer(i) => i.clone(),
                _ => panic!(),
            })
            .collect();
        assert_eq!(ints[0].text(), "-0");
        assert_eq!(ints[0].to_i64(), Some(0));
        assert_eq!(ints[1].to_i64(), Some(i64::MAX));
        assert_eq!(ints[2].to_i64(), None);
        assert_eq!(ints[2].text(), "9223372036854775808");
    }

    #[test]
    fn rejects_bare_decimals() {
        assert_eq!(kind("1.5"), ErrorKind::BareDecimal);
        assert_eq!(kind("{\"x\": 2e3}"), ErrorKind::BareDecimal);
        assert_eq!(kind("[01]"), ErrorKind::InvalidNumber);
        assert_eq!(kind("-"), ErrorKind::InvalidNumber);
    }

    #[test]
    fn rejects_duplicate_keys_with_span() {
        let src = "{\"k\": 1, \"k\": 2}";
        let err = parse(src).unwrap_err();
        assert_eq!(err.kind, ErrorKind::DuplicateKey("k".into()));
        assert_eq!(&src[err.span.start..err.span.end], "\"k\"");
    }

    #[test]
    fn writer_round_trips() {
        let src =
            r#"{"a": [1, -2, {"b": "q\"\\\u0001é"}], "c": {}, "d": [], "e": null, "f": true}"#;
        let node = parse(src).unwrap();
        let text = write(&node);
        let again = parse(&text).unwrap();
        assert_eq!(write(&again), text);
        assert_eq!(
            again.get("a").map(|a| matches!(a.value, Value::Array(_))),
            Some(true)
        );
    }

    #[test]
    fn decodes_escapes() {
        let doc = parse(r#""a\né😀\"""#).unwrap();
        assert_eq!(doc.as_str(), Some("a\né😀\""));
        assert_eq!(kind(r#""\ud83d""#), ErrorKind::InvalidEscape);
        assert_eq!(kind(r#""\q""#), ErrorKind::InvalidEscape);
        assert_eq!(kind("\"a\nb\""), ErrorKind::ControlCharInString);
    }

    #[test]
    fn rejects_trailing_content_and_bad_comments() {
        assert_eq!(kind("{} {}"), ErrorKind::TrailingContent);
        assert_eq!(kind("{} /* open"), ErrorKind::UnexpectedEnd);
        assert_eq!(kind("{} / x"), ErrorKind::UnexpectedChar('/'));
        assert_eq!(kind(""), ErrorKind::UnexpectedEnd);
    }

    #[test]
    fn bounds_nesting() {
        let deep = "[".repeat(MAX_DEPTH + 1);
        assert_eq!(kind(&deep), ErrorKind::TooDeep);
        let ok = format!("{}{}", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
        assert!(parse(&ok).is_ok());
    }

    #[test]
    fn spans_and_positions() {
        let src = "{\n  \"a\": true\n}";
        let doc = parse(src).unwrap();
        let a = doc.get("a").unwrap();
        assert_eq!(&src[a.span.start..a.span.end], "true");
        assert_eq!(line_col(src, a.span.start), (2, 8));
    }
}
