// Written by tools/gen_bundle.py.
fn paper_hook_on_equip(game: &mut crate::game::Game<'_>, slot: &str, new: Option<&horizon_client::items::ItemStack>) {
    crate::game::paper_events::on_equip(game, slot, new);
}
