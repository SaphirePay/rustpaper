// Written by tools/gen_bundle.py.
fn paper_hook_ev_items_hive_enter(game: &mut crate::game::Game<'_>, mob: u32, at: [i32; 3]) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("items_hive_enter") {
        let payload = serde_json::json!({ "uuid": "", "refused": (crate::game::paper_fam_items::hive_enter(game, mob, at)) });
        if crate::game::paper_bridge::on_refusable(game, "items_hive_enter", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
