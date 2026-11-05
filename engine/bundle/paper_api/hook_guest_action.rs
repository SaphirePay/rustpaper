// Written by tools/gen_bundle.py.
fn paper_hook_guest_action(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, packet: &S) -> ::core::ops::ControlFlow<()> {
    if let S::PlayerAction(a) = packet
        && matches!(a.action, 3 | 4 | 6)
        && crate::game::paper_bridge::on_player_action(game, host, conn, a.action)
    {
        return ::core::ops::ControlFlow::Break(());
    }
    ::core::ops::ControlFlow::Continue(())
}
