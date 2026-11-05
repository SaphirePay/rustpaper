// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_console(game: &mut crate::game::Game<'_>, state: &mut ServerState, line: &str) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_console") {
        let payload = serde_json::json!({ "uuid": null, "line": line, "remote": crate::game::paper_fam_ev_world::remote_line() });
        if crate::game::paper_bridge::on_refusable(game, "ew_console", payload) {
            return ::core::ops::ControlFlow::Break({ if let Some(line) = crate::game::paper_fam_ev_world::console_line() { game.console_command(state, &line); } });
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
