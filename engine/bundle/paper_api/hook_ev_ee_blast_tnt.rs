// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_blast_tnt(game: &mut crate::game::Game<'_>) {
    // `note_tnt` notes where each primed TNT is for the blasts that follow; listeners only pass `ee_why`, so asking each plugin every tick (the whole list of primed TNT) was work for nothing.
    if crate::game::paper_bridge::hears("ee_blast_tnt") {
        let _ = crate::game::paper_fam_ev_entity::note_tnt(game.tnts.iter().map(|tnt| (tnt.id, [tnt.position.x, tnt.position.y, tnt.position.z])).collect());
    }
}
