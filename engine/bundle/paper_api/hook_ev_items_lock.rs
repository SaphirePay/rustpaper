// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_lock(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, pos: (i32, i32, i32)) -> ::core::ops::ControlFlow<bool> {
    if crate::game::paper_bridge::hears("items_lock") {
        let payload = serde_json::json!({ "uuid": "", "refused": (crate::game::paper_fam_items::lock_refuses(game, host, conn, pos)) });
        if crate::game::paper_bridge::on_refusable(game, "items_lock", payload) {
            return ::core::ops::ControlFlow::Break(true);
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
