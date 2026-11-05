// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_bell_hit(game: &mut crate::game::Game<'_>, p: &Projectile, pos: [i32; 3], face: [i32; 3]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_bell_hit") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": pos, "face": face, "entity": p.id, "shot": crate::game::paper_fam_ev_world::bell_shot(game, p.owner, p.guest) });
        if crate::game::paper_bridge::on_refusable(game, "ew_bell_hit", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
