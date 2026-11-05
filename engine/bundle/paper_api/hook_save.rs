// Written by tools/gen_bundle.py.
fn paper_hook_save(game: &mut crate::game::Game<'_>, wait: bool) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::paper_events::save_begins(game, wait) {
        return ::core::ops::ControlFlow::Continue(());
    }
    let done = game.save_game_with(wait);
    crate::game::paper_events::save_ended(game);
    ::core::ops::ControlFlow::Break(done)
}
