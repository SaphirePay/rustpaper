// Written by tools/gen_bundle.py.
fn paper_hook_ev_enchant_done(game: &mut crate::game::Game<'_>, host: &LanHost, gi: usize, button: usize) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("enchant_done") {
        let payload = serde_json::json!({ "uuid": format!("{:032x}", host.hub.guests[gi].uuid), "button": button });
        if crate::game::paper_bridge::on_refusable(game, "enchant_done", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
