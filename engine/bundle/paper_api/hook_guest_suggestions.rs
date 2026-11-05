// Written by tools/gen_bundle.py.
fn paper_hook_guest_suggestions(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, payload: &[u8]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::on_suggestions(game, host, conn, payload) {
        ::core::ops::ControlFlow::Break(())
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
