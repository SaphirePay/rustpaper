// Written by tools/gen_bundle.py.
fn paper_hook_interact_entity(game: &mut crate::game::Game<'_>, entity: i32, kind: &str, stack: Option<&horizon_client::items::ItemStack>, claimed: &mut bool) {
    crate::game::paper_events::on_interact_entity(game, entity, kind, stack, claimed);
}
