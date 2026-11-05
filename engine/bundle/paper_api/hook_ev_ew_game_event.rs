// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_game_event(game: &mut crate::game::Game<'_>, event: &'static str, pos: [f64; 3], ctx: &Context) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("ew_game_event") {
        let payload = serde_json::json!({ "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "event": event, "at": pos, "radius": sculk::notification_radius(event), "entity": match ctx.source { Some(Who::Mob(id)) => Some(id), _ => None }, "uuid": if matches!(ctx.source, Some(Who::Player)) { serde_json::json!(format!("{:032x}", game.native_acting_id())) } else { serde_json::Value::Null } });
        if crate::game::paper_bridge::on_refusable(game, "ew_game_event", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
