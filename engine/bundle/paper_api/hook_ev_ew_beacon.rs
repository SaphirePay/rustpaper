// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_beacon(game: &mut crate::game::Game<'_>, p: [i32; 3], b: &Beacon) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_beacon") {
        let payload = serde_json::json!({ "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": p, "primary": b.primary.as_deref(), "secondary": b.secondary.as_deref(), "levels": b.levels });
        if crate::game::paper_bridge::on_refusable(game, "ew_beacon", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
