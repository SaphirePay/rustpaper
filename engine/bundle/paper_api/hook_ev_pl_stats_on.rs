// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_stats_on(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_stats_on") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::stats_on() } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_stats_on", payload);
    }
}
