// Written by tools/gen_bundle.py.
fn paper_hook_entity_leave_level(game: &mut crate::game::Game<'_>, dimension: &str, entity: u32, kind: &str) {
    crate::game::paper_events::on_entity_leave(game, dimension, entity, kind);
}
