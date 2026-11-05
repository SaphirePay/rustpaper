// Written by tools/gen_bundle.py.
fn paper_hook_mob_killed(game: &mut crate::game::Game<'_>, player: u128, entity: u32, kind: &str) {
    crate::game::paper_events::on_mob_killed(game, player, entity, kind);
}
