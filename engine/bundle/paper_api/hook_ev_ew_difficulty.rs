// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_difficulty(game: &mut crate::game::Game<'_>, i: u8) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_difficulty") {
        let payload = serde_json::json!({ "uuid": if game.lan_acting.is_some() { serde_json::json!(format!("{:032x}", game.native_acting_id())) } else { serde_json::Value::Null }, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "difficulty": i });
        if crate::game::paper_bridge::on_refusable(game, "ew_difficulty", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
