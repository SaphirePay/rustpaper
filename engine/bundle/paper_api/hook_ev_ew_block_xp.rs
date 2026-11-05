// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_block_xp(game: &mut crate::game::Game<'_>, name: &str) -> ::core::ops::ControlFlow<u32> {
    if crate::game::paper_bridge::hears("ew_block_xp") {
        let payload = serde_json::json!({ "uuid": null, "xp_block": name });
        if crate::game::paper_bridge::on_refusable(game, "ew_block_xp", payload) {
            return ::core::ops::ControlFlow::Break(crate::game::paper_fam_ev_world::take_xp());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
