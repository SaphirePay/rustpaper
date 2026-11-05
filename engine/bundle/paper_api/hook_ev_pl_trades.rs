// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_trades(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_trades") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::trades_are(|host, conn| host.play.get_mut(&conn)?.merchant.as_mut().map(|trade| (trade.window, &mut trade.slots))) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_trades", payload);
    }
}
