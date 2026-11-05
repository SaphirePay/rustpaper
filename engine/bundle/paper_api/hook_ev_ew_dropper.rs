// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_dropper(game: &mut crate::game::Game<'_>, pos: Pos, facing: Direction, stack: &ItemStack) -> ::core::ops::ControlFlow<Option<ItemStack>> {
    if crate::game::paper_bridge::hears("ew_dropper") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "pos": [pos.0, pos.1, pos.2], "slot": crate::game::paper_fam_ev_world::dispense_slot(), "stack": crate::game::paper_query::stack_json(stack), "velocity": if matches!(&*stack.item, "minecraft:arrow" | "minecraft:tipped_arrow" | "minecraft:spectral_arrow" | "minecraft:egg" | "minecraft:blue_egg" | "minecraft:brown_egg" | "minecraft:snowball" | "minecraft:experience_bottle" | "minecraft:splash_potion" | "minecraft:lingering_potion" | "minecraft:firework_rocket" | "minecraft:fire_charge" | "minecraft:wind_charge") { step(facing).map(f64::from) } else { let mut r = game.world.random.clone(); let s = step(facing); let pow = r.next_double() * 0.1 + 0.2; let spread = 0.017_227_5 * 6.0; [r.triangle(f64::from(s[0]) * pow, spread), r.triangle(0.2, spread), r.triangle(f64::from(s[2]) * pow, spread)] } });
        if crate::game::paper_bridge::on_refusable(game, "ew_dropper", payload) {
            return ::core::ops::ControlFlow::Break(match crate::game::paper_fam_ev_world::take_result() { Some(item) => { game.dispense_default(pos, facing, item); let mut left = stack.clone(); left.count -= 1; (left.count > 0).then_some(left) } None => Some(stack.clone()) });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
