// Written by tools/gen_bundle.py.
fn paper_hook_living_incoming_damage(entity: i32, kind: &str, damage_type: &str, causing: i32, direct: i32, amount: &mut f32, canceled: &mut bool) {
    crate::game::paper_events::on_incoming_damage(entity, kind, damage_type, causing, direct, amount, canceled);
}
