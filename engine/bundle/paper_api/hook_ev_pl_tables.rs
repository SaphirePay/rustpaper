// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_tables(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_tables") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::tables_are(|host, conn| host.play.get_mut(&conn)?.crafting.table.as_mut().map(|table| (table.0, table.3, &mut *table.2))) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_tables", payload);
    }
}
