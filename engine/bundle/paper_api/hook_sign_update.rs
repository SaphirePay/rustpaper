// Written by tools/gen_bundle.py.
fn paper_hook_sign_update(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, p: &play::SignUpdate) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::on_sign_update(game, host, conn, p.pos, &p.lines, p.front) {
        ::core::ops::ControlFlow::Break(())
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
