// Written by tools/gen_bundle.py.
fn paper_hook_ev_rocket_explode(game: &mut crate::game::Game<'_>, r: &Rocket) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("rocket_explode") {
        let payload = serde_json::json!({ "entity": r.id });
        if crate::game::paper_bridge::on_refusable(game, "rocket_explode", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
