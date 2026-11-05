// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_lecterns(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_lecterns") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::lecterns_are(|host, conn| host.play.get(&conn)?.lectern.as_ref().map(|lectern| (lectern.window, lectern.key))) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_lecterns", payload);
    }
}
