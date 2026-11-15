//! Row IC7's second alternative: a deployed infantry in-place deployer
//! whose type fires while deployed gets no-move on a cell out of range of
//! its first weapon (#354; S-0214 table IC row IC7, table M part 1). On the
//! synthetic combat ruleset of the testkit (the WALKER's Rifle reaches 4
//! cells), no game data.

use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::HouseId;

const SIZE: i32 = 32;
const ALPHA: HouseId = HouseId(0);

// spec: S-0214 table IC row IC7 (in its deployed state, c′ attack: no-move when its type does not fire while deployed or the cell centre is out of range of its first weapon) and table M part 1 (no-move on a cell: no order; scenario T10, line IC4, IC6, IC7, VC1)
#[test]
fn issue_354_row_ic7_a_deployed_deploy_fire_infantry_on_a_cell_out_of_range() {
    let mut r = testkit::combat_ruleset();
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.deployer = Some(true);
    walker.deploy_fire = Some(true);
    let mut w = testkit::game_with(&r, SIZE, 7);
    let a = spawn(&mut w, Kind::Infantry, "WALKER", 0, mid(SIZE, 0, 0), 0);
    w.obj_mut(a).unwrap().infantry.as_mut().unwrap().seq =
        Some(glasnost_sim::infantry::Seq::Deployed);
    let ff = Modifiers {
        force_fire: true,
        ..Default::default()
    };
    let near = ClickOn::Cell(mid(SIZE, 1, 0));
    let far = ClickOn::Cell(mid(SIZE, 0, 19));
    assert!(targeting::movable(&w, ALPHA, a), "set-up");
    assert_eq!(
        targeting::action(&w, ALPHA, a, near, ff),
        Action::Attack,
        "control: in range the attack stands"
    );
    let act = targeting::action(&w, ALPHA, a, far, ff);
    let ord = targeting::order(&w, ALPHA, a, act, far, ff, false);
    assert_eq!(
        (act, ord),
        (Action::NoMove, None),
        "row IC7 on a cell out of range"
    );
}
