// Written by tools/gen_bundle.py.
fn paper_hook_ev_mob_mount(game: &mut crate::game::Game<'_>, rider: u32, vehicle: u32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("mob_mount") {
        let payload = serde_json::json!({ "rider": rider, "vehicle": vehicle });
        if crate::game::paper_bridge::on_refusable(game, "mob_mount", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
