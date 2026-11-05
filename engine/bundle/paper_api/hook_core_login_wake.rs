// Written by tools/gen_bundle.py.
fn paper_hook_core_login_wake(game: &mut crate::game::Game<'_>, paused: bool) -> ::core::ops::ControlFlow<Result<()>> {
    if paused && horizon_client::paper_hub::logins_waiting() {
        return ::core::ops::ControlFlow::Break(game.tick_server(false));
    }
    ::core::ops::ControlFlow::Continue(())
}
