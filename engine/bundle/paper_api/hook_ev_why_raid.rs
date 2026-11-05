// Written by tools/gen_bundle.py.
fn paper_hook_ev_why_raid(game: &mut crate::game::Game<'_>) {
    if crate::game::paper_bridge::hears("why_raid") {
        let payload = serde_json::json!({ "reason": "RAID" });
        crate::game::paper_bridge::on_trigger(game, "why_raid", payload);
    }
}
