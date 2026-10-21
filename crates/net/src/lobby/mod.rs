//! The network lobby (M9 N2): rooms, their setup and the start, with no
//! I/O.
//!
//! A person creates a room on the relay and gets a room code; others join
//! by the code. The room holds the client's own skirmish setup
//! ([`glasnost_ui::setup::Setup`], the values of S-0412 claim 1) with one
//! member of the room on each person's row, the map by its content hash,
//! and who is ready. The host edits the setup, each person sets the
//! choices of their own row and says ready, and the host starts. Starting
//! gives every member the same [`Begin`]: the values N1's session starts
//! from ([`crate::StartInfo`]), which each member completes with its own
//! rules (the setup hash) and checks against the others in N1's handshake.
//!
//! - [`wire`]: the lobby's messages ([`ToRelay`], [`FromRelay`]) and their
//!   encoding.
//! - [`room`]: one room's state machine ([`Room`]): who may change what,
//!   the choice rules of S-0412 and S-0009 the room enforces, readiness and
//!   the start.
//!
//! The relay (`glasnost-relay`) keeps the rooms and moves the bytes; the
//! client (N3) drives the same messages and shows [`RoomView`]. Like the
//! lockstep session, nothing here reads a clock, draws host randomness or
//! touches a socket: the relay passes the room code and the seed in. The
//! protocol and the room rules are our own design (docs/development.md,
//! *The relay and lobby (N2)*).

pub mod room;
pub mod wire;

pub use room::{Room, Started};
pub use wire::{
    Begin, Edit, FromRelay, MapChoice, MemberView, Offered, Refusal, RoomInfo, RoomView, ToRelay,
    Value,
};

/// A member of a room: the relay's number for the member's connection.
pub type MemberId = u64;

/// The letters of a room code: digits and capitals without I, L, O and U,
/// so that a code read aloud or typed is not mistaken.
// nocite: our own netcode
pub const CODE_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// The length of a room code: 8 letters of 5 bits, 40 bits drawn from the
/// relay's host randomness.
// nocite: our own netcode
pub const CODE_LEN: usize = 8;

/// The room code of `bytes` (the relay's host randomness): each byte picks
/// one letter by its low five bits.
pub fn code_from_bytes(bytes: &[u8; CODE_LEN]) -> String {
    bytes
        .iter()
        .map(|b| char::from(CODE_ALPHABET[usize::from(b & 31)])) // nocite: our own netcode (five bits per letter)
        .collect()
}

/// A typed room code in its standard form: spaces and dashes left out,
/// capitals, O read as 0 and I or L as 1. `None` when it is not a code.
pub fn normalise_code(typed: &str) -> Option<String> {
    let mut out = String::with_capacity(CODE_LEN);
    for c in typed.chars() {
        let c = match c.to_ascii_uppercase() {
            ' ' | '-' => continue,
            'O' => '0',
            'I' | 'L' => '1',
            c => c,
        };
        if !c.is_ascii() || !CODE_ALPHABET.contains(&(c as u8)) {
            return None;
        }
        out.push(c);
    }
    (out.len() == CODE_LEN).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_use_every_letter_and_normalise() {
        let mut seen = std::collections::BTreeSet::new();
        for b in 0..=255u8 {
            let c = code_from_bytes(&[b; CODE_LEN]);
            assert_eq!(c.len(), CODE_LEN);
            assert_eq!(normalise_code(&c), Some(c.clone()));
            seen.insert(c);
        }
        assert_eq!(seen.len(), 32);
        assert_eq!(normalise_code("ab2c-d3ef"), Some("AB2CD3EF".into()));
        assert_eq!(
            normalise_code("o1il 0000"),
            Some("0111 0000".replace(' ', ""))
        );
        assert_eq!(normalise_code("ABCDEFG"), None);
        assert_eq!(normalise_code("ABCDEFGHU"), None);
        assert_eq!(normalise_code("ABCDEFGÜ"), None);
    }
}
