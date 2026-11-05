// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_unknown_command(game: &mut crate::game::Game<'_>, e: &CommandError, command: &str) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_unknown_command") {
        let payload = serde_json::json!({ "uuid": if game.lan_acting.is_some() { serde_json::json!(format!("{:032x}", game.native_acting_id())) } else { serde_json::Value::Null }, "line": command, "message": crate::game::paper_fam_ev_world::command_failed(e.message.plain(), e.input.as_deref(), e.cursor) });
        if crate::game::paper_bridge::on_refusable(game, "ew_unknown_command", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
