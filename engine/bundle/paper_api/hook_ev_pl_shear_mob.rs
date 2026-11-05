// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_shear_mob(game: &mut crate::game::Game<'_>, target: Option<InteractTarget>) -> ::core::ops::ControlFlow<Result<bool>> {
    if crate::game::paper_bridge::hears("pl_shear_mob") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::shear_gate(game, target.map(|target| target.id), |g, index| { let m = &g.mobs[index]; if !m.alive() || m.is_baby() { return None; } match m.species() { Species::Mooshroom => Some(vec![ItemStack::new(if m.extra.variant == 1 { "minecraft:brown_mushroom" } else { "minecraft:red_mushroom" }, 5)]), Species::Sheep if !m.extra.sheared => { let color = horizon_client::mobs::render::DYE_NAMES[m.extra.color as usize & 15]; Some(world_items!(g).loot_table(&format!("minecraft:shearing/sheep/{color}"), false, None, &mut g.world.random)) } _ => None } }, Ok(true)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_shear_mob", payload) {
            return ::core::ops::ControlFlow::Break(Ok(true));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
