// Written by tools/gen_bundle.py.
fn paper_hook_ev_mob_egg(game: &mut crate::game::Game<'_>, index: usize) {
    if crate::game::paper_bridge::hears("mob_egg") {
        let payload = serde_json::json!({ "index": index });
        crate::game::paper_bridge::on_trigger(game, "mob_egg", payload);
    }
}
