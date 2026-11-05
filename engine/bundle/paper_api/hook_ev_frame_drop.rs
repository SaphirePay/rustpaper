// Written by tools/gen_bundle.py.
fn paper_hook_ev_frame_drop(game: &mut crate::game::Game<'_>, by_player: bool) {
    if crate::game::paper_bridge::hears("frame_drop") {
        let payload = serde_json::json!({ "by_player": by_player });
        crate::game::paper_bridge::on_trigger(game, "frame_drop", payload);
    }
}
