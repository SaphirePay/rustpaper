// Written by tools/gen_bundle.py.
fn paper_hook_ev_pl_bucket(game: &mut crate::game::Game<'_>, stack: &ItemStack, pos: [i32; 3], canceled: &mut bool) {
    if crate::game::paper_bridge::hears("pl_bucket") {
        let payload = serde_json::json!({ "x": if true { return crate::game::paper_fam_ev_player::bucket_gate(game, stack, pos, canceled) } else { 0 } });
        crate::game::paper_bridge::on_trigger(game, "pl_bucket", payload);
    }
}
