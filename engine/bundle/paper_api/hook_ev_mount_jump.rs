// Written by tools/gen_bundle.py.
fn paper_hook_ev_mount_jump(game: &mut crate::game::Game<'_>, i: usize) {
    if crate::game::paper_bridge::hears("mount_jump") {
        let payload = serde_json::json!({ "index": i });
        crate::game::paper_bridge::on_trigger(game, "mount_jump", payload);
    }
}
