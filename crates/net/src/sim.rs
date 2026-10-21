//! What the lockstep session drives: a game it can give orders to, step and
//! hash.
//!
//! [`glasnost_ai::game::Game`] is the game every peer runs: the world plus
//! our AI for the computer slots, which thinks inside [`Simulation::step`]
//! on every peer, so its orders never cross the network. A person's orders
//! reach it through [`Game::issue`](glasnost_ai::game::Game::issue), the
//! same path the client and the headless runner use, and are recorded in its
//! replay.

use glasnost_ai::game::Game;
use glasnost_sim::orders::Order;
use glasnost_sim::world::HouseId;

use crate::wire::Hash;

/// A game the session can drive.
pub trait Simulation {
    /// Issues a person's order in a house's name in the current tick.
    /// Returns whether the simulation took it (a refused order is simply not
    /// applied, on every peer alike).
    fn issue(&mut self, house: HouseId, order: Order) -> bool;
    /// Runs one tick.
    fn step(&mut self);
    /// The state hash after the last tick run.
    fn state_hash(&self) -> Hash;
}

impl Simulation for Game {
    fn issue(&mut self, house: HouseId, order: Order) -> bool {
        Game::issue(self, house, order)
    }

    fn step(&mut self) {
        Game::step(self);
    }

    fn state_hash(&self) -> Hash {
        hash_from_hex(&Game::state_hash(self)).expect("the game's state hash is 64 hex digits")
    }
}

/// The bytes of a hexadecimal SHA-256 digest.
pub fn hash_from_hex(s: &str) -> Option<Hash> {
    let b = s.as_bytes();
    // nocite: a SHA-256 digest is 64 hex digits
    if b.len() != 64 {
        return None;
    }
    let digit = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10), // nocite: hexadecimal digits
            b'A'..=b'F' => Some(c - b'A' + 10), // nocite: hexadecimal digits
            _ => None,
        }
    };
    let mut out = [0u8; 32];
    for (i, o) in out.iter_mut().enumerate() {
        *o = (digit(b[2 * i])? << 4) | digit(b[2 * i + 1])?; // nocite: two hex digits per byte
    }
    Some(out)
}

/// A hash as hexadecimal text.
pub fn hex(h: &Hash) -> String {
    h.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}"); // nocite: hexadecimal text of a digest
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips() {
        let mut h = [0u8; 32];
        for (i, b) in h.iter_mut().enumerate() {
            *b = (i * 37) as u8;
        }
        assert_eq!(hash_from_hex(&hex(&h)), Some(h));
        assert_eq!(hash_from_hex("00"), None);
        assert_eq!(hash_from_hex(&"g".repeat(64)), None);
    }
}
