// Written by tools/gen_bundle.py.
fn paper_hook_mob_died(value: (), game: &mut crate::game::Game<'_>, m: &Mob) -> () {
    crate::game::paper_events::on_mob_died(game, m.id, m.kind.id.as_ref());
    value
}
