// Written by tools/gen_bundle.py.
fn paper_hook_ev_guest_attack(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, id: i32) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("guest_attack") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "entity": id });
        if crate::game::paper_bridge::on_refusable(game, "guest_attack", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
