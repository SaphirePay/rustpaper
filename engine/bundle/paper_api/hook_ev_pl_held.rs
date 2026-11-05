// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_held(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_held") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::run_held(game) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_held", payload);
    }
}
