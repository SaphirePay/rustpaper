// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_border(game: &mut crate::game::Game<'_>, id: &str, ctx: &Context) -> ::core::ops::ControlFlow<R<i32>> {
    if crate::game::paper_bridge::hears("ew_border") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "id": id, "old": game.border.size(), "distance": match ctx.get("distance") { Some(Arg::Double(d)) => Some(*d), _ => None }, "time": match ctx.get("time") { Some(Arg::Int(t)) => i64::from(*t), _ => 0 }, "old_center": game.border.center(), "new_center": ({ let p = game.position_arg(ctx, "pos").unwrap_or_else(|| game.source_position()); [p[0], p[2]] }) });
        if crate::game::paper_bridge::on_refusable(game, "ew_border", payload) {
            return ::core::ops::ControlFlow::Break(Ok(0));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
