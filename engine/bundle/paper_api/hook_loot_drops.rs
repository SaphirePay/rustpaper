// Written by tools/gen_bundle.py.
fn paper_hook_loot_drops(game: &mut crate::game::Game<'_>, context: &crate::game::native_hooks::NativeLootDrops<'_>, drops: &mut Vec<horizon_client::items::ItemStack>) {
    crate::game::paper_events::on_loot_drops(game, context, drops);
}
