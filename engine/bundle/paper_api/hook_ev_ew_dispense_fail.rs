// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_dispense_fail(game: &mut crate::game::Game<'_>, id: i32, pos: Pos) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_dispense_fail") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": [pos.0, pos.1, pos.2], "id": id, "empty": crate::game::paper_fam_ev_world::dispenser_empty() });
        if crate::game::paper_bridge::on_refusable(game, "ew_dispense_fail", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
