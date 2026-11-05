// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_gateway(game: &mut crate::game::Game<'_>, id: u32, gateway: BlockPos) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_gateway") {
        let payload = serde_json::json!({ "level": crate::game::paper_fam_ev_entity::here(game), "entity": id, "pos": [gateway.x, gateway.y, gateway.z] });
        if crate::game::paper_bridge::on_refusable(game, "ee_gateway", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
