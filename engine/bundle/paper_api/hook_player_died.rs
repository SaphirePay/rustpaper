// Written by tools/gen_bundle.py.
fn paper_hook_player_died(game: &mut crate::game::Game<'_>, player: u128, cause: Option<&str>) {
    crate::game::paper_events::on_player_died(game, player, cause);
}
