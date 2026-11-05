// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_tables_tick(game: &mut crate::game::Game<'_>, host: &mut LanHost) {
    if crate::game::paper_bridge::hears("items_tables_tick") {
        let payload = serde_json::json!({ "uuid": "", "sent": (for conn in crate::game::paper_fam_items::pending_conns(&*host) { game.lan_send_table_content(host, conn); }) });
        crate::game::paper_bridge::on_trigger(game, "items_tables_tick", payload);
    }
}
