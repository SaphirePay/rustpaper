// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_detonate(game: &mut crate::game::Game<'_>, center: [f64; 3], blocks: &mut Vec<[i32; 3]>) {
    // The blast's EntityExplodeEvent went out inside `detonate`; no listener reads this `ee_done` after it (BlockExplodeEvent read it as a blast with no blocks), so every blast cost each plugin a second round trip for nothing.
    if crate::game::paper_bridge::hears("ee_detonate") {
        let _ = crate::game::paper_fam_ev_entity::detonate(game, center, blocks);
    }
}
