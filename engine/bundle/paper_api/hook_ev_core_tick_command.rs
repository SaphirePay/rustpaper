// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_tick_command(game: &mut crate::game::Game<'_>, id: &str, ctx: &Context) {
    if crate::game::paper_bridge::hears("core_tick_command") {
        let payload = serde_json::json!({ "done": crate::game::paper_fam_core::note_step(id, match ctx.get("time") { Some(Arg::Int(v)) => *v, _ => 1 }) });
        crate::game::paper_bridge::on_trigger(game, "core_tick_command", payload);
    }
}
