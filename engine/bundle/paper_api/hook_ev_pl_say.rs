// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_say(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32, command: &str) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("pl_say") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::say_gate(game, host, conn, command, |g, h, line| g.guest_message_command(h, conn, line, None)) } else { 0 } });
        if crate::game::paper_bridge::on_refusable(game, "pl_say", payload) {
            return ::core::ops::ControlFlow::Break(false);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
