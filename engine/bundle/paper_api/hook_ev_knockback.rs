// Written by tools/gen_bundle.py.
fn paper_hook_ev_knockback(game: &mut crate::game::Game<'_>, power: f64, xd: f64, zd: f64) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("knockback") {
        let payload = serde_json::json!({ "power": power, "x": xd, "z": zd });
        if crate::game::paper_bridge::on_refusable(game, "knockback", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
