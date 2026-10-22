//! The multiplayer screen (our own layout): the person's name, the relay's
//! address and a room code, then host a room or join one. What the relay
//! answers is the client's (`glasnost-app`); this screen only edits the
//! three fields and reports the buttons.

use crate::draw::{DrawList, colour};
use crate::geom::{Point, Rect};
use crate::widget::{self, Targets};

/// A text field of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetField {
    /// The name the room shows.
    Name,
    /// The relay's address.
    Relay,
    /// The code of the room to join.
    Code,
}

/// A target of the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetHit {
    /// Focus a field.
    Field(NetField),
    /// Create a room on the relay.
    Host,
    /// Join the room of the code.
    Join,
    /// Rejoin the network game this client was in (M9 N5).
    Reconnect,
    /// Back to the main menu.
    Back,
}

/// The longest name typed, in characters (the relay keeps 24).
// nocite: our own netcode (the relay's name limit)
pub const NAME_CHARS: usize = 24;
/// The longest relay address typed, in characters.
// nocite: our own layout
pub const RELAY_CHARS: usize = 200;
/// The longest room code typed, in characters (8 letters and some dashes).
// nocite: our own netcode (an 8-letter code with separators)
pub const CODE_CHARS: usize = 12;

/// The screen's fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetForm {
    /// The person's name.
    pub name: String,
    /// The relay's address.
    pub relay: String,
    /// The room code typed.
    pub code: String,
    /// The field typing goes to.
    pub focus: Option<NetField>,
    /// The room code of a network game this client can rejoin, shown with
    /// a **Reconnect** button (M9 N5).
    pub rejoin: Option<String>,
    /// The client's build id, shown at the foot of the screen so players
    /// can compare builds (M9 N8b); empty shows nothing.
    pub build: String,
}

impl NetForm {
    /// The fields at their start: a plain name, `relay`, no code.
    pub fn new(relay: &str) -> Self {
        Self {
            name: "Player".into(),
            relay: relay.into(),
            code: String::new(),
            focus: None,
            rejoin: None,
            build: String::new(),
        }
    }

    fn field(&mut self, f: NetField) -> (&mut String, usize) {
        match f {
            NetField::Name => (&mut self.name, NAME_CHARS),
            NetField::Relay => (&mut self.relay, RELAY_CHARS),
            NetField::Code => (&mut self.code, CODE_CHARS),
        }
    }

    /// Typed text goes into the focused field: printable characters only,
    /// up to the field's length.
    pub fn type_text(&mut self, text: &str) {
        let Some(f) = self.focus else {
            return;
        };
        let (s, max) = self.field(f);
        for c in text.chars().filter(|c| !c.is_control()) {
            if s.chars().count() < max {
                s.push(c);
            }
        }
    }

    /// Backspace in the focused field.
    pub fn backspace(&mut self) {
        if let Some(f) = self.focus {
            self.field(f).0.pop();
        }
    }

    /// Draws the screen and lays out its targets. `status` is a line about
    /// the connection (`bad` for a refusal or a failure); while `busy`
    /// (connecting), the buttons wait.
    pub fn view(
        &self,
        size: (i32, i32),
        status: &str,
        bad: bool,
        busy: bool,
        hover: Option<Point>,
    ) -> (DrawList, Targets<NetHit>) {
        let mut d = DrawList::new();
        let mut t = Targets::new();
        let (w, h) = size;
        d.fill(Rect::new(0, 0, w, h), colour::BACKDROP);
        // nocite: our own layout (a row more for the reconnect button)
        let tall = if self.rejoin.is_some() { 274 } else { 244 };
        let outer = Rect::new(
            (w - 460).max(0) / 2,
            (h - tall).max(0) / 2,
            460.min(w),
            tall.min(h),
        );
        widget::panel(&mut d, outer, Some("MULTIPLAYER"));
        let hot = |r: Rect| hover.is_some_and(|p| r.contains(p));
        let x0 = outer.x + 10;
        let mut y = outer.y + 28;
        for (f, label, value) in [
            (NetField::Name, "Your name", &self.name),
            (NetField::Relay, "Relay address", &self.relay),
            (NetField::Code, "Room code", &self.code),
        ] {
            let r = Rect::new(x0, y, 440, 18);
            let b = widget::text_field(&mut d, r, label, value, self.focus == Some(f), hot(r));
            t.push(b, NetHit::Field(f));
            y += 26;
        }
        d.text_left(
            Rect::new(x0, y, 440, 12),
            "Host a room and give its code to the others, or type a code and join.",
            colour::DIM,
        );
        y += 22;
        let named = !self.name.trim().is_empty() && !self.relay.trim().is_empty();
        let host = Rect::new(x0, y, 140, 22);
        let join = Rect::new(x0 + 150, y, 140, 22);
        let back = Rect::new(outer.right() - 110, y, 100, 22);
        let can_host = named && !busy;
        let can_join = can_host && !self.code.trim().is_empty();
        widget::button(&mut d, host, "Host a room", hot(host), can_host);
        widget::button(&mut d, join, "Join", hot(join), can_join);
        widget::button(&mut d, back, "Back", hot(back), true);
        if can_host {
            t.push(host, NetHit::Host);
        }
        if can_join {
            t.push(join, NetHit::Join);
        }
        t.push(back, NetHit::Back);
        if let Some(code) = &self.rejoin {
            y += 30;
            let r = Rect::new(x0, y, 290, 22);
            let can = !busy && !self.relay.trim().is_empty();
            widget::button(&mut d, r, &format!("Reconnect to game {code}"), hot(r), can);
            if can {
                t.push(r, NetHit::Reconnect);
            }
        }
        d.text_left(
            Rect::new(x0, outer.bottom() - 26, 440, 14),
            status,
            if bad { colour::BAD } else { colour::TEXT },
        );
        if !self.build.is_empty() {
            d.text_left(
                Rect::new(x0, outer.bottom() - 13, 440, 12),
                &format!("Build {}", self.build),
                colour::DIM,
            );
        }
        (d, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_take_typing_and_buttons_need_their_values() {
        let mut f = NetForm::new("ws://127.0.0.1:8787/");
        let (_, t) = f.view((800, 600), "", false, false, None);
        assert!(t.rect_of(&NetHit::Host).is_some());
        assert!(t.rect_of(&NetHit::Join).is_none(), "no code yet");
        f.focus = Some(NetField::Code);
        f.type_text("ab2c-d3ef\u{8}");
        assert_eq!(f.code, "ab2c-d3ef");
        f.backspace();
        assert_eq!(f.code, "ab2c-d3e");
        f.type_text("FGHIJK");
        assert_eq!(f.code.chars().count(), CODE_CHARS);
        let (_, t) = f.view((800, 600), "", false, false, None);
        let j = t.rect_of(&NetHit::Join).expect("join with a code");
        assert_eq!(t.hit(j.centre()), Some(NetHit::Join));
        let name = t.rect_of(&NetHit::Field(NetField::Name)).unwrap();
        assert_eq!(t.hit(name.centre()), Some(NetHit::Field(NetField::Name)));
        // Busy: no button but Back.
        let (_, t) = f.view((800, 600), "Connecting", false, true, None);
        assert!(t.rect_of(&NetHit::Host).is_none());
        assert!(t.rect_of(&NetHit::Back).is_some());
    }

    #[test]
    fn the_build_is_shown_at_the_foot() {
        let mut f = NetForm::new("ws://127.0.0.1:8787/");
        let (d, _) = f.view((800, 600), "", false, false, None);
        assert!(!d.texts().iter().any(|s| s.starts_with("Build ")));
        f.build = "glasnost 0.0.0 (abc123def)".into();
        let (d, _) = f.view((800, 600), "", false, false, None);
        assert!(
            d.texts()
                .iter()
                .any(|s| s.contains("Build glasnost 0.0.0 (abc123def)"))
        );
    }

    #[test]
    fn a_game_to_rejoin_shows_a_reconnect_button() {
        let mut f = NetForm::new("ws://127.0.0.1:8787/");
        let (_, t) = f.view((800, 600), "", false, false, None);
        assert!(t.rect_of(&NetHit::Reconnect).is_none());
        f.rejoin = Some("ABCD2345".into());
        let (d, t) = f.view((800, 600), "", false, false, None);
        let r = t.rect_of(&NetHit::Reconnect).expect("a reconnect button");
        assert_eq!(t.hit(r.centre()), Some(NetHit::Reconnect));
        assert!(
            d.texts()
                .iter()
                .any(|s| s.contains("Reconnect to game ABCD2345"))
        );
        let (_, t) = f.view((800, 600), "", false, true, None);
        assert!(t.rect_of(&NetHit::Reconnect).is_none(), "not while busy");
    }
}
