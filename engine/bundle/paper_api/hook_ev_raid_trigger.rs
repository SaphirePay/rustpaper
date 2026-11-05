// Written by tools/gen_bundle.py.
fn paper_hook_ev_raid_trigger(game: &mut crate::game::Game<'_>, at: [i32; 3], amplifier: Option<i32>) -> ::core::ops::ControlFlow<Option<i32>> {
    if crate::game::paper_bridge::hears("raid_trigger") {
        let payload = serde_json::json!({ "pos": at, "amplifier": amplifier });
        if crate::game::paper_bridge::on_refusable(game, "raid_trigger", payload) {
            return ::core::ops::ControlFlow::Break(None);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
