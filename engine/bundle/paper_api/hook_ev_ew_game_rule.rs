// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_game_rule(game: &mut crate::game::Game<'_>, ctx: &Context) -> ::core::ops::ControlFlow<R<i32>> {
    if crate::game::paper_bridge::hears("ew_game_rule") {
        let payload = serde_json::json!({ "uuid": if game.lan_acting.is_some() { serde_json::json!(format!("{:032x}", game.native_acting_id())) } else { serde_json::Value::Null }, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "rule": ctx.literals.get(1), "value": match ctx.get("value") { Some(Arg::Bool(b)) => Some(b.to_string()), Some(Arg::Int(i)) => Some(i.to_string()), _ => None } });
        if crate::game::paper_bridge::on_refusable(game, "ew_game_rule", payload) {
            return ::core::ops::ControlFlow::Break(Ok(0));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
