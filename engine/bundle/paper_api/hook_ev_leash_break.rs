// Written by tools/gen_bundle.py.
fn paper_hook_ev_leash_break(game: &mut crate::game::Game<'_>, id: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("leash_break") {
        let payload = serde_json::json!({ "entity": id });
        if crate::game::paper_bridge::on_refusable(game, "leash_break", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
