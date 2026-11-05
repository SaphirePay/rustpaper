// Written by tools/gen_bundle.py.
fn paper_hook_ev_core_void_guest(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32) {
    if crate::game::paper_bridge::hears("core_void_guest") {
        let payload = serde_json::json!({ "done": crate::game::paper_fam_core::void_rule_here(host.hub.guest(conn).map_or("minecraft:overworld", |guest| guest.dimension.as_str())) });
        crate::game::paper_bridge::on_trigger(game, "core_void_guest", payload);
    }
}
