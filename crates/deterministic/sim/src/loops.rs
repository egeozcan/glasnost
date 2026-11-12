//! Loops in the links between rules types (#224, #174, #203, #198): our
//! engine's own input validation, not game behaviour.
//!
//! Some rules fields name another type that the game creates from the
//! first: an animation's trailer, a bouncing animation's spawn, bounce and
//! expiry animations, a meteor piece's spawn type, an airburst weapon, a
//! smoke particle's next type. Data whose links lead back to the first type
//! made one tick run for ever or made the number of objects grow without
//! end. At rules build each such graph is split into its loop groups (the
//! strongly connected groups of the link graph that hold a loop), and a link
//! between two types of one group is a *looped link*. An object made through
//! a looped link carries a saved mark and takes no looped link itself, so one
//! creation leads to at most one generation of a loop. Links on no loop, and
//! everything made through them, are unchanged. Q-2996 to Q-2999 ask what the
//! game does with such data.

/// The loop groups of one link graph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Loops {
    /// Each node's loop group; `None` for a node on no loop.
    group: Vec<Option<u32>>,
}

impl Loops {
    /// The loop groups of the graph whose node `v` links to each node of
    /// `links[v]` (a link to a node past the end is ignored).
    pub fn new(links: &[Vec<usize>]) -> Self {
        let n = links.len();
        let unseen = usize::MAX;
        let mut index = vec![unseen; n];
        let mut low = vec![0usize; n];
        let mut on_stack = vec![false; n];
        let mut stack = Vec::new();
        let mut comp = vec![0usize; n];
        let mut next = 0usize;
        let mut comps = 0usize;
        // Tarjan's strongly connected groups, with an explicit work list
        // (a rules file may hold long chains of links).
        for root in 0..n {
            if index[root] != unseen {
                continue;
            }
            index[root] = next;
            low[root] = next;
            next += 1;
            stack.push(root);
            on_stack[root] = true;
            let mut work: Vec<(usize, usize)> = vec![(root, 0)];
            while let Some(top) = work.len().checked_sub(1) {
                let (v, i) = work[top];
                if let Some(&u) = links[v].get(i) {
                    work[top].1 += 1;
                    if u >= n {
                        continue;
                    }
                    if index[u] == unseen {
                        index[u] = next;
                        low[u] = next;
                        next += 1;
                        stack.push(u);
                        on_stack[u] = true;
                        work.push((u, 0));
                    } else if on_stack[u] {
                        low[v] = low[v].min(index[u]);
                    }
                    continue;
                }
                work.pop();
                if let Some(&(p, _)) = work.last() {
                    low[p] = low[p].min(low[v]);
                }
                if low[v] == index[v] {
                    while let Some(x) = stack.pop() {
                        on_stack[x] = false;
                        comp[x] = comps;
                        if x == v {
                            break;
                        }
                    }
                    comps += 1;
                }
            }
        }
        let mut size = vec![0usize; comps];
        for &c in &comp {
            size[c] += 1;
        }
        // A group holds a loop when it has two nodes or more, or its one
        // node links to itself.
        let group = (0..n)
            .map(|v| {
                let c = comp[v];
                (size[c] > 1 || links[v].contains(&v)).then(|| u32::try_from(c).unwrap_or(u32::MAX))
            })
            .collect();
        Self { group }
    }

    /// Whether the link from node `from` to node `to` lies on a loop.
    pub fn looped(&self, from: usize, to: usize) -> bool {
        match (self.group.get(from), self.group.get(to)) {
            (Some(Some(a)), Some(Some(b))) => a == b,
            _ => false,
        }
    }

    /// What an object of node `from`, marked when `marked`, gives the
    /// object of node `to` it makes through the link between them: `None`
    /// when it makes none (a marked object takes no looped link), else the
    /// new object's mark (whether the link is looped). A missing node is on
    /// no loop.
    pub fn link(&self, from: Option<usize>, marked: bool, to: Option<usize>) -> Option<bool> {
        let looped = matches!((from, to), (Some(f), Some(t)) if self.looped(f, t));
        if looped && marked { None } else { Some(looped) }
    }

    /// Whether any node lies on a loop.
    pub fn any(&self) -> bool {
        self.group.iter().any(Option::is_some)
    }

    /// The nodes on a loop, in index order.
    pub fn nodes(&self) -> Vec<usize> {
        self.group
            .iter()
            .enumerate()
            .filter_map(|(i, g)| g.map(|_| i))
            .collect()
    }
}

/// The loop groups of the four link graphs of a ruleset, and a warning for
/// each graph that has any.
#[derive(Clone, Debug, Default)]
pub struct LinkLoops {
    /// Animation types, by registration index: trailer, bounce, expiry and
    /// spawn animations, and the follow-up type (a switch of the same
    /// animation, so a loop through it is found; it is never refused).
    pub anims: Loops,
    /// Debris piece types, by registration index: a meteor piece's spawn
    /// type.
    pub pieces: Loops,
    /// Weapons, by index: the airburst weapon of the weapon's projectile,
    /// when that projectile is an airburst one.
    pub airburst: Loops,
    /// Particle types, by registration index: the next type.
    pub particles: Loops,
    /// One line for each graph that has a loop, naming its types.
    pub warnings: Vec<String>,
}

impl LinkLoops {
    /// The loop groups of the rules built so far.
    pub fn new(
        animations: &[crate::rules::AnimRules],
        pieces: &crate::pieces::PieceRules,
        combat: &crate::combat::data::CombatRules,
        gas: &crate::gas::GasRules,
    ) -> Self {
        let anim_index = |name: &Option<String>| {
            name.as_ref().and_then(|n| {
                animations
                    .iter()
                    .position(|a| a.name.eq_ignore_ascii_case(n))
            })
        };
        let anim_links: Vec<Vec<usize>> = animations
            .iter()
            .map(|a| {
                let b = &a.bounce;
                [&b.trailer, &b.bounce, &b.expire, &b.spawns]
                    .into_iter()
                    .filter_map(anim_index)
                    .chain(a.next_index.map(|i| i as usize))
                    .collect()
            })
            .collect();
        let piece_links: Vec<Vec<usize>> = pieces
            .types
            .iter()
            .map(|t| match t.spawns {
                Some(s) if t.meteor && t.spawn_count > 0 => vec![s as usize],
                _ => Vec::new(),
            })
            .collect();
        let airburst_links: Vec<Vec<usize>> = combat
            .weapons
            .iter()
            .map(|wr| {
                wr.projectile
                    .and_then(|p| combat.projectiles.get(p as usize))
                    .filter(|p| p.airburst)
                    .and_then(|p| p.airburst_weapon)
                    .map(|aw| vec![aw as usize])
                    .unwrap_or_default()
            })
            .collect();
        let particle_links: Vec<Vec<usize>> = gas
            .particles
            .iter()
            .map(|t| t.next.map(|n| vec![n as usize]).unwrap_or_default())
            .collect();
        let out = Self {
            anims: Loops::new(&anim_links),
            pieces: Loops::new(&piece_links),
            airburst: Loops::new(&airburst_links),
            particles: Loops::new(&particle_links),
            warnings: Vec::new(),
        };
        let mut warnings = Vec::new();
        let mut warn = |what: &str, l: &Loops, name: &dyn Fn(usize) -> String| {
            if l.any() {
                let names: Vec<String> = l.nodes().into_iter().map(name).collect();
                warnings.push(format!(
                    "{what} lead back to themselves: {}; each makes one generation of the loop (#224, #174, #203, #198)",
                    names.join(", ")
                ));
            }
        };
        warn("animation types", &out.anims, &|i| {
            animations[i].name.clone()
        });
        warn("debris piece types", &out.pieces, &|i| {
            pieces.types[i].name.clone()
        });
        warn("airburst weapons", &out.airburst, &|i| {
            combat.weapons[i].name.clone()
        });
        warn("particle types", &out.particles, &|i| {
            gas.particles[i].name.clone()
        });
        Self { warnings, ..out }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups() {
        // 0 → 1 → 2 → 0 is a loop; 3 → 3 links itself; 2 → 5 leaves the
        // loop and 4 → 0 enters it, so neither lies on one.
        let l = Loops::new(&[vec![1], vec![2], vec![0, 5], vec![3], vec![0], vec![]]);
        assert!(l.looped(0, 1) && l.looped(1, 2) && l.looped(2, 0));
        assert!(l.looped(3, 3));
        assert!(!l.looped(2, 5) && !l.looped(4, 0) && !l.looped(0, 3));
        assert_eq!(l.nodes(), vec![0, 1, 2, 3]);
        // 2 → 4 → 0 closes a second path into the same group; a link past
        // the end is ignored.
        let l = Loops::new(&[vec![1], vec![2], vec![0, 4], vec![], vec![0], vec![9]]);
        assert!(l.looped(2, 4) && l.looped(4, 0));
        assert_eq!(l.nodes(), vec![0, 1, 2, 4]);
        assert!(!Loops::new(&[vec![1], vec![]]).any());
    }

    #[test]
    fn a_long_chain_needs_no_deep_stack() {
        let n = 200_000;
        let links: Vec<Vec<usize>> = (0..n).map(|i| vec![(i + 1) % n]).collect();
        let l = Loops::new(&links);
        assert!(l.looped(n - 1, 0));
    }
}
