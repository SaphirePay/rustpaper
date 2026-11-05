// Written by tools/gen_bundle.py.
fn paper_hook_ev_ew_time(game: &mut crate::game::Game<'_>, id: &str, ctx: &Context) -> ::core::ops::ControlFlow<R<i32>> {
    if crate::game::paper_bridge::hears("ew_time") {
        let payload = serde_json::json!({ "uuid": null, "level": format!("minecraft:{}", game.dim.id().trim_start_matches("minecraft:")), "id": id, "now": game.world.day_time, "time": Game::int(ctx, "time") });
        if crate::game::paper_bridge::on_refusable(game, "ew_time", payload) {
            return ::core::ops::ControlFlow::Break(Ok(0));
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
