// Written by tools/gen_bundle.py.
fn paper_hook_ev_ee_portal(game: &mut crate::game::Game<'_>, id: u32, entry: BlockPos) -> ::core::ops::ControlFlow<()> {
    // Every event on this hook answers from `ee_refused` alone (`verdict` first: tools' echoes check), so its listeners never ran here; the round trip to each plugin is gone.
    if crate::game::paper_bridge::hears("ee_portal") && ({ let body = game.dropped.iter().find(|e| e.id == id).map(|e| (e.position, 0.25_f32, 0.25_f32)).or_else(|| game.mobs.iter().find(|m| m.id == id).map(|m| (m.position, m.width(), m.height()))).or_else(|| game.boats.iter().find(|b| b.id == id).map(|b| (b.position, super::boats::WIDTH as f32, super::boats::HEIGHT as f32))).or_else(|| game.carts.iter().find(|c| c.id == id).map(|c| (c.position, super::minecarts::WIDTH as f32, super::minecarts::HEIGHT as f32))); match body { Some((at, width, height)) => { let plan = game.entity_plan(entry, at, width, height, true); crate::game::paper_fam_ev_entity::portal(game, id, entry, plan) }, None => false } }) {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
