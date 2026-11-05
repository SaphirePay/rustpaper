// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_riptide(game: &mut crate::game::Game<'_>, ticks: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("pl_riptide") {
        let payload = serde_json::json!({ "x": if true { if crate::game::paper_fam_ev_player::riptide(game.in_water_or_rain(), view_vector(game.yaw, game.pitch), game, ticks) { return ::core::ops::ControlFlow::Break(()); } return ::core::ops::ControlFlow::Continue(()) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_riptide", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
