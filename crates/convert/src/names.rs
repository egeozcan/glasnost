//! Byte strings as text, without loss.
//!
//! INI names and values are bytes with no encoding (S-0001 claim 1); bytes
//! 128 to 255 are ordinary name bytes. Our rules format is text, so the
//! converter maps each byte below 128 to the character of that code and
//! each byte from 128 to 255 to a character of the private-use block
//! U+F780 to U+F7FF. The mapping is reversible, and because every such
//! character encodes in UTF-8 above every ASCII character and in byte
//! order, sorting the text by its UTF-8 bytes sorts by the original bytes.

/// First code point of the block that stands for bytes 128 to 255.
const HIGH_BASE: u32 = 0xF700; // nocite: our own reversible encoding

/// The text form of `bytes`.
pub fn encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if b < 128 {
                char::from(b)
            } else {
                char::from_u32(HIGH_BASE + u32::from(b)).unwrap_or(char::REPLACEMENT_CHARACTER)
            }
        })
        .collect()
}

/// The bytes a text form stands for, or `None` if it holds a character
/// that no byte maps to.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    text.chars()
        .map(|c| {
            let u = u32::from(c);
            if u < 128 {
                Some(u as u8)
            } else if (HIGH_BASE + 128..=HIGH_BASE + 255).contains(&u) {
                Some((u - HIGH_BASE) as u8)
            } else {
                None
            }
        })
        .collect()
}

/// A readable form for reports (high bytes as `\xNN`).
pub fn show(bytes: &[u8]) -> String {
    let mut s = String::new();
    for &b in bytes {
        if (32..127).contains(&b) {
            s.push(char::from(b));
        } else {
            s.push_str(&format!("\\x{b:02X}"));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_order() {
        let names: [&[u8]; 4] = [b"Alpha", b"Alpha\x80", b"Alpha\xFF", b"Beta"];
        for n in names {
            assert_eq!(decode(&encode(n)).unwrap(), n);
        }
        let mut by_text: Vec<String> = names.iter().map(|n| encode(n)).collect();
        by_text.reverse();
        by_text.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        let back: Vec<Vec<u8>> = by_text.iter().map(|t| decode(t).unwrap()).collect();
        let mut sorted: Vec<Vec<u8>> = names.iter().map(|n| n.to_vec()).collect();
        sorted.sort();
        assert_eq!(back, sorted);
        assert_ne!(encode(b"\xE9"), encode(b"\xC3\xA9"));
        assert!(decode("é").is_none());
        assert_eq!(show(b"a\x01\xFF"), "a\\x01\\xFF");
    }
}
