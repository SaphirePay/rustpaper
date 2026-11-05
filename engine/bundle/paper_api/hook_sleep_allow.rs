// Written by tools/gen_bundle.py.
fn paper_hook_sleep_allow(game: &mut crate::game::Game<'_>, pos: [i32; 3]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_events::on_try_sleep(game, pos) {
        ::core::ops::ControlFlow::Continue(())
    } else {
        ::core::ops::ControlFlow::Break(())
    }
}
