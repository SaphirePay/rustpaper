// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_portal(game: &mut crate::game::Game<'_>, id: u32, entry: BlockPos) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ee_portal") {
        let payload = serde_json::json!({ "ee_refused": ({ let body = game.dropped.iter().find(|e| e.id == id).map(|e| (e.position, 0.25_f32, 0.25_f32)).or_else(|| game.mobs.iter().find(|m| m.id == id).map(|m| (m.position, m.width(), m.height()))).or_else(|| game.boats.iter().find(|b| b.id == id).map(|b| (b.position, super::boats::WIDTH as f32, super::boats::HEIGHT as f32))).or_else(|| game.carts.iter().find(|c| c.id == id).map(|c| (c.position, super::minecarts::WIDTH as f32, super::minecarts::HEIGHT as f32))); match body { Some((at, width, height)) => { let plan = game.entity_plan(entry, at, width, height, true); crate::game::paper_fam_ev_entity::portal(game, id, entry, plan) }, None => false } }) });
        if crate::game::paper_bridge::on_refusable(game, "ee_portal", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
