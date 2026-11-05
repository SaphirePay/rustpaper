// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_end_tick(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("core_end_tick") {
        let payload = serde_json::json!({ "done": ({
    if game.dim == Dim::End && let Some(active) = crate::game::paper_fam_core::take_portal_order() {
        let origin = game.exit_portal().unwrap_or([0, game.podium_level().unwrap_or(61), 0]);
        let _ = game.place_podium(origin, active);
    }
    0
}) });
        crate::game::paper_bridge::on_trigger(game, "core_end_tick", payload);
    }
}
