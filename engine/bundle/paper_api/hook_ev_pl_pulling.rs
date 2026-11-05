// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_pulling(game: &mut crate::game::Game<'_>, source: &View) {
    if crate::game::paper_bridge::hears("pl_pulling") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::pulling(source.parts.first().map(|part| part.0)) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_pulling", payload);
    }
}
