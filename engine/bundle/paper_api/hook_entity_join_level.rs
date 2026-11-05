// Written by tools/gen_bundle.py.
fn paper_hook_entity_join_level(game: &mut crate::game::Game<'_>, dimension: &str, entity: u32, kind: &str, pos: [f64; 3], loaded_from_disk: bool, canceled: &mut bool) {
    crate::game::paper_events::on_entity_join(game, dimension, entity, kind, pos, loaded_from_disk, canceled);
}
