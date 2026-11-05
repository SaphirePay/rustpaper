// Written by tools/gen_bundle.py.
fn paper_hook_ev_entity_portal_gate(game: &mut crate::game::Game<'_>, id: u32, entry: BlockPos) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("entity_portal_gate") {
        let payload = serde_json::json!({ "entity": id });
        if crate::game::paper_bridge::on_refusable(game, "entity_portal_gate", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
