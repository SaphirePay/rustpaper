// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_table(game: &mut crate::game::Game<'_>, host: &mut LanHost, conn: u32) {
    if crate::game::paper_bridge::hears("items_table") {
        let payload = serde_json::json!({ "uuid": "", "open": ({ let who = host.hub.guest(conn).map(|g| g.uuid); let hub = &host.hub; match (who, host.play.get_mut(&conn).and_then(|p| p.crafting.table.as_mut())) { (Some(who), Some(table)) => crate::game::paper_fam_items::menu_sent(game, hub, who, table.0, table.3, &mut table.2), _ => serde_json::Value::Null } }) });
        crate::game::paper_bridge::on_trigger(game, "items_table", payload);
    }
}
