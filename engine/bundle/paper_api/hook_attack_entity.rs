// Written by tools/gen_bundle.py.
fn paper_hook_attack_entity(game: &mut crate::game::Game<'_>, entity: i32, kind: &str, pos: [f64; 3], canceled: &mut bool) {
    crate::game::paper_events::on_attack_entity(game, entity, kind, pos, canceled);
}
