// Written by tools/gen_bundle.py.
fn paper_hook_ev_sulfur_split(game: &mut crate::game::Game<'_>, m: &Mob) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("sulfur_split") {
        let payload = serde_json::json!({ "entity": m.id });
        if crate::game::paper_bridge::on_refusable(game, "sulfur_split", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
