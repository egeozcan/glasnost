//! Rules patch layers of our own that ship with the engine (PLAN.md 4.6, D6).
//!
//! One so far: *Pathing and other fixes* (`data/layers/play-fixes.jsonc`, M8
//! step C7). It sets chosen quirk switches to their off position, which each
//! switch's spec states, where the original's position reads as a bug in
//! play. The switches' defaults stay the original's (D6, D8): the layer is an
//! option of a game, applied as a mod patch layer by
//! [`crate::resolve_ruleset`], so it changes `sim_rules_hash` and travels with
//! replays, saves and network starts by that hash. The reference ruleset (the
//! arena, the sweeps, the gates) has no layer unless asked for.
//!
//! The text is compiled in, so a build names its layer: two clients of one
//! build apply the same layer.

use serde::{Deserialize, Serialize};

use crate::jsonc::{self, Node, Value};
use crate::quirks::Quirks;
use crate::resolve::Source;

/// The text of the play-fixes layer.
// nocite: our own data file (each entry cites its claim in the file)
pub const PLAY_FIXES_TEXT: &str = include_str!("../../../../data/layers/play-fixes.jsonc");

/// The play-fixes layer's file, as rules provenance names it.
// nocite: our own file name
pub const PLAY_FIXES_FILE: &str = "data/layers/play-fixes.jsonc";

/// The play-fixes layer's label, as rules provenance names it.
// nocite: our own layer name
pub const PLAY_FIXES_LAYER: &str = "pathing and other fixes";

/// The name a person sees for the play-fixes option (setup screen, network
/// room, reports).
// nocite: our own interface text
pub const PLAY_FIXES_NAME: &str = "Pathing and other fixes";

/// The play-fixes layer as a patch-layer source.
pub fn play_fixes() -> Source<'static> {
    Source {
        layer: PLAY_FIXES_LAYER,
        file: PLAY_FIXES_FILE,
        text: PLAY_FIXES_TEXT,
    }
}

/// Which of our own patch layers a game's rules carry. The default is the
/// reference ruleset: no layer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layers {
    /// *Pathing and other fixes* ([`play_fixes`]).
    pub play_fixes: bool,
}

impl Layers {
    /// No layer: the reference ruleset (the original's behaviour wherever a
    /// switch exists).
    // nocite: our own option (no layer)
    pub const REFERENCE: Layers = Layers { play_fixes: false };

    /// The play-fixes layer on.
    // nocite: our own option (the layer's entries cite their claims in its file)
    pub const PLAY_FIXES: Layers = Layers { play_fixes: true };

    /// The layers' sources, in application order, to follow the converted
    /// base tree in [`crate::resolve_ruleset`].
    pub fn sources(self) -> Vec<Source<'static>> {
        let mut out = Vec::new();
        if self.play_fixes {
            out.push(play_fixes());
        }
        out
    }

    /// One line for reports and lane outputs: "Pathing and other fixes: on"
    /// or "…: off".
    pub fn describe(self) -> String {
        format!(
            "{PLAY_FIXES_NAME}: {}",
            if self.play_fixes { "on" } else { "off" }
        )
    }

    /// The quirk switches of `base` with the layers' switches applied: for a
    /// ruleset built as Rust values rather than resolved from a tree (the
    /// synthetic test rulesets, the sim-gate fixtures). A tree-resolved
    /// ruleset takes the layers through [`Layers::sources`] instead.
    pub fn apply_quirks(self, base: Quirks) -> Result<Quirks, String> {
        let mut q = base;
        for s in self.sources() {
            q = quirks_with(q, &s)?;
        }
        Ok(q)
    }

    /// From an environment value of a local tool: `1`, `on`, `yes` or
    /// `true` turn the play fixes on; anything else (or nothing) keeps the
    /// reference ruleset.
    pub fn from_flag(value: Option<&str>) -> Layers {
        let on = value.is_some_and(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "on" | "yes" | "true"
            )
        });
        Layers { play_fixes: on }
    }
}

/// The `globals.quirks` object of a layer, if it has one.
fn quirks_node(tree: &Node) -> Option<&Node> {
    tree.get("globals")?.get("quirks")
}

/// `base` with the quirk switches `layer` sets in `globals.quirks`, read as
/// the resolve pipeline reads them (strictly: an unknown switch or a value
/// that is not a boolean is an error).
pub fn quirks_with(base: Quirks, layer: &Source) -> Result<Quirks, String> {
    let tree = jsonc::parse(layer.text).map_err(|e| format!("{}: {e:?}", layer.file))?;
    let mut members: Vec<(String, Node)> = base
        .named()
        .into_iter()
        .map(|(n, v)| (n.to_owned(), jsonc::boolean(v)))
        .collect();
    if let Some(Value::Object(set)) = quirks_node(&tree).map(|n| &n.value) {
        for m in set {
            match members.iter_mut().find(|(k, _)| *k == m.key) {
                Some(slot) => slot.1 = m.value.clone(),
                None => members.push((m.key.clone(), m.value.clone())),
            }
        }
    }
    crate::de::from_node(&jsonc::object(members)).map_err(|e| format!("{}: {e}", layer.file))
}

/// The paths a layer sets, as dotted keys (`globals.quirks.name`), in source
/// order: for checking what a layer touches.
pub fn paths(layer: &Source) -> Result<Vec<String>, String> {
    fn walk(node: &Node, prefix: &str, out: &mut Vec<String>) {
        match &node.value {
            Value::Object(members) if !members.is_empty() => {
                for m in members {
                    let p = if prefix.is_empty() {
                        m.key.clone()
                    } else {
                        format!("{prefix}.{}", m.key)
                    };
                    walk(&m.value, &p, out);
                }
            }
            _ => out.push(prefix.to_owned()),
        }
    }
    let tree = jsonc::parse(layer.text).map_err(|e| format!("{}: {e:?}", layer.file))?;
    let mut out = Vec::new();
    walk(&tree, "", &mut out);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The switches the play-fixes layer turns off (M8 step C7). Changing the
    /// layer's list means changing this one, its docs section and its tests.
    const ADMITTED: [&str; 19] = [
        "absorber_stale_contact_kept",
        "aircraft_edge_hold",
        "aircraft_grounded_enter_stall",
        "aircraft_offmap_stop_start",
        "aircraft_ready_mark_outlives_enter",
        "corridor_free_part_ignores_height",
        "corridor_retry_no_fallback",
        "deadlock_no_yield",
        "depot_loss_strands_hover",
        "group_spread_no_detour",
        "harvest_cell_shared",
        "hover_landing_request_stands",
        "path_through_stalled_blockers",
        "rally_point_shared",
        "switched_off_hover_lead",
        "ungranted_super_reads_clock",
        "unload_without_contact_stalls",
        "unreachable_move_dropped",
        "walker_attacks_unhittable_obstacle",
    ];

    // spec: S-0212 claim 13 and Edge cases (quirks.corridor_retry_no_fallback off, quirks.corridor_free_part_ignores_height off); S-0314 Edge cases (quirks.depot_loss_strands_hover off); S-0402 claim 11 and Edge cases (quirks.unload_without_contact_stalls off); S-0605 claim 17 and Edge cases (quirks.absorber_stale_contact_kept off); S-0503 claim 19 and Edge cases (quirks.hover_landing_request_stands off); S-0212 claims 8, 22 and 23 and S-0213 claim 7 (quirks.path_through_stalled_blockers, quirks.unreachable_move_dropped and quirks.group_spread_no_detour off, our design, C7 part 4); S-0417 claims 8 and 10, S-0212 claims 8 and 22 and S-0402 claim 4 (quirks.rally_point_shared, quirks.deadlock_no_yield and quirks.harvest_cell_shared off, our design, C7 part 5); S-0108 claims 11 and 18 and Edge cases (quirks.ungranted_super_reads_clock off); S-0306 claim 3 (b) and Edge cases (quirks.switched_off_hover_lead off); S-0501 claims 4, 10 and 51 and Edge cases (quirks.aircraft_edge_hold off); S-0209 claims 4 and 9 and Edge cases (quirks.walker_attacks_unhittable_obstacle off); S-0501 claim 24 and Edge cases (quirks.aircraft_grounded_enter_stall off) and claim 31 and Edge cases (quirks.aircraft_ready_mark_outlives_enter off); S-0501 claim 6 and Edge cases (quirks.aircraft_offmap_stop_start off): the layer sets exactly these nineteen switches, and only switches
    #[test]
    fn play_fixes_sets_only_the_admitted_switches_off() {
        let mut paths = paths(&play_fixes()).unwrap();
        paths.sort();
        let want: Vec<String> = ADMITTED
            .iter()
            .map(|n| format!("globals.quirks.{n}"))
            .collect();
        assert_eq!(paths, want, "the layer touches only these quirk switches");
        let base = Quirks::default();
        let q = Layers::PLAY_FIXES.apply_quirks(base).unwrap();
        let mut changed: Vec<&str> = base
            .named()
            .iter()
            .zip(q.named())
            .filter(|(a, b)| a.1 != b.1)
            .map(|(a, _)| a.0)
            .collect();
        changed.sort_unstable();
        assert_eq!(changed, ADMITTED);
        assert!(
            q.corridor_split_probe,
            "left on: no pass or fail difference with the retry fallback off"
        );
        assert!(!q.corridor_retry_no_fallback && !q.corridor_free_part_ignores_height);
        assert!(!q.depot_loss_strands_hover);
        // Every switch the layer changes is on by default (the original),
        // so the layer only ever turns the original's behaviour off.
        for n in ADMITTED {
            let d = base.named().into_iter().find(|(k, _)| *k == n).unwrap();
            assert!(d.1, "{n} defaults to on");
        }
    }

    /// The reference ruleset has no layer; applying none changes nothing.
    #[test]
    fn the_reference_has_no_layer() {
        assert!(Layers::default().sources().is_empty());
        assert_eq!(Layers::default(), Layers::REFERENCE);
        let base = Quirks::default();
        assert_eq!(Layers::REFERENCE.apply_quirks(base).unwrap(), base);
        assert_eq!(Layers::PLAY_FIXES.sources().len(), 1);
    }

    /// The layer resolves over a base tree and changes the sim rules hash.
    #[test]
    fn the_layer_resolves_and_changes_the_rules_hash() {
        let base = r#"{"schema": 1}"#;
        let src = Source {
            layer: "base",
            file: "base.jsonc",
            text: base,
        };
        let plain = crate::resolve_ruleset(None, &[src]).unwrap();
        let fixed = crate::resolve_ruleset(None, &[src, play_fixes()]).unwrap();
        assert_ne!(plain.sim_rules_hash, fixed.sim_rules_hash);
        assert_eq!(
            fixed.ruleset.globals.quirks,
            Layers::PLAY_FIXES.apply_quirks(Quirks::default()).unwrap()
        );
        let again = crate::resolve_ruleset(None, &[src, play_fixes()]).unwrap();
        assert_eq!(fixed.sim_rules_hash, again.sim_rules_hash);
    }

    /// A layer naming a switch that does not exist is refused, as the
    /// resolve pipeline refuses it.
    #[test]
    fn an_unknown_switch_is_refused() {
        let bad = Source {
            layer: "bad",
            file: "bad.jsonc",
            text: r#"{"globals": {"quirks": {"no_such_switch": false}}}"#,
        };
        assert!(quirks_with(Quirks::default(), &bad).is_err());
    }

    #[test]
    fn the_flag_reads_on_and_off() {
        assert!(Layers::from_flag(Some("1")).play_fixes);
        assert!(Layers::from_flag(Some(" On ")).play_fixes);
        assert!(!Layers::from_flag(Some("0")).play_fixes);
        assert!(!Layers::from_flag(None).play_fixes);
        assert_eq!(Layers::PLAY_FIXES.describe(), "Pathing and other fixes: on");
    }
}
