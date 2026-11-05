// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_sync(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("pl_sync") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::sync_is(|| SYNC.lock().ok().and_then(|sync| sync.as_ref()?.1.as_ref().ok().cloned())) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_sync", payload);
    }
}
