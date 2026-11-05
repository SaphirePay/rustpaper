// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_stats(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_stats") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::stat_events(game) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_stats", payload);
    }
}
