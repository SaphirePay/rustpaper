// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_bolt(game: &mut crate::game::Game<'_>, at: [f64; 3], visual_only: bool) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_bolt") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "at": at, "visual_only": visual_only, "cause": crate::game::paper_fam_ev_world::bolt_cause(), "bolt": game.bolt_ahead(at, visual_only) });
        if crate::game::paper_bridge::on_refusable(game, "ew_bolt", payload) {
            return ::core::ops::ControlFlow::Break(game.drop_bolt_ahead());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
