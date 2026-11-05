// Written by tools/gen_bundle.py.
fn paper_hook_net_from_client(game: &mut crate::game::Game<'_>, player: u128, channel: &str, data: &[u8]) {
    crate::game::paper_events::on_net_from_client(game, player, channel, data);
}
