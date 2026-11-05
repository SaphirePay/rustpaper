// Written by tools/gen_bundle.py.
fn paper_hook_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    crate::game::paper_events::on_left(game, player, name);
}
