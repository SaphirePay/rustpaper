// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_beacon(game: &mut crate::game::Game<'_>, p: [i32; 3], b: &Beacon) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("items_beacon") {
        let payload = serde_json::json!({ "uuid": "", "refused": (crate::game::paper_fam_items::beacon_range(game, p, b)) });
        if crate::game::paper_bridge::on_refusable(game, "items_beacon", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
