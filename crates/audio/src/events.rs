//! What the game client calls every frame: [`Audio`] turns simulation
//! effects, order acknowledgements and interface cues into voices.
//!
//! Sound never feeds back into the simulation. [`Audio`] reads effects and
//! the world, and the one random stream it draws from is the
//! *unsynchronised* stream (S-0202 claim 11), which the caller owns and
//! passes in; the scenario random stream is never touched.

use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::orders::Order;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{ClickResult, selection_response, selection_responses};
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

use crate::bank::SoundBank;
use crate::catalog::SoundCatalog;
use crate::mixer::{Listener, MixParams, Mixer, Placement, Refused, UNITY, VoiceId, VoiceRequest};
use crate::names::{Announcement, SoundNames};
use crate::output::{Output, OutputState};

/// Why a sound did not start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayError {
    /// The catalog does not know the sound name.
    Unknown(String),
    /// The bank has no playable sample for it.
    Missing(String),
    /// The mixer refused the voice.
    Refused(Refused),
}

/// Interface cues the client reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiCue {
    /// The credit counter moved this cycle (S-0416 claim 4).
    CreditTick {
        /// Rising (true) or falling.
        rising: bool,
    },
    /// The local player's radar became available (S-0410 claim 3).
    RadarOn,
    /// The local player's radar became unavailable (S-0410 claim 3).
    RadarOff,
    /// A sidebar click started or queued an item (S-0411 claim 12).
    BuildStarted,
    /// A sidebar click resumed an item (S-0411 claim 12).
    BuildResumed,
    /// A sidebar click held an item (S-0411 claim 12).
    BuildHeld,
    /// A sidebar click cancelled an item (S-0411 claim 12).
    BuildCancelled,
    /// A build click refused for a busy line or a full queue (S-0411
    /// claims 4 and 12, S-0404 claim 3).
    BuildRefused,
    /// A placement refused by its tests (S-0411 claim 10, S-0406).
    PlacementRefused,
    /// A building item became done (S-0411 claims 9 and 12).
    ConstructionComplete,
}

/// The order kinds that have an acknowledgement of their own kind (S-0214
/// claim 14): harvest, attack, move and attack move, enter, capture and
/// unload. `None` for an order with no actor.
fn own_kind(o: &Order) -> bool {
    match o {
        Order::Behaviour {
            behaviour,
            attack_move,
            ..
        } => {
            *attack_move
                || matches!(
                    behaviour,
                    Behaviour::Harvest
                        | Behaviour::Attack
                        | Behaviour::Move
                        | Behaviour::AttackMove
                        | Behaviour::Enter
                        | Behaviour::Capture
                        | Behaviour::Unload
                )
        }
        _ => false,
    }
}

/// The actor of an object order.
fn actor(o: &Order) -> Option<ObjectId> {
    match o {
        Order::Behaviour { actor, .. }
        | Order::Stop { actor }
        | Order::Scatter { actor }
        | Order::Deploy { actor }
        | Order::GuardSpot { actor, .. }
        | Order::Sell { actor }
        | Order::Repair { actor }
        | Order::PowerOff { actor }
        | Order::PowerOn { actor }
        | Order::Primary { actor } => Some(*actor),
        _ => None,
    }
}

fn at(p: Pos) -> Placement {
    Placement::At { x: p.x, y: p.y }
}

/// The audio of a game: mixer, output, samples, catalog and the rules
/// data's sound names.
pub struct Audio {
    mixer: Mixer,
    output: Box<dyn Output>,
    bank: SoundBank,
    catalog: SoundCatalog,
    names: SoundNames,
}

impl Audio {
    /// Audio on an output, with the interim catalog (Q-0260), an empty bank
    /// and no sound names.
    pub fn new(output: Box<dyn Output>) -> Self {
        let mixer = Mixer::new(output.rate(), MixParams::default());
        Self {
            mixer,
            output,
            bank: SoundBank::new(),
            catalog: SoundCatalog::interim(),
            names: SoundNames::default(),
        }
    }

    /// Audio on the platform's default output
    /// ([`crate::output::platform_default`]).
    pub fn platform() -> Self {
        Self::new(crate::output::platform_default())
    }

    /// Sets the sample bank.
    pub fn set_bank(&mut self, bank: SoundBank) {
        self.bank = bank;
    }

    /// The sample bank.
    pub fn bank_mut(&mut self) -> &mut SoundBank {
        &mut self.bank
    }

    /// Sets the catalog.
    pub fn set_catalog(&mut self, catalog: SoundCatalog) {
        self.catalog = catalog;
    }

    /// Sets the rules data's sound names (from
    /// [`SoundNames::from_ruleset`]).
    pub fn set_names(&mut self, names: SoundNames) {
        self.names = names;
    }

    /// The sound names.
    pub fn names(&self) -> &SoundNames {
        &self.names
    }

    /// The mixer (offline rendering, tests, volume).
    pub fn mixer_mut(&mut self) -> &mut Mixer {
        &mut self.mixer
    }

    /// Moves the listener: the view centre in map leptons. Call every frame.
    pub fn set_listener(&mut self, l: Listener) {
        self.mixer.set_listener(l);
    }

    /// Starts or resumes the output; call from a user-input handler.
    pub fn resume(&mut self) {
        self.output.resume();
    }

    /// Feeds the output. Call once per frame, after the frame's events.
    pub fn frame(&mut self) {
        self.output.pump(&mut self.mixer);
    }

    /// The output's state.
    pub fn state(&self) -> OutputState {
        self.output.state()
    }

    /// Plays a sound by name at a placement, with an extra gain
    /// ([`UNITY`] = as the catalog gives it).
    pub fn play_sound(
        &mut self,
        name: &str,
        placement: Placement,
        gain: u32,
    ) -> Result<VoiceId, PlayError> {
        let def = self
            .catalog
            .resolve(name)
            .ok_or_else(|| PlayError::Unknown(name.to_owned()))?;
        let clip = def
            .samples
            .first()
            .and_then(|s| self.bank.clip(s))
            .ok_or_else(|| PlayError::Missing(name.to_owned()))?;
        let gain = ((u64::from(def.gain) * u64::from(gain)) >> 16) as u32;
        self.mixer
            .play(VoiceRequest {
                clip,
                key: name.to_ascii_uppercase(),
                gain,
                placement,
                priority: def.priority,
                limit: def.limit,
            })
            .map_err(PlayError::Refused)
    }

    fn play_opt(&mut self, name: Option<String>, p: Placement, gain: u32) -> Option<VoiceId> {
        self.play_sound(&name?, p, gain).ok()
    }

    /// Plays an announcement for the local player, where a spec names its
    /// entry (Q-0261 otherwise).
    pub fn announce(&mut self, a: Announcement) -> Option<VoiceId> {
        let entry = self.names.announcement(a)?;
        self.play_sound(entry, Placement::Everywhere, UNITY).ok()
    }

    /// Plays the sounds of a tick's (or frame's) presentation effects, as
    /// heard by the local player's house (`None`: an observer, who hears
    /// no house's own sounds). Returns the voices started.
    pub fn play_effects(&mut self, effects: &[Effect], local: Option<HouseId>) -> usize {
        let mut started = 0;
        for e in effects {
            let v = match e {
                // spec: S-0109 claim 13 (an animation's creation start event plays its type's start sound)
                Effect::Anim { name, pos } => {
                    let s = self.names.anim_report(name).map(str::to_owned);
                    self.play_opt(s, at(*pos), UNITY)
                }
                // No spec gives a shot's or a death's own sound (Q-0264);
                // a detonation's sounds are those of the animations it
                // creates.
                Effect::Shot { .. } | Effect::Killed { .. } | Effect::Detonation { .. } => None,
                // spec: S-0011 claim 32 (play sound, and sound at a waypoint: the sound at the cell's centre; giving a building or terrain object on that cell the sound as its ambient sound is not done yet)
                Effect::Trigger {
                    code: 19 | 99,
                    name,
                    pos,
                    ..
                } => {
                    let p = pos.map_or(Placement::Everywhere, at);
                    self.play_sound(name, p, UNITY).ok()
                }
                // Text messages, speech and radar events are the interface's.
                Effect::Trigger { .. } => None,
                Effect::Sound {
                    sound,
                    of,
                    house,
                    pos,
                } => {
                    // A sound that concerns a house is heard only by that
                    // house's player (S-0408 claims 6, 11 and 12; S-0415
                    // claims 4 and 9).
                    if house.is_some() && *house != local {
                        continue;
                    }
                    let p = pos.map_or(Placement::Everywhere, at);
                    let ts = of
                        .and_then(|t| self.names.of_type(t.kind, t.index))
                        .cloned();
                    match sound {
                        // spec: S-0415 claims 6 and 9 (the type's deploy sound at its position)
                        Sound::Deploy => self.play_opt(ts.and_then(|t| t.deploy), p, UNITY),
                        // spec: S-0415 claim 9 (the pack-up sound, when the type has one)
                        Sound::Packup => self.play_opt(ts.and_then(|t| t.packup), p, UNITY),
                        // spec: S-0408 claims 6 and 12 (the sell sound)
                        Sound::Sell => self.play_opt(self.names.sell.clone(), p, UNITY),
                        // spec: S-0415 claim 4 (the refusal announcement)
                        Sound::CannotDeploy => self.announce(Announcement::CannotDeploy),
                        // spec: S-0408 claim 11 (the unit-sold announcement; its entry: Q-0261)
                        Sound::UnitSold => self.announce(Announcement::UnitSold),
                    }
                }
            };
            started += usize::from(v.is_some());
        }
        started
    }

    /// The spoken acknowledgement of a click's orders (S-0214 claim 14).
    /// `by_local_player` is false for any other house's orders, AI houses
    /// included: they give none. `unsync` is the unsynchronised random
    /// stream (S-0202 claim 11).
    pub fn acknowledge(
        &mut self,
        w: &World,
        click: &ClickResult,
        by_local_player: bool,
        unsync: &mut Rng,
    ) -> Option<VoiceId> {
        if !by_local_player {
            return None;
        }
        let id = click.acknowledge?;
        let order = click.orders.iter().find(|o| actor(o) == Some(id))?;
        let o = w.obj(id)?;
        let n = selection_responses(w, id);
        self.acknowledge_parts(o.kind, o.type_index, own_kind(order), n, unsync)
    }

    /// [`Audio::acknowledge`] for a known speaker: its type, whether the
    /// order has an acknowledgement of its own kind, and the type's number
    /// of selection responses `n`.
    pub fn acknowledge_parts(
        &mut self,
        kind: Kind,
        type_index: u32,
        own: bool,
        n: u32,
        unsync: &mut Rng,
    ) -> Option<VoiceId> {
        if own {
            // The own-kind acknowledgements have no Import mapping
            // (Q-0265): nothing plays and no value is taken.
            return None;
        }
        if n == 0 {
            // spec: S-0214 claim 14, F5 (no value is taken when the type has no selection response)
            return None;
        }
        // spec: S-0214 F5 (one raw value of the unsynchronised stream, i = x mod n)
        let i = selection_response(unsync.raw(), n)?;
        let name = self
            .names
            .of_type(kind, type_index)
            .and_then(|t| t.voice_select.get(i as usize))
            .cloned();
        self.play_opt(name, Placement::Everywhere, UNITY)
    }

    /// Plays an interface cue for the local player.
    pub fn cue(&mut self, cue: UiCue) -> Option<VoiceId> {
        match cue {
            UiCue::CreditTick { rising } => {
                let t = &self.names.credit_ticks;
                // spec: S-0416 claim 4 (fewer than two entries: no tick sound at all)
                if t.len() < 2 {
                    return None;
                }
                // spec: S-0416 claim 4 (first entry rising, second falling)
                let name = t[usize::from(!rising)].clone();
                // spec: S-0416 claim 4 and Constants (tick sound volume one half; linear amplitude, Q-0262)
                self.play_opt(Some(name), Placement::Everywhere, UNITY / 2)
            }
            // spec: S-0410 claim 3 (the radar-on and radar-off sounds)
            UiCue::RadarOn => {
                self.play_opt(self.names.radar_on.clone(), Placement::Everywhere, UNITY)
            }
            UiCue::RadarOff => {
                self.play_opt(self.names.radar_off.clone(), Placement::Everywhere, UNITY)
            }
            // spec: S-0411 claims 9 and 12 (the done announcement; its entry: Q-0261)
            UiCue::ConstructionComplete => self.announce(Announcement::ConstructionComplete),
            // spec: S-0411 claim 12 (a cue plays; which sound is Q-0266)
            UiCue::BuildStarted
            | UiCue::BuildResumed
            | UiCue::BuildHeld
            | UiCue::BuildCancelled
            | UiCue::BuildRefused
            | UiCue::PlacementRefused => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::TypeSounds;
    use crate::output::NullOutput;
    use glasnost_fmt::pcm::Pcm;
    use glasnost_sim::econ::TypeRef;

    const RATE: u32 = 8000;

    /// Audio with one invented sample per name: a constant level, so the
    /// mix shows which sound played and at what gain.
    fn audio(samples: &[(&str, i16)]) -> Audio {
        let mut a = Audio::new(Box::new(NullOutput::new(RATE, "test")));
        let mut bank = SoundBank::new();
        for (n, level) in samples {
            bank.add_pcm(
                n,
                Pcm {
                    sample_rate: RATE,
                    channels: 1,
                    samples: vec![*level; 4],
                },
            );
        }
        a.set_bank(bank);
        a
    }

    fn first_frame(a: &mut Audio) -> [i16; 2] {
        let mut out = [0; 2];
        a.mixer_mut().render(&mut out);
        out
    }

    fn sound(
        sound: Sound,
        of: Option<TypeRef>,
        house: Option<HouseId>,
        pos: Option<Pos>,
    ) -> Effect {
        Effect::Sound {
            sound,
            of,
            house,
            pos,
        }
    }

    fn hut() -> TypeRef {
        TypeRef {
            kind: Kind::Building,
            index: 2,
        }
    }

    fn with_hut(a: &mut Audio, packup: Option<&str>) {
        let mut n = SoundNames::default();
        n.set_type(
            Kind::Building,
            2,
            TypeSounds {
                deploy: Some("Whirr".into()),
                packup: packup.map(Into::into),
                ..TypeSounds::default()
            },
        );
        n.sell = Some("Kaching".into());
        n.set_anim_report("Puff", "Poof");
        a.set_names(n);
    }

    // spec: S-0415 claims 6 and 9, S-0408 claims 6 and 12 (deploy at a position for everyone; sell and pack-up only for the house's player)
    #[test]
    fn structure_sounds_follow_their_house() {
        let mut a = audio(&[("Whirr", 100), ("Kaching", 1000), ("Clunk", 10)]);
        with_hut(&mut a, Some("Clunk"));
        let me = Some(HouseId(1));
        let them = Some(HouseId(2));
        let pos = Some(Pos { x: 0, y: 0, z: 0 });
        let fx = [
            sound(Sound::Deploy, Some(hut()), None, pos),
            sound(Sound::Sell, Some(hut()), them, None),
            sound(Sound::Packup, Some(hut()), them, None),
        ];
        assert_eq!(a.play_effects(&fx, me), 1);
        assert_eq!(first_frame(&mut a), [100, 100]);
        a.mixer_mut().clear();
        let fx = [
            sound(Sound::Sell, Some(hut()), me, None),
            sound(Sound::Packup, Some(hut()), me, None),
        ];
        assert_eq!(a.play_effects(&fx, me), 2);
        assert_eq!(first_frame(&mut a), [1010, 1010]);
    }

    // spec: S-0415 claim 9 (no pack-up sound plays for a type without one)
    #[test]
    fn packup_without_a_sound_is_silent() {
        let mut a = audio(&[("Whirr", 100)]);
        with_hut(&mut a, None);
        let fx = [sound(Sound::Packup, Some(hut()), Some(HouseId(0)), None)];
        assert_eq!(a.play_effects(&fx, Some(HouseId(0))), 0);
    }

    // spec: S-0109 claim 13 (an animation's start sound at its position)
    #[test]
    fn animations_play_their_report() {
        let mut a = audio(&[("Poof", 300)]);
        with_hut(&mut a, None);
        let fx = [
            Effect::Anim {
                name: "PUFF".into(),
                pos: Pos { x: 10, y: 10, z: 0 },
            },
            Effect::Anim {
                name: "Silent".into(),
                pos: Pos { x: 0, y: 0, z: 0 },
            },
        ];
        assert_eq!(a.play_effects(&fx, None), 1);
        assert_eq!(first_frame(&mut a), [300, 300]);
    }

    // spec: S-0415 claim 4 and Import mapping (`EVA_CannotDeployHere`)
    #[test]
    fn the_deploy_refusal_announces_its_entry() {
        let mut a = audio(&[("eva_cannotdeployhere", 50)]);
        let fx = [sound(
            Sound::CannotDeploy,
            Some(hut()),
            Some(HouseId(3)),
            None,
        )];
        assert_eq!(a.play_effects(&fx, Some(HouseId(4))), 0);
        assert_eq!(a.play_effects(&fx, Some(HouseId(3))), 1);
        assert_eq!(first_frame(&mut a), [50, 50]);
    }

    fn grunt(a: &mut Audio) {
        let mut n = SoundNames::default();
        n.set_type(
            Kind::Infantry,
            0,
            TypeSounds {
                voice_select: vec!["Yes0".into(), "Yes1".into(), "Yes2".into()],
                ..TypeSounds::default()
            },
        );
        a.set_names(n);
    }

    // spec: S-0214 claim 14, F5 (one raw value of the unsynchronised stream, response x mod n)
    #[test]
    fn acknowledgement_takes_one_raw_value_and_plays_response_x_mod_n() {
        let mut a = audio(&[("Yes0", 1), ("Yes1", 2), ("Yes2", 3)]);
        grunt(&mut a);
        let mut stream = Rng::new(77);
        let mut copy = stream.clone();
        let expect = copy.raw() % 3;
        a.acknowledge_parts(Kind::Infantry, 0, false, 3, &mut stream)
            .unwrap();
        assert_eq!(stream, copy, "exactly one raw value taken");
        assert_eq!(first_frame(&mut a), [expect as i16 + 1; 2]);
    }

    // spec: S-0214 claim 14 (no value for a type without responses; own-kind orders, Q-0265)
    #[test]
    fn no_value_is_taken_without_responses_or_for_own_kind_orders() {
        let mut a = audio(&[("Yes0", 1)]);
        grunt(&mut a);
        let mut stream = Rng::new(5);
        let before = stream.clone();
        assert!(
            a.acknowledge_parts(Kind::Infantry, 0, false, 0, &mut stream)
                .is_none()
        );
        assert!(
            a.acknowledge_parts(Kind::Infantry, 0, true, 3, &mut stream)
                .is_none()
        );
        assert_eq!(stream, before);
        let mv = Order::Behaviour {
            actor: ObjectId(1),
            behaviour: Behaviour::Guard,
            attack_move: true,
            target: None,
            dest: None,
        };
        assert!(own_kind(&mv));
        assert!(!own_kind(&Order::Deploy { actor: ObjectId(1) }));
    }

    // spec: S-0416 claim 4 (first entry rising, second falling, at half volume; fewer than two entries: none)
    #[test]
    fn credit_ticks() {
        let mut a = audio(&[("Up", 1000), ("Down", 2000)]);
        let mut n = SoundNames::default();
        n.credit_ticks = vec!["Up".into(), "Down".into()];
        a.set_names(n.clone());
        a.cue(UiCue::CreditTick { rising: true }).unwrap();
        assert_eq!(first_frame(&mut a), [500, 500]);
        a.mixer_mut().clear();
        a.cue(UiCue::CreditTick { rising: false }).unwrap();
        assert_eq!(first_frame(&mut a), [1000, 1000]);
        n.credit_ticks.truncate(1);
        a.set_names(n);
        assert!(a.cue(UiCue::CreditTick { rising: true }).is_none());
    }

    // spec: S-0410 claim 3 (radar-on and radar-off sounds)
    #[test]
    fn radar_cues() {
        let mut a = audio(&[("Bleep", 7)]);
        let mut n = SoundNames::default();
        n.radar_on = Some("Bleep".into());
        a.set_names(n);
        assert!(a.cue(UiCue::RadarOn).is_some());
        assert!(a.cue(UiCue::RadarOff).is_none());
        // Interface cues without a named sound stay silent (Q-0266).
        assert!(a.cue(UiCue::BuildStarted).is_none());
    }

    #[test]
    fn play_sound_reports_why_it_did_not_start() {
        let mut a = audio(&[]);
        a.set_catalog(SoundCatalog::explicit());
        assert_eq!(
            a.play_sound("x", Placement::Everywhere, UNITY),
            Err(PlayError::Unknown("x".into()))
        );
        a.set_catalog(SoundCatalog::interim());
        assert_eq!(
            a.play_sound("x", Placement::Everywhere, UNITY),
            Err(PlayError::Missing("x".into()))
        );
    }
}
